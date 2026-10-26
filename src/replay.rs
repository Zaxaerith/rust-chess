use crate::{
    annotated_game::{AnnotatedGame, NodeId},
    game_record::ValidatedRecord,
};
use shakmaty::Chess;
use std::time::Duration;

pub const SPEEDS: [(&str, u32); 6] = [
    ("0.5×", 1),
    ("1×", 2),
    ("1.5×", 3),
    ("2×", 4),
    ("3×", 6),
    ("4×", 8),
];
const MOVE_CREDIT: u128 = 2_000_000_000;
const MAX_TICK: Duration = Duration::from_secs(2);
const MAX_TICK_MOVES: usize = 8;

#[derive(Clone)]
pub struct Replay {
    // The original linear record remains the exclusive target for Game Review.
    pub game: ValidatedRecord,
    // Legacy mainline cursor; branch rendering must use node/position().
    pub cursor: usize,
    pub scroll: usize,
    pub external: bool,
    pub identity: String,
    pub tree: AnnotatedGame,
    pub node: NodeId,
    pub selected_path: Vec<NodeId>,
    pub dirty: bool,
    pub playing: bool,
    pub speed_index: usize,
    mainline_path: Vec<NodeId>,
    mainline_index: Vec<Option<usize>>,
    mainline_return: usize,
    credit: u128,
}
impl Replay {
    #[cfg(test)]
    pub fn new(game: ValidatedRecord, external: bool) -> Self {
        Self::from_annotated(AnnotatedGame::from_linear(game), external)
    }

    pub fn from_annotated(tree: AnnotatedGame, external: bool) -> Self {
        let game = tree.mainline.clone();
        let cursor = game.moves.len();
        let mainline_path = tree.mainline_ids();
        let mut mainline_index = vec![None; tree.nodes.len()];
        for (ply, id) in mainline_path.iter().enumerate() {
            mainline_index[*id] = Some(ply);
        }
        let node = mainline_path[cursor];
        Self {
            identity: crate::review_cache::game_key(&game),
            game,
            cursor,
            scroll: cursor.saturating_sub(8),
            external,
            tree,
            node,
            selected_path: mainline_path.clone(),
            dirty: false,
            playing: false,
            speed_index: 1,
            mainline_path,
            mainline_index,
            mainline_return: cursor,
            credit: 0,
        }
    }

    pub fn position(&self) -> &Chess {
        &self.tree.nodes[self.node].position
    }

    pub fn path_ids(&self) -> Vec<NodeId> {
        self.tree.path(self.node)
    }

    pub fn on_mainline(&self) -> bool {
        self.mainline_index[self.node].is_some()
    }

    pub fn path_cursor(&self) -> usize {
        self.tree.nodes[self.node].ply
    }

    pub fn path_len(&self) -> usize {
        self.selected_path.len().saturating_sub(1)
    }

    fn set_current(&mut self, id: NodeId) {
        self.node = id;
        if let Some(ply) = self.mainline_index[id] {
            self.cursor = ply;
        }
        self.scroll = self.path_cursor().saturating_sub(5);
    }

    fn path_through(&self, id: NodeId) -> Vec<NodeId> {
        let mut path = self.tree.path(id);
        while let Some(&next) = path
            .last()
            .and_then(|node| self.tree.nodes.get(*node))
            .and_then(|node| node.children.first())
        {
            // Model construction validates an acyclic tree. Keep this extension
            // bounded even if a future in-memory caller breaks that invariant.
            if next >= self.tree.nodes.len() || path.contains(&next) {
                break;
            }
            path.push(next);
        }
        path
    }

    pub fn select_node(&mut self, id: NodeId) -> Result<(), String> {
        if id >= self.tree.nodes.len() {
            return Err("Unknown replay node".into());
        }
        self.pause();
        if self.on_mainline() && self.mainline_index[id].is_none() {
            self.mainline_return = self.cursor;
        }
        self.selected_path = self.path_through(id);
        self.set_current(id);
        Ok(())
    }

    pub fn choices(&self) -> Vec<NodeId> {
        self.tree.nodes[self.node].children.clone()
    }

    pub fn enter_choice(&mut self, index: usize) -> Result<(), String> {
        let id = self.tree.nodes[self.node]
            .children
            .get(index)
            .copied()
            .ok_or("Unknown replay variation")?;
        self.select_node(id)
    }

    pub fn parent_variation(&mut self) {
        self.pause();
        let path = self.path_ids();
        let branch_point = path.windows(2).rev().find_map(|edge| {
            (self.tree.nodes[edge[0]].children.first() != Some(&edge[1])).then_some(edge[0])
        });
        if let Some(id) = branch_point {
            self.selected_path = self.path_through(id);
            self.set_current(id);
        }
    }

    pub fn return_mainline(&mut self) {
        self.pause();
        let ply = self.mainline_return.min(self.game.moves.len());
        self.selected_path = self.mainline_path.clone();
        self.set_current(self.mainline_path[ply]);
    }

    pub fn jump(&mut self, ply: usize) {
        self.pause();
        self.set_current(self.selected_path[ply.min(self.path_len())]);
    }

    pub fn step(&mut self, delta: isize) {
        self.jump(self.path_cursor().saturating_add_signed(delta));
    }

    pub fn first(&mut self) {
        self.jump(0);
    }

    pub fn last(&mut self) {
        self.jump(self.path_len());
    }

    pub fn play(&mut self) {
        self.credit = 0;
        self.playing = self.path_cursor() < self.path_len();
    }

    pub fn pause(&mut self) {
        self.playing = false;
        self.credit = 0;
    }

    pub fn toggle_play(&mut self) {
        if self.playing {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn speed_label(&self) -> &'static str {
        SPEEDS.get(self.speed_index).unwrap_or(&SPEEDS[1]).0
    }

    pub fn set_speed(&mut self, index: usize) -> Result<(), String> {
        if index >= SPEEDS.len() {
            return Err("Unknown replay playback speed".into());
        }
        self.speed_index = index;
        self.credit = 0;
        Ok(())
    }

    pub fn cycle_speed(&mut self) {
        self.set_speed((self.speed_index + 1) % SPEEDS.len())
            .expect("bounded speed index");
    }

    /// Replay-only frame update. Huge frame gaps are capped to two seconds and
    /// at most eight nodes, rather than performing an unbounded catch-up loop.
    pub fn tick(&mut self, elapsed: Duration) -> bool {
        if !self.playing {
            return false;
        }
        let units = SPEEDS.get(self.speed_index).unwrap_or(&SPEEDS[1]).1;
        self.credit += elapsed.min(MAX_TICK).as_nanos() * u128::from(units);
        let steps = (self.credit / MOVE_CREDIT).min(MAX_TICK_MOVES as u128) as usize;
        self.credit %= MOVE_CREDIT;
        let start = self.node;
        for _ in 0..steps {
            let next = self.path_cursor() + 1;
            if next > self.path_len() {
                self.pause();
                break;
            }
            self.set_current(self.selected_path[next]);
            if self.path_cursor() == self.path_len() {
                self.pause();
                break;
            }
        }
        self.node != start
    }
}
