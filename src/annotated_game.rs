//! In-memory PGN metadata tree. The authoritative move legality is shakmaty;
//! the existing linear record remains the original mainline and save format.
use crate::{fen, game_record::ValidatedRecord};
use shakmaty::{Chess, Move, Position, san::SanPlus, uci::UciMove};

pub type NodeId = usize;
pub const MAX_NODES: usize = 8192;
pub const MAX_DEPTH: usize = 64;
pub const MAX_COMMENT_BYTES: usize = 4096;
pub const MAX_TOTAL_COMMENT_BYTES: usize = 1024 * 1024;
pub const MAX_COMMENTS: usize = 16384;
pub const MAX_TREE_BUDGET: usize = 24 * 1024 * 1024;
pub const MAX_NAGS_PER_NODE: usize = 64;
pub const MAX_TOTAL_NAGS: usize = 65536;

#[derive(Clone)]
pub struct Node {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    /// First child is the continuation; subsequent children are alternatives.
    pub children: Vec<NodeId>,
    pub mv: Option<Move>,
    pub uci: Option<String>,
    pub san: String,
    pub position: Chess,
    pub ply: usize,
    pub before: Vec<String>,
    pub after: Vec<String>,
    pub nags: Vec<u8>,
    /// Optional result inside a RAV; it does not replace the game result.
    pub branch_result: Option<String>,
}
impl Node {
    pub(crate) fn root(position: Chess) -> Self {
        Self {
            id: 0,
            parent: None,
            children: vec![],
            mv: None,
            uci: None,
            san: String::new(),
            position,
            ply: 0,
            before: vec![],
            after: vec![],
            nags: vec![],
            branch_result: None,
        }
    }
}
#[derive(Clone)]
pub struct AnnotatedGame {
    pub mainline: ValidatedRecord,
    pub nodes: Vec<Node>,
}
impl AnnotatedGame {
    pub fn from_linear(mainline: ValidatedRecord) -> Self {
        let mut nodes = vec![Node::root(mainline.positions[0].clone())];
        for (i, mv) in mainline.moves.iter().enumerate() {
            nodes[i].children.push(i + 1);
            nodes.push(Node {
                id: i + 1,
                parent: Some(i),
                children: vec![],
                mv: Some(mv.clone()),
                uci: Some(UciMove::from_standard(mv).to_string()),
                san: mainline.sans[i].clone(),
                position: mainline.positions[i + 1].clone(),
                ply: i + 1,
                before: vec![],
                after: vec![],
                nags: vec![],
                branch_result: None,
            });
        }
        Self { mainline, nodes }
    }
    pub fn mainline_ids(&self) -> Vec<NodeId> {
        let mut result = vec![0];
        let mut node = 0;
        while let Some(&next) = self.nodes.get(node).and_then(|n| n.children.first()) {
            if next >= self.nodes.len() || result.len() > self.nodes.len() {
                return vec![];
            }
            result.push(next);
            node = next;
        }
        result
    }
    pub fn path(&self, id: NodeId) -> Vec<NodeId> {
        if id >= self.nodes.len() {
            return vec![];
        }
        let mut result = vec![];
        let mut node = Some(id);
        while let Some(i) = node {
            if i >= self.nodes.len() || result.len() > self.nodes.len() {
                return vec![];
            }
            result.push(i);
            node = self.nodes[i].parent;
        }
        result.reverse();
        result
    }
    pub fn has_annotations(&self) -> bool {
        self.nodes.iter().any(|n| {
            n.children.len() > 1
                || !n.before.is_empty()
                || !n.after.is_empty()
                || !n.nags.is_empty()
                || n.branch_result.is_some()
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.nodes.is_empty() || self.nodes.len() > MAX_NODES + 1 {
            return Err("PGN tree node limit".into());
        }
        self.mainline.record.validate()?;
        let root = &self.nodes[0];
        if root.id != 0
            || root.parent.is_some()
            || root.mv.is_some()
            || root.uci.is_some()
            || !root.san.is_empty()
            || !root.after.is_empty()
            || !root.nags.is_empty()
            || root.branch_result.is_some()
            || root.ply != 0
            || fen::write(&root.position) != self.mainline.record.initial_fen
        {
            return Err("Invalid PGN root".into());
        }
        let mut comment_bytes = 0;
        let mut comment_count = 0;
        let mut total_nags = 0;
        let mut seen = vec![false; self.nodes.len()];
        seen[0] = true;
        for (i, node) in self.nodes.iter().enumerate() {
            if i != node.id {
                return Err("Unstable PGN node ID".into());
            }
            for comment in node.before.iter().chain(&node.after) {
                validate_comment(comment)?;
                comment_bytes += comment.len();
                comment_count += 1;
            }
            total_nags += node.nags.len();
            if comment_count > MAX_COMMENTS
                || comment_bytes > MAX_TOTAL_COMMENT_BYTES
                || node.nags.len() > MAX_NAGS_PER_NODE
                || total_nags > MAX_TOTAL_NAGS
                || self.nodes.len() * 2048 + comment_bytes + comment_count * 32 + total_nags
                    > MAX_TREE_BUDGET
            {
                return Err("PGN annotation/memory budget limit".into());
            }
            for &child in &node.children {
                if child <= i
                    || child >= self.nodes.len()
                    || seen[child]
                    || self.nodes[child].parent != Some(i)
                {
                    return Err("Invalid PGN child relationship".into());
                }
                seen[child] = true;
            }
            if i == 0 {
                continue;
            }
            let parent = node.parent.filter(|p| *p < i).ok_or("Invalid PGN parent")?;
            let before = &self.nodes[parent].position;
            let mv = node.mv.as_ref().ok_or("PGN node has no move")?;
            if !before.is_legal(mv)
                || node.ply != self.nodes[parent].ply + 1
                || node.san != SanPlus::from_move(before.clone(), mv).to_string()
                || node.uci.as_deref() != Some(UciMove::from_standard(mv).to_string().as_str())
            {
                return Err("PGN node move mismatch".into());
            }
            let mut after = before.clone();
            after.play_unchecked(mv);
            if fen::write(&after) != fen::write(&node.position) {
                return Err("PGN node position mismatch".into());
            }
            if !node.before.is_empty() && self.nodes[parent].children.first() == Some(&i) {
                return Err("Before-comments are supported at game/RAV starts only".into());
            }
            if let Some(result) = &node.branch_result
                && (!node.children.is_empty()
                    || !["*", "1-0", "0-1", "1/2-1/2"].contains(&result.as_str()))
            {
                return Err("Invalid variation result".into());
            }
        }
        if seen.iter().any(|v| !*v) {
            return Err("Unreachable PGN node".into());
        }
        let ids = self.mainline_ids();
        if ids.len() != self.mainline.positions.len() {
            return Err("PGN mainline length mismatch".into());
        }
        for (ply, &id) in ids.iter().enumerate() {
            if fen::write(&self.nodes[id].position) != fen::write(&self.mainline.positions[ply]) {
                return Err("PGN mainline differs from linear record".into());
            }
        }
        Ok(())
    }
}
/// PGN braces cannot escape braces. Reject unsafe edits/imports explicitly,
/// rather than truncate or silently alter the user's comment text.
pub fn validate_comment(text: &str) -> Result<(), String> {
    if text.len() > MAX_COMMENT_BYTES {
        return Err("PGN comment exceeds 4096 UTF-8 bytes".into());
    }
    if text.contains(['{', '}'])
        || text
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err("PGN comments cannot contain braces or unsafe control characters".into());
    }
    Ok(())
}
