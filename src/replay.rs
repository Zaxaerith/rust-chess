use crate::game_record::ValidatedRecord;

#[derive(Clone)]
pub struct Replay {
    pub game: ValidatedRecord,
    pub cursor: usize,
    pub scroll: usize,
    pub external: bool,
}
impl Replay {
    pub fn new(game: ValidatedRecord, external: bool) -> Self {
        let cursor = game.moves.len();
        Self {
            game,
            cursor,
            scroll: cursor.saturating_sub(8),
            external,
        }
    }
    pub fn jump(&mut self, ply: usize) {
        self.cursor = ply.min(self.game.moves.len());
        self.scroll = self.cursor.saturating_sub(5);
    }
    pub fn step(&mut self, delta: isize) {
        self.jump(self.cursor.saturating_add_signed(delta));
    }
}
