use crate::{pgn, replay::Replay};
use shakmaty::Position;
use std::time::Duration;

fn annotated() -> Replay {
    Replay::from_annotated(
        pgn::import_annotated(b"1. e4 (1. d4 d5 2. c4 (2. Nf3 Nf6) e6) e5 2. Nf3 Nc6 *").unwrap(),
        true,
    )
}

fn find(r: &Replay, san: &str, parent_san: &str) -> usize {
    r.tree
        .nodes
        .iter()
        .find(|n| n.san == san && n.parent.is_some_and(|p| r.tree.nodes[p].san == parent_san))
        .unwrap()
        .id
}

fn path(r: &Replay) -> Vec<String> {
    r.path_ids()
        .iter()
        .skip(1)
        .map(|id| r.tree.nodes[*id].san.clone())
        .collect()
}

#[test]
fn stable_nodes_nested_branch_navigation_and_mainline_restore() {
    let mut r = annotated();
    let mainline_end = r.node;
    let original = r.game.record.clone();
    let identity = r.identity.clone();
    let d4 = find(&r, "d4", "");
    r.select_node(d4).unwrap();
    assert!(!r.on_mainline());
    assert_eq!(path(&r), ["d4"]);
    assert_eq!(r.position().turn(), shakmaty::Color::Black);
    assert_eq!(r.path_cursor(), 1);
    assert_eq!(r.cursor, 4); // Not an index into the branch position array.
    r.step(1);
    assert_eq!(path(&r), ["d4", "d5"]);
    let choices = r.choices();
    assert_eq!(choices.len(), 2);
    assert_eq!(r.tree.nodes[choices[0]].san, "c4");
    assert_eq!(r.tree.nodes[choices[1]].san, "Nf3");
    r.enter_choice(1).unwrap();
    r.step(1);
    assert_eq!(path(&r), ["d4", "d5", "Nf3", "Nf6"]);
    r.parent_variation();
    assert_eq!(path(&r), ["d4", "d5"]);
    r.step(1);
    assert_eq!(path(&r), ["d4", "d5", "c4"]);
    r.parent_variation();
    assert_eq!(r.node, 0);
    assert!(r.on_mainline());
    r.return_mainline();
    assert_eq!(r.node, mainline_end);
    assert_eq!(r.cursor, 4);
    assert_eq!(r.game.record, original);
    assert_eq!(r.identity, identity);
}

#[test]
fn selected_path_first_last_jump_and_invalid_targets_are_bounded() {
    let tree = pgn::import_annotated(b"1. e4 (1. d4 d5 2. c4 e6 3. Nc3 Nf6) e5 *").unwrap();
    let mut r = Replay::from_annotated(tree, true);
    let original_mainline_node = r.node;
    let d4 = find(&r, "d4", "");
    r.select_node(d4).unwrap();
    assert_eq!(r.path_len(), 6);
    r.jump(usize::MAX);
    assert_eq!(r.path_cursor(), 6);
    assert_eq!(r.tree.nodes[r.node].san, "Nf6");
    r.first();
    assert_eq!(r.node, 0);
    r.step(-1);
    assert_eq!(r.node, 0);
    r.step(1);
    assert_eq!(r.node, d4); // First/previous preserve the chosen future.
    r.last();
    assert_eq!(r.path_cursor(), 6);
    let node = r.node;
    assert!(r.select_node(usize::MAX).is_err());
    assert!(r.enter_choice(usize::MAX).is_err());
    assert_eq!(r.node, node);
    r.return_mainline();
    assert_eq!(r.node, original_mainline_node);
    assert_eq!(r.cursor, 2);
}

#[test]
fn playback_rates_use_controlled_time_and_stop_at_selected_path_end() {
    for (speed, (_, units)) in crate::replay::SPEEDS.iter().enumerate() {
        let mut r = annotated();
        r.first();
        r.set_speed(speed).unwrap();
        r.play();
        let nanos = 2_000_000_000u64.div_ceil(u64::from(*units));
        assert!(!r.tick(Duration::from_nanos(nanos - 1)));
        assert_eq!(r.path_cursor(), 0);
        assert!(r.tick(Duration::from_nanos(1)));
        assert_eq!(r.path_cursor(), 1);
    }
    let mut r = annotated();
    assert_eq!(r.speed_label(), "1×");
    r.first();
    r.set_speed(5).unwrap();
    r.play();
    assert!(r.tick(Duration::from_secs(3600)));
    assert_eq!(r.path_cursor(), 4);
    assert!(!r.playing);
    assert!(!r.tick(Duration::from_secs(1)));
    r.play();
    assert!(!r.playing); // No hidden restart from an endpoint.
    let d4 = find(&r, "d4", "");
    r.select_node(d4).unwrap();
    r.play();
    r.tick(Duration::from_secs(1));
    assert_eq!(path(&r), ["d4", "d5", "c4", "e6"]);
    assert!(!r.playing);
}

#[test]
fn playback_manual_actions_pause_and_metadata_never_changes_review_identity() {
    let mut r = annotated();
    let original_game = r.game.record.clone();
    let identity = r.identity.clone();
    r.tree.nodes[1].after.push("测试 café — комментарий".into());
    r.dirty = true;
    r.first();
    r.play();
    assert!(!r.tick(Duration::from_millis(500)));
    r.jump(0); // Even a no-op manual jump pauses.
    assert!(!r.playing);
    assert!(!r.tick(Duration::from_secs(1)));
    r.toggle_play();
    assert!(r.playing);
    r.cycle_speed();
    assert_eq!(r.speed_label(), "1.5×");
    r.tick(Duration::from_secs(2));
    assert_eq!(r.path_cursor(), 3);
    assert!(r.dirty);
    let d4 = find(&r, "d4", "");
    r.select_node(d4).unwrap();
    assert!(!r.playing);
    assert_eq!(r.game.record, original_game);
    assert_eq!(r.identity, identity);
    assert_eq!(crate::review_cache::game_key(&r.game), identity);
    assert!(r.set_speed(usize::MAX).is_err());
    assert_eq!(r.speed_label(), "1.5×");
}

#[test]
fn playback_catchup_is_bounded_and_branch_position_is_real() {
    let game = pgn::import(b"1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 *").unwrap();
    let mut r = Replay::new(game, false);
    r.first();
    r.set_speed(5).unwrap();
    r.play();
    r.tick(Duration::from_secs(24 * 60 * 60));
    assert_eq!(r.path_cursor(), 8);
    assert!(r.playing);
    r.tick(Duration::from_secs(1));
    assert_eq!(r.path_cursor(), 10);
    assert!(!r.playing);
    assert_eq!(r.position(), &r.game.positions[10]);
}
