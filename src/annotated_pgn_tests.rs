use crate::{annotated_game, fen, pgn};
fn semantic(g: &annotated_game::AnnotatedGame) -> Vec<String> {
    g.nodes
        .iter()
        .map(|n| {
            format!(
                "{:?}|{:?}|{}|{:?}|{:?}|{:?}|{:?}",
                g.path(n.id)
                    .iter()
                    .skip(1)
                    .map(|id| g.nodes[*id].uci.clone().unwrap())
                    .collect::<Vec<_>>(),
                n.children
                    .iter()
                    .map(|id| g.nodes[*id].uci.clone().unwrap())
                    .collect::<Vec<_>>(),
                fen::write(&n.position),
                n.before,
                n.after,
                n.nags,
                n.branch_result
            )
        })
        .collect()
}
#[test]
fn nested_rav_comments_nags_and_unicode_semantic_roundtrip() {
    let text = "[Event \"测试 café\"]\n\n{root} 1. e4! {后注释} ( {前注释} 1. d4 $0 $255 d5 (1... Nf6) 2. c4 (2. Nf3 Nf6) e6) e5?! 2. Nf3 Nc6 *";
    let g = pgn::import_annotated(text.as_bytes()).unwrap();
    let back = pgn::import_annotated(pgn::export_annotated(&g).unwrap().as_bytes()).unwrap();
    assert_eq!(g.mainline.record.moves, back.mainline.record.moves);
    assert_eq!(g.mainline.record.tags, back.mainline.record.tags);
    assert_eq!(semantic(&g), semantic(&back));
    let d4 = g.nodes.iter().find(|n| n.san == "d4").unwrap();
    assert_eq!(d4.parent, Some(0));
    assert_eq!(d4.before, ["前注释"]);
    assert_eq!(d4.nags, [0, 255]);
    assert_eq!(g.nodes[1].nags, [1]);
    assert!(!g.mainline.record.omitted_annotations);
    assert!(
        pgn::import(text.as_bytes())
            .unwrap()
            .record
            .omitted_annotations
    );
}
#[test]
fn branches_validate_special_moves_and_black_first_numbering() {
    for text in [
        "[FEN \"r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 42\"]\n\n42... O-O (42... O-O-O) 43. O-O-O *",
        "[FEN \"7k/5P2/8/8/8/8/8/K7 w - - 0 1\"]\n\n1. f8=Q+ (1. f8=N) Kh7 *",
        "[FEN \"7k/8/8/3pP3/8/8/8/K7 w - d6 0 1\"]\n\n1. e6 (1. exd6) *",
    ] {
        let g = pgn::import_annotated(text.as_bytes()).unwrap();
        let out = pgn::export_annotated(&g).unwrap();
        let back = pgn::import_annotated(out.as_bytes()).unwrap();
        assert_eq!(semantic(&g), semantic(&back));
    }
}
#[test]
fn strict_invalid_branches_glyphs_delimiters_extensions_and_limits() {
    for text in [
        "1. e4 (1. e5) *",
        "1. e4 (1. d4 *",
        "1. e4 ) *",
        "1. e4 () *",
        "(1. e4) *",
        "1. e4 $256 *",
        "1. e4 $-1 *",
        "1. e4!!! *",
        "1. e4 {open *",
        "1. e4 * 1. d4 *",
        "%escape\n1. e4 *",
        "1. e4 (1. d4 {bad { brace}) *",
    ] {
        assert!(pgn::import_annotated(text.as_bytes()).is_err(), "{text}");
    }
    let mut deep = String::from("1. e4 ");
    for _ in 0..65 {
        deep.push_str("(1. d4 ");
    }
    for _ in 0..65 {
        deep.push(')');
    }
    deep.push('*');
    assert!(pgn::import_annotated(deep.as_bytes()).is_err());
    assert!(pgn::import_annotated(format!("1. e4 {{{}}} *", "x".repeat(4097)).as_bytes()).is_err());
    assert!(
        pgn::import_annotated(
            format!("1. e4 {} *", "{} ".repeat(annotated_game::MAX_COMMENTS + 1)).as_bytes()
        )
        .is_err()
    );
    assert!(pgn::import_annotated(format!("1. e4 {} *", "$1 ".repeat(65)).as_bytes()).is_err());
}
#[test]
fn semicolon_comments_and_all_common_glyphs_survive_without_private_extensions() {
    let text =
        "\u{feff}[Event \"A\"]\r\n\r\n1. e4! ;测试 café\r\ne5? 2. Nf3!! Nc6?? 3. Bb5!? a6?! *";
    let g = pgn::import_annotated(text.as_bytes()).unwrap();
    assert_eq!(g.nodes[1].after, ["测试 café\r"]);
    assert_eq!(
        g.nodes
            .iter()
            .skip(1)
            .map(|n| n.nags[0])
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6]
    );
    let back = pgn::import_annotated(pgn::export_annotated(&g).unwrap().as_bytes()).unwrap();
    assert_eq!(semantic(&g), semantic(&back));
}

#[test]
fn cumulative_node_mainline_comment_and_file_limits_are_enforced_before_expansion() {
    let too_many = format!("1. e4 {} *", "(1. d4) ".repeat(annotated_game::MAX_NODES));
    assert!(pgn::import_annotated(too_many.as_bytes()).is_err());
    let too_long = format!("{} Nf3 *", "Nf3 Nf6 Ng1 Ng8 ".repeat(1024));
    assert!(pgn::import_annotated(too_long.as_bytes()).is_err());
    let comment = format!("{{{}}} ", "a".repeat(4096));
    let text = format!("1. e4 {} {{x}} *", comment.repeat(256));
    assert!(pgn::import_annotated(text.as_bytes()).is_err());
    assert!(pgn::import_annotated(&vec![b' '; 4 * 1024 * 1024 + 1]).is_err());
    let input = "{before headers}[Event \"prefix\"]\n1. e4 (1. d4 d5 *) e5 *";
    let g = pgn::import_annotated(input.as_bytes()).unwrap();
    let back = pgn::import_annotated(pgn::export_annotated(&g).unwrap().as_bytes()).unwrap();
    assert_eq!(semantic(&g), semantic(&back));
    assert_eq!(g.mainline.record.tags, back.mainline.record.tags);
}
