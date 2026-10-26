use crate::{
    App,
    advanced_app::{Action as A, EditKind},
    assets::{fill_rect, fill_rect_alpha},
    font::TextRenderer,
    i18n::Language,
    records_ui::{Action, Canvas},
};
pub fn words(lang: Language) -> [&'static str; 24] {
    match lang {
        Language::Chinese => [
            "注释复盘",
            "主线",
            "变化",
            "播放",
            "暂停",
            "速度",
            "选择变化",
            "上级变化",
            "回主线",
            "注释",
            "NAG",
            "未导出",
            "注释尚未导出，是否离开？",
            "导出后离开",
            "放弃修改",
            "取消",
            "应用",
            "清除",
            "着前/着后",
            "粘贴",
            "仅主线分析",
            "变化分支不启用单点分析",
            "路径",
            "注释片段",
        ],
        Language::TraditionalChinese => [
            "註釋複盤",
            "主線",
            "變化",
            "播放",
            "暫停",
            "速度",
            "選擇變化",
            "上級變化",
            "回主線",
            "註釋",
            "NAG",
            "未匯出",
            "註釋尚未匯出，是否離開？",
            "匯出後離開",
            "放棄修改",
            "取消",
            "套用",
            "清除",
            "著前/著後",
            "貼上",
            "僅主線分析",
            "變化分支不啟用單點分析",
            "路徑",
            "註釋片段",
        ],
        Language::English => [
            "Annotated replay",
            "Mainline",
            "Variation",
            "Play",
            "Pause",
            "Speed",
            "Choose variation",
            "Parent variation",
            "Mainline",
            "Comment",
            "NAG",
            "Not exported",
            "Annotations not exported. Leave?",
            "Export and leave",
            "Discard edits",
            "Cancel",
            "Apply",
            "Clear",
            "Before/after",
            "Paste",
            "Mainline analysis only",
            "Single-position analysis disabled in branches",
            "Path",
            "Comment excerpt",
        ],
        Language::French => [
            "Relecture annotée",
            "Principale",
            "Variante",
            "Lecture",
            "Pause",
            "Vitesse",
            "Variantes",
            "Variante parente",
            "Principale",
            "Commentaire",
            "NAG",
            "Non exporté",
            "Annotations non exportées. Quitter ?",
            "Exporter et quitter",
            "Abandonner",
            "Annuler",
            "Appliquer",
            "Effacer",
            "Avant/après",
            "Coller",
            "Analyse principale seule",
            "Analyse désactivée dans les variantes",
            "Chemin",
            "Extrait",
        ],
        Language::Spanish => [
            "Repetición anotada",
            "Principal",
            "Variante",
            "Reproducir",
            "Pausa",
            "Velocidad",
            "Variantes",
            "Variante anterior",
            "Principal",
            "Comentario",
            "NAG",
            "Sin exportar",
            "Notas sin exportar. ¿Salir?",
            "Exportar y salir",
            "Descartar",
            "Cancelar",
            "Aplicar",
            "Borrar",
            "Antes/después",
            "Pegar",
            "Análisis de principal",
            "Análisis desactivado en variantes",
            "Ruta",
            "Extracto",
        ],
        Language::Latin => [
            "Repetitio annotata",
            "Principalis",
            "Variatio",
            "Lude",
            "Pausa",
            "Celeritas",
            "Variationes",
            "Variatio superior",
            "Principalis",
            "Commentarius",
            "NAG",
            "Non exportata",
            "Notae non exportatae. Exire?",
            "Exporta et exi",
            "Abice notas",
            "Revoca",
            "Applica",
            "Dele",
            "Ante/post",
            "Insere",
            "Analysis principalis",
            "Analysis variationum inactiva",
            "Iter",
            "Excerptum",
        ],
        Language::Japanese => [
            "注釈付き棋譜",
            "本譜",
            "変化",
            "再生",
            "停止",
            "速度",
            "変化選択",
            "上の変化",
            "本譜へ",
            "コメント",
            "NAG",
            "未保存",
            "注釈は未保存です。終了しますか？",
            "保存して終了",
            "変更を破棄",
            "取消",
            "適用",
            "消去",
            "着手前/後",
            "貼付",
            "本譜のみ解析",
            "変化の単一局面解析は無効",
            "手順",
            "注釈抜粋",
        ],
        Language::Korean => [
            "주석 복기",
            "본보",
            "변형",
            "재생",
            "일시 중지",
            "속도",
            "변형 선택",
            "상위 변형",
            "본보로",
            "주석",
            "NAG",
            "미저장",
            "주석을 저장하지 않았습니다. 나갈까요?",
            "저장 후 나가기",
            "변경 버리기",
            "취소",
            "적용",
            "지우기",
            "착수 전/후",
            "붙여넣기",
            "본보만 분석",
            "변형 단일 국면 분석 비활성",
            "경로",
            "주석 일부",
        ],
        Language::German => [
            "Kommentierte Partie",
            "Hauptlinie",
            "Variante",
            "Abspielen",
            "Pause",
            "Tempo",
            "Varianten",
            "Übergeordnete",
            "Hauptlinie",
            "Kommentar",
            "NAG",
            "Nicht exportiert",
            "Kommentare nicht exportiert. Verlassen?",
            "Export und Ende",
            "Verwerfen",
            "Abbrechen",
            "Anwenden",
            "Löschen",
            "Vorher/nachher",
            "Einfügen",
            "Nur Hauptlinie analysiert",
            "Einzelanalyse in Varianten deaktiviert",
            "Pfad",
            "Auszug",
        ],
        Language::Italian => [
            "Replay annotato",
            "Principale",
            "Variante",
            "Riproduci",
            "Pausa",
            "Velocità",
            "Varianti",
            "Variante superiore",
            "Principale",
            "Commento",
            "NAG",
            "Non esportato",
            "Note non esportate. Uscire?",
            "Esporta ed esci",
            "Scarta",
            "Annulla",
            "Applica",
            "Cancella",
            "Prima/dopo",
            "Incolla",
            "Analisi principale",
            "Analisi disattiva nelle varianti",
            "Percorso",
            "Estratto",
        ],
        Language::Russian => [
            "Партия с заметками",
            "Основная",
            "Вариант",
            "Пуск",
            "Пауза",
            "Скорость",
            "Варианты",
            "Родительский",
            "Основная",
            "Комментарий",
            "NAG",
            "Не экспортировано",
            "Заметки не сохранены. Выйти?",
            "Экспорт и выход",
            "Отбросить",
            "Отмена",
            "Применить",
            "Очистить",
            "До/после",
            "Вставить",
            "Анализ основной линии",
            "Анализ в вариантах отключён",
            "Путь",
            "Отрывок",
        ],
    }
}
fn text(c: &mut Canvas<'_>, x: f32, y: f32, value: &str, width: f32) {
    let mut shown: String = value.chars().take(180).collect();
    let size = 13.0;
    if c.font.text_width(&shown, size) > width {
        while !shown.is_empty() && c.font.text_width(&format!("{shown}…"), size) > width {
            shown.pop();
        }
        shown.push('…');
    }
    c.font
        .draw_text(c.buf, c.size, (x, y), &shown, c.pal.text, size);
}
pub fn draw_panel(
    font: &TextRenderer,
    buf: &mut [u32],
    size: (usize, usize),
    app: &App,
    mouse: Option<(f32, f32)>,
    pressed: bool,
) -> Vec<Action> {
    let Some(r) = &app.records.replay else {
        return vec![];
    };
    let t = words(app.settings.language);
    let x = size.0 as f32 - 292.0;
    let h = size.1 as f32;
    let mut c = Canvas {
        font,
        buf,
        size,
        pal: app.settings.theme.palette(),
        mouse,
        pressed,
        actions: vec![],
    };
    fill_rect(c.buf, size, x as i32, 0, 276, h as i32, c.pal.panel);
    let n = &r.tree.nodes[r.node];
    text(
        &mut c,
        x + 12.0,
        53.0,
        &format!(
            "{} {} #{} {} {}",
            if r.tree.has_annotations() {
                t[0]
            } else {
                t[22]
            },
            if app.needs_replay_save() { t[11] } else { "" },
            r.node,
            if r.on_mainline() { t[1] } else { t[2] },
            crate::records_ui::label(
                app.settings.language,
                if r.game.record.omitted_annotations {
                    crate::records_ui::Text::MainlineOnly
                } else if r.external {
                    crate::records_ui::Text::ReadOnly
                } else {
                    crate::records_ui::Text::Live
                }
            )
        ),
        252.0,
    );
    text(
        &mut c,
        x + 12.0,
        75.0,
        &format!(
            "{}/{} · {} · {} {}",
            r.path_cursor(),
            r.path_len(),
            r.game.record.result,
            n.san,
            n.nags
                .iter()
                .map(|v| crate::advanced_app::nag_label(*v))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        252.0,
    );
    for (i, (s, a)) in [
        ("|<", Action::First),
        ("<", Action::Previous),
        (">", Action::Next),
        (">|", Action::Last),
    ]
    .into_iter()
    .enumerate()
    {
        c.button((x + 12.0 + i as f32 * 64.0, 86.0, 58.0, 26.0), s, a);
    }
    c.button(
        (x + 12.0, 118.0, 122.0, 26.0),
        if r.playing { t[4] } else { t[3] },
        Action::Advanced(A::Play),
    );
    c.button(
        (x + 140.0, 118.0, 124.0, 26.0),
        &format!("{} {}", t[5], r.speed_label()),
        Action::Advanced(A::Speed),
    );
    c.button(
        (x + 12.0, 150.0, 112.0, 26.0),
        &format!("{} ({})", t[6], r.choices().len()),
        Action::Advanced(A::Choices),
    );
    c.button(
        (x + 130.0, 150.0, 64.0, 26.0),
        t[7],
        Action::Advanced(A::Parent),
    );
    c.button(
        (x + 200.0, 150.0, 64.0, 26.0),
        t[8],
        Action::Advanced(A::Mainline),
    );
    c.button(
        (x + 12.0, 182.0, 122.0, 26.0),
        t[9],
        Action::Advanced(A::Comment),
    );
    c.button(
        (x + 140.0, 182.0, 124.0, 26.0),
        t[10],
        Action::Advanced(A::Nags),
    );
    let offset = app.advanced.note_scroll * 16;
    let excerpt: String = n
        .before
        .iter()
        .chain(&n.after)
        .flat_map(|s| s.chars().chain(std::iter::once('\n')))
        .skip(offset)
        .take(32)
        .collect();
    text(
        &mut c,
        x + 12.0,
        226.0,
        &format!("{} [{}+]", t[23], app.advanced.note_scroll),
        204.0,
    );
    c.button(
        (x + 220.0, 211.0, 20.0, 20.0),
        "↑",
        Action::Advanced(A::NoteUp),
    );
    c.button(
        (x + 244.0, 211.0, 20.0, 20.0),
        "↓",
        Action::Advanced(A::NoteDown),
    );
    let first: String = excerpt.chars().take(16).collect();
    let second: String = excerpt.chars().skip(16).collect();
    text(
        &mut c,
        x + 12.0,
        246.0,
        app.advanced
            .error
            .as_deref()
            .unwrap_or(&first.replace('\n', " ")),
        252.0,
    );
    text(&mut c, x + 12.0, 265.0, &second.replace('\n', " "), 252.0);
    let rows = ((h - 404.0) / 24.0).max(2.0) as usize;
    if app.advanced.choices {
        let choices = r.choices();
        for (row, i) in (app.advanced.choice_scroll..choices.len())
            .take(rows)
            .enumerate()
        {
            let id = choices[i];
            let node = &r.tree.nodes[id];
            c.button(
                (x + 12.0, 274.0 + row as f32 * 24.0, 252.0, 22.0),
                &format!("{} {}: {}", t[2], i + 1, node.san),
                Action::Advanced(A::Choose(i)),
            );
        }
    } else {
        for (row, i) in (r.scroll..r.path_len()).take(rows).enumerate() {
            let id = r.selected_path[i + 1];
            let node = &r.tree.nodes[id];
            let pos = &r.tree.nodes[node.parent.expect("move parent")].position;
            use shakmaty::Position;
            c.button(
                (x + 12.0, 274.0 + row as f32 * 24.0, 252.0, 22.0),
                &format!(
                    "{} {}{} {} {}",
                    if id == r.node { ">" } else { "" },
                    pos.fullmoves(),
                    if pos.turn() == shakmaty::Color::White {
                        "."
                    } else {
                        "..."
                    },
                    node.san,
                    node.nags
                        .iter()
                        .map(|v| crate::advanced_app::nag_label(*v))
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                Action::Advanced(A::Node(id)),
            );
        }
    }
    c.button(
        (x + 12.0, h - 124.0, 252.0, 32.0),
        crate::records_ui::label(app.settings.language, crate::records_ui::Text::Export),
        Action::ExportPgn,
    );
    c.button(
        (x + 12.0, h - 84.0, 122.0, 32.0),
        crate::records_ui::label(app.settings.language, crate::records_ui::Text::CopyFen),
        Action::CopyFen,
    );
    c.button(
        (x + 140.0, h - 84.0, 124.0, 32.0),
        crate::library_ui::words(app.settings.language)[if app.records.library_id.is_some() {
            22
        } else {
            23
        }],
        Action::Library(if app.records.library_id.is_some() {
            crate::library_app::Action::Save
        } else {
            crate::library_app::Action::Add
        }),
    );
    c.button(
        (x + 12.0, h - 44.0, 252.0, 32.0),
        crate::records_ui::label(app.settings.language, crate::records_ui::Text::Back),
        Action::Back,
    );
    c.actions
}
pub fn draw_modal(
    font: &TextRenderer,
    buf: &mut [u32],
    size: (usize, usize),
    app: &App,
    mouse: Option<(f32, f32)>,
    pressed: bool,
) -> Vec<Action> {
    if !app.advanced_modal() || app.records.message.is_some() {
        return vec![];
    }
    let t = words(app.settings.language);
    let w = size.0 as f32;
    let h = size.1 as f32;
    let mut c = Canvas {
        font,
        buf,
        size,
        pal: app.settings.theme.palette(),
        mouse,
        pressed,
        actions: vec![],
    };
    fill_rect_alpha(c.buf, size, (0, 0), w as i32, h as i32, 0, 210);
    let bw = (w - 48.0).min(650.0);
    let bh = (h - 48.0).min(350.0);
    let x = (w - bw) / 2.0;
    let y = (h - bh) / 2.0;
    fill_rect(
        c.buf,
        size,
        x as i32,
        y as i32,
        bw as i32,
        bh as i32,
        c.pal.panel,
    );
    if app.advanced.pending.is_some() {
        text(&mut c, x + 16.0, y + 35.0, t[12], bw - 32.0);
        for (i, (s, a)) in [
            (t[13], A::ExportExit),
            (t[14], A::Discard),
            (t[15], A::CancelExit),
        ]
        .into_iter()
        .enumerate()
        {
            c.button(
                (x + 16.0, y + 75.0 + i as f32 * 48.0, bw - 32.0, 36.0),
                s,
                Action::Advanced(a),
            );
        }
    } else if let Some(e) = &app.advanced.editor {
        text(
            &mut c,
            x + 16.0,
            y + 30.0,
            &format!(
                "{} #{} {}",
                if e.kind == EditKind::Nags {
                    t[10]
                } else {
                    t[9]
                },
                e.node,
                match e.kind {
                    EditKind::Before => "<",
                    EditKind::After => ">",
                    EditKind::Nags => "$0..255",
                }
            ),
            bw - 32.0,
        );
        let start = e.text[..e.cursor]
            .char_indices()
            .rev()
            .nth(150)
            .map(|v| v.0)
            .unwrap_or(0);
        let mut shown = e.text[start..].chars().take(330).collect::<String>();
        let caret = e.cursor - start;
        if caret <= shown.len() {
            shown.insert(caret, '|');
        }
        if start > 0 {
            shown.insert(0, '…');
        }
        c.wrapped(x + 16.0, y + 62.0, &shown, bw - 32.0, 6);
        let labels = [
            (t[16], A::Apply),
            (t[15], A::CancelEdit),
            (t[17], A::Clear),
            (t[19], A::Paste),
            (t[18], A::ToggleBefore),
        ];
        let count = if e.kind == EditKind::Nags { 4 } else { 5 };
        let width = (bw - 40.0) / count as f32;
        for (i, (s, a)) in labels.into_iter().take(count).enumerate() {
            c.button(
                (
                    x + 16.0 + i as f32 * (width + 2.0),
                    y + bh - 48.0,
                    width,
                    32.0,
                ),
                s,
                Action::Advanced(a),
            );
        }
    }
    if let Some(error) = &app.advanced.error {
        text(&mut c, x + 16.0, y + bh - 66.0, error, bw - 32.0);
    }
    c.actions
}
