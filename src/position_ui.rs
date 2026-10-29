//! Paged draft tools and study controls, using the existing board and UCI panel.
use crate::{
    App, Screen,
    assets::{PieceImages, draw_scaled, fill_rect, fill_rect_alpha},
    font::TextRenderer,
    i18n::Language,
    position_workspace::{Action as A, Field, Mode},
};
use shakmaty::Position;

// Indices are shared with notices; all principal operations have 11 translations.
pub fn words(lang: Language) -> [&'static str; 39] {
    match lang {
        Language::Chinese => [
            "局面编辑器",
            "编辑",
            "研究",
            "棋子工具",
            "局面参数",
            "擦除",
            "移动",
            "空棋盘",
            "初始布局",
            "翻转",
            "撤销编辑",
            "重做编辑",
            "清除选格",
            "走棋方",
            "白方",
            "黑方",
            "易位权",
            "吃过路兵",
            "半回合",
            "完整回合",
            "FEN 文本",
            "验证",
            "复制 FEN",
            "导出 PGN",
            "保存工作区",
            "返回",
            "旧起点改变后研究路线失效",
            "草稿：尚未验证",
            "未保存修改",
            "保存 / 放弃 / 继续编辑",
            "局面验证成功",
            "无引擎也可试走",
            "已保存 / 已复制",
            "放弃",
            "继续编辑",
            "粘贴",
            "应用",
            "取消",
            "研究当前局面",
        ],
        Language::TraditionalChinese => [
            "局面編輯器",
            "編輯",
            "研究",
            "棋子工具",
            "局面參數",
            "擦除",
            "移動",
            "空棋盤",
            "初始佈局",
            "翻轉",
            "撤銷編輯",
            "重做編輯",
            "清除選格",
            "行棋方",
            "白方",
            "黑方",
            "易位權",
            "吃過路兵",
            "半回合",
            "完整回合",
            "FEN 文字",
            "驗證",
            "複製 FEN",
            "匯出 PGN",
            "儲存工作區",
            "返回",
            "更改起點使研究路線失效",
            "草稿：尚未驗證",
            "未儲存修改",
            "儲存 / 放棄 / 繼續編輯",
            "局面驗證成功",
            "無引擎也可試走",
            "已儲存 / 已複製",
            "放棄",
            "繼續編輯",
            "貼上",
            "套用",
            "取消",
            "研究目前局面",
        ],
        Language::English => [
            "Position Editor",
            "Edit",
            "Study",
            "Piece tools",
            "FEN fields",
            "Erase",
            "Move",
            "Empty board",
            "Starting position",
            "Flip",
            "Undo edit",
            "Redo edit",
            "Clear square",
            "Turn",
            "White",
            "Black",
            "Castling",
            "En passant",
            "Halfmove",
            "Fullmove",
            "FEN text",
            "Validate",
            "Copy FEN",
            "Export PGN",
            "Save workspace",
            "Back",
            "Changing root invalidates its study line",
            "Draft: not validated",
            "Unsaved changes",
            "Save / discard / keep editing",
            "Position validated",
            "Study without an engine",
            "Saved / copied",
            "Discard",
            "Keep editing",
            "Paste",
            "Apply",
            "Cancel",
            "Study current position",
        ],
        Language::French => [
            "Éditeur de position",
            "Éditer",
            "Étudier",
            "Pièces",
            "Champs FEN",
            "Effacer",
            "Déplacer",
            "Échiquier vide",
            "Position initiale",
            "Retourner",
            "Annuler édition",
            "Rétablir",
            "Vider case",
            "Trait",
            "Blancs",
            "Noirs",
            "Roques",
            "En passant",
            "Demi-coups",
            "Numéro du coup",
            "Texte FEN",
            "Valider",
            "Copier FEN",
            "Exporter PGN",
            "Enregistrer espace",
            "Retour",
            "Changer le départ invalide la ligne",
            "Brouillon non validé",
            "Modifications non sauvées",
            "Enregistrer / ignorer / continuer",
            "Position validée",
            "Étudier sans moteur",
            "Enregistré / copié",
            "Ignorer",
            "Continuer",
            "Coller",
            "Appliquer",
            "Annuler",
            "Étudier cette position",
        ],
        Language::Spanish => [
            "Editor de posición",
            "Editar",
            "Estudiar",
            "Piezas",
            "Campos FEN",
            "Borrar",
            "Mover",
            "Tablero vacío",
            "Posición inicial",
            "Girar",
            "Deshacer edición",
            "Rehacer",
            "Vaciar casilla",
            "Turno",
            "Blancas",
            "Negras",
            "Enroques",
            "Al paso",
            "Medias jugadas",
            "Número de jugada",
            "Texto FEN",
            "Validar",
            "Copiar FEN",
            "Exportar PGN",
            "Guardar espacio",
            "Volver",
            "Cambiar el inicio invalida la línea",
            "Borrador sin validar",
            "Cambios sin guardar",
            "Guardar / descartar / continuar",
            "Posición válida",
            "Estudiar sin motor",
            "Guardado / copiado",
            "Descartar",
            "Seguir editando",
            "Pegar",
            "Aplicar",
            "Cancelar",
            "Estudiar posición actual",
        ],
        Language::Latin => [
            "Editor positionis",
            "Edere",
            "Studere",
            "Pedes et figurae",
            "Campi FEN",
            "Delere",
            "Movere",
            "Tabula vacua",
            "Initium",
            "Vertere",
            "Revocare",
            "Restituere",
            "Vacuare locum",
            "Ordo",
            "Albi",
            "Nigri",
            "Arces",
            "En passant",
            "Semimotus",
            "Numerus motus",
            "Textus FEN",
            "Probare",
            "Copia FEN",
            "Exporta PGN",
            "Serva studium",
            "Retro",
            "Mutatio initii variat lineam",
            "Forma nondum probata",
            "Mutationes non servatae",
            "Serva / omitte / perge",
            "Positio probata",
            "Studium sine machina",
            "Servatum / copiatum",
            "Omitte",
            "Perge edere",
            "Infige",
            "Adhibe",
            "Abroga",
            "Stude positionem",
        ],
        Language::Japanese => [
            "局面エディター",
            "編集",
            "研究",
            "駒ツール",
            "FEN 項目",
            "消去",
            "移動",
            "空の盤",
            "初期配置",
            "反転",
            "編集を戻す",
            "やり直す",
            "選択升を消去",
            "手番",
            "白",
            "黒",
            "キャスリング権",
            "アンパッサン",
            "半手数",
            "手数番号",
            "FEN テキスト",
            "検証",
            "FEN コピー",
            "PGN 書出し",
            "研究を保存",
            "戻る",
            "開始局面変更で研究手順は無効",
            "未検証の下書き",
            "未保存の変更",
            "保存 / 破棄 / 編集を続ける",
            "局面検証済み",
            "エンジンなしで試せます",
            "保存 / コピー済み",
            "破棄",
            "編集を続ける",
            "貼付け",
            "適用",
            "取消",
            "現在局面を研究",
        ],
        Language::Korean => [
            "국면 편집기",
            "편집",
            "연구",
            "기물 도구",
            "FEN 항목",
            "지우기",
            "이동",
            "빈 보드",
            "초기 배치",
            "뒤집기",
            "편집 취소",
            "다시 실행",
            "선택 칸 삭제",
            "차례",
            "백",
            "흑",
            "캐슬링 권리",
            "앙파상",
            "반수",
            "전체 수",
            "FEN 텍스트",
            "검증",
            "FEN 복사",
            "PGN 내보내기",
            "작업 저장",
            "뒤로",
            "시작 국면 변경 시 수순 무효",
            "검증 전 초안",
            "저장 안 된 변경",
            "저장 / 버리기 / 계속 편집",
            "국면 검증 완료",
            "엔진 없이 연구 가능",
            "저장 / 복사 완료",
            "버리기",
            "계속 편집",
            "붙여넣기",
            "적용",
            "취소",
            "현재 국면 연구",
        ],
        Language::German => [
            "Stellungseditor",
            "Bearbeiten",
            "Studieren",
            "Figuren",
            "FEN-Felder",
            "Löschen",
            "Verschieben",
            "Leeres Brett",
            "Grundstellung",
            "Drehen",
            "Änderung zurück",
            "Wiederholen",
            "Feld leeren",
            "Am Zug",
            "Weiß",
            "Schwarz",
            "Rochaderechte",
            "En passant",
            "Halbzüge",
            "Zugnummer",
            "FEN-Text",
            "Prüfen",
            "FEN kopieren",
            "PGN exportieren",
            "Arbeitsraum sichern",
            "Zurück",
            "Neuer Start verwirft alte Zugfolge",
            "Ungeprüfter Entwurf",
            "Ungespeicherte Änderungen",
            "Speichern / verwerfen / weiter",
            "Stellung geprüft",
            "Studie ohne Engine",
            "Gespeichert / kopiert",
            "Verwerfen",
            "Weiter bearbeiten",
            "Einfügen",
            "Anwenden",
            "Abbrechen",
            "Aktuelle Stellung studieren",
        ],
        Language::Italian => [
            "Editor di posizione",
            "Modifica",
            "Studia",
            "Pezzi",
            "Campi FEN",
            "Cancella",
            "Sposta",
            "Scacchiera vuota",
            "Posizione iniziale",
            "Ruota",
            "Annulla modifica",
            "Ripristina",
            "Svuota casa",
            "Turno",
            "Bianco",
            "Nero",
            "Arrocchi",
            "En passant",
            "Semimosse",
            "Numero mossa",
            "Testo FEN",
            "Verifica",
            "Copia FEN",
            "Esporta PGN",
            "Salva spazio",
            "Indietro",
            "Nuovo inizio invalida la linea",
            "Bozza non verificata",
            "Modifiche non salvate",
            "Salva / scarta / continua",
            "Posizione verificata",
            "Studia senza motore",
            "Salvato / copiato",
            "Scarta",
            "Continua modifica",
            "Incolla",
            "Applica",
            "Annulla",
            "Studia posizione attuale",
        ],
        Language::Russian => [
            "Редактор позиции",
            "Правка",
            "Изучение",
            "Фигуры",
            "Поля FEN",
            "Стереть",
            "Переместить",
            "Пустая доска",
            "Начальная позиция",
            "Повернуть",
            "Отмена правки",
            "Повторить",
            "Очистить поле",
            "Ход",
            "Белые",
            "Чёрные",
            "Рокировки",
            "На проходе",
            "Полуходы",
            "Номер хода",
            "Текст FEN",
            "Проверить",
            "Копировать FEN",
            "Экспорт PGN",
            "Сохранить работу",
            "Назад",
            "Новый старт отменяет старую линию",
            "Черновик не проверен",
            "Есть несохранённые правки",
            "Сохранить / сбросить / продолжить",
            "Позиция проверена",
            "Изучение без движка",
            "Сохранено / скопировано",
            "Сбросить",
            "Продолжить",
            "Вставить",
            "Применить",
            "Отмена",
            "Изучить эту позицию",
        ],
    }
}
pub(crate) struct Canvas<'a> {
    pub font: &'a TextRenderer,
    pub buf: &'a mut [u32],
    pub size: (usize, usize),
    pub mouse: Option<(f32, f32)>,
    pub pressed: bool,
    pub pal: crate::theme::Palette,
    pub actions: Vec<A>,
}
impl Canvas<'_> {
    fn text(&mut self, x: f32, y: f32, s: &str, width: f32) {
        let mut s: String = s.chars().take(300).collect();
        if self.font.text_width(&s, 14.0) > width {
            while !s.is_empty() && self.font.text_width(&format!("{s}…"), 14.0) > width {
                s.pop();
            }
            s.push('…');
        }
        self.font
            .draw_text(self.buf, self.size, (x, y), &s, self.pal.text, 14.0);
    }
    fn button(&mut self, rect: (f32, f32, f32, f32), s: &str, a: A, selected: bool) {
        let (x, y, w, h) = rect;
        let hover = self
            .mouse
            .is_some_and(|(mx, my)| mx >= x && mx < x + w && my >= y && my < y + h);
        fill_rect(
            self.buf,
            self.size,
            x as i32,
            y as i32,
            w as i32,
            h as i32,
            if selected {
                self.pal.button_active
            } else if hover {
                self.pal.button_hover
            } else {
                self.pal.button
            },
        );
        self.text(x + 5.0, y + h * 0.70, s, w - 10.0);
        if hover && self.pressed {
            self.actions.push(a);
        }
    }
    fn wrap(&mut self, x: f32, y: f32, s: &str, width: f32, rows: usize) {
        for (n, line) in crate::analysis_ui::wrap_pv(self.font, s, width, rows)
            .iter()
            .enumerate()
        {
            self.text(x, y + n as f32 * 17.0, line, width);
        }
    }
}
pub(crate) fn source_rect(screen: Screen, size: (usize, usize)) -> (f32, f32, f32, f32) {
    if screen == Screen::Game {
        // Between the existing FEN button and UCI toggle, above board coordinates.
        (size.0 as f32 - 284.0, 4.0, 142.0, 28.0)
    } else {
        (24.0, 4.0, (size.0 as f32 - 320.0).clamp(200.0, 280.0), 28.0)
    }
}
pub fn draw(
    font: &TextRenderer,
    images: &PieceImages,
    buf: &mut [u32],
    size: (usize, usize),
    app: &App,
    mouse: Option<(f32, f32)>,
    pressed: bool,
) -> Vec<A> {
    let t = words(app.settings.language);
    let (w, h) = (size.0 as f32, size.1 as f32);
    let mut c = Canvas {
        font,
        buf,
        size,
        mouse,
        pressed,
        pal: app.settings.theme.palette(),
        actions: vec![],
    };
    if app.records.message.is_some()
        || app.records.resume_prompt
        || app.records.fen_input.is_some()
        || app.advanced_modal()
    {
        return vec![];
    }
    match app.screen {
        Screen::Menu => {
            c.button((36.0, 62.0, 230.0, 32.0), t[0], A::Enter, false);
            return c.actions;
        }
        Screen::Game | Screen::Replay => {
            c.button(source_rect(app.screen, size), t[38], A::FromCurrent, false);
            return c.actions;
        }
        Screen::PositionWorkspace => {}
        _ => return vec![],
    }
    let s = &app.workspace;
    let x = w - 292.0;
    if !app.analysis.panel || s.mode == Mode::Edit {
        fill_rect(c.buf, size, x as i32, 0, 292, size.1 as i32, c.pal.panel);
        c.text(
            x + 12.0,
            23.0,
            &format!("{}{}", t[0], if s.dirty { " *" } else { "" }),
            264.0,
        );
        for (i, (label, a)) in [(t[1], A::Edit), (t[2], A::Analyze)]
            .into_iter()
            .enumerate()
        {
            c.button(
                (x + 12.0 + i as f32 * 132.0, 32.0, 126.0, 28.0),
                label,
                a,
                (i == 0) == (s.mode == Mode::Edit),
            );
        }
        if s.mode == Mode::Edit {
            c.button(
                (x + 12.0, 68.0, 168.0, 28.0),
                if s.parameters { t[3] } else { t[4] },
                A::Parameters,
                false,
            );
            c.button((x + 186.0, 68.0, 90.0, 28.0), t[9], A::Flip, s.flipped);
            if !s.parameters {
                for i in 0..12 {
                    let (px, py) = (
                        x + 12.0 + (i % 6) as f32 * 44.0,
                        104.0 + (i / 6) as f32 * 44.0,
                    );
                    c.button((px, py, 40.0, 40.0), "", A::Tool(i), s.tool == i);
                    draw_scaled(
                        c.buf,
                        size,
                        images.get(
                            app.settings.piece_set,
                            if i < 6 {
                                shakmaty::Color::White
                            } else {
                                shakmaty::Color::Black
                            },
                            [
                                shakmaty::Role::King,
                                shakmaty::Role::Queen,
                                shakmaty::Role::Rook,
                                shakmaty::Role::Bishop,
                                shakmaty::Role::Knight,
                                shakmaty::Role::Pawn,
                            ][i % 6],
                        ),
                        px + 4.0,
                        py + 4.0,
                        32.0,
                        32.0,
                    );
                }
                for (i, (label, a, active)) in [
                    (t[5], A::Tool(12), s.tool == 12),
                    (t[6], A::Tool(13), s.tool == 13),
                ]
                .into_iter()
                .enumerate()
                {
                    c.button(
                        (x + 12.0 + i as f32 * 132.0, 198.0, 126.0, 28.0),
                        label,
                        a,
                        active,
                    );
                }
                for (i, (label, a)) in [
                    (t[7], A::Empty),
                    (t[8], A::Starting),
                    (t[10], A::UndoEdit),
                    (t[11], A::RedoEdit),
                ]
                .into_iter()
                .enumerate()
                {
                    c.button(
                        (
                            x + 12.0 + (i % 2) as f32 * 132.0,
                            234.0 + (i / 2) as f32 * 34.0,
                            126.0,
                            28.0,
                        ),
                        label,
                        a,
                        false,
                    );
                }
                c.button((x + 12.0, 302.0, 258.0, 28.0), t[12], A::ClearSquare, false);
            } else {
                c.button(
                    (x + 12.0, 104.0, 258.0, 28.0),
                    &format!(
                        "{}: {}",
                        t[13],
                        if s.draft.setup.turn == shakmaty::Color::White {
                            t[14]
                        } else {
                            t[15]
                        }
                    ),
                    A::Turn,
                    false,
                );
                c.text(x + 12.0, 154.0, t[16], 258.0);
                for (i, ch) in ["K", "Q", "k", "q"].iter().enumerate() {
                    c.button(
                        (x + 12.0 + i as f32 * 66.0, 164.0, 60.0, 28.0),
                        ch,
                        A::Castle(i),
                        s.draft
                            .setup
                            .castling_rights
                            .contains(crate::position_editor::RIGHTS[i]),
                    );
                }
                for (i, (field, label, value)) in [
                    (
                        Field::Ep,
                        t[17],
                        s.draft
                            .setup
                            .ep_square
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "-".into()),
                    ),
                    (Field::Halfmove, t[18], s.draft.setup.halfmoves.to_string()),
                    (Field::Fullmove, t[19], s.draft.setup.fullmoves.to_string()),
                ]
                .into_iter()
                .enumerate()
                {
                    c.button(
                        (x + 12.0, 202.0 + i as f32 * 34.0, 258.0, 28.0),
                        &format!("{label}: {value}"),
                        A::Input(field),
                        false,
                    );
                }
            }
            c.button(
                (x + 12.0, 338.0, 126.0, 28.0),
                t[20],
                A::Input(Field::Fen),
                false,
            );
            c.button((x + 144.0, 338.0, 126.0, 28.0), t[21], A::Validate, false);
        } else {
            for (i, (label, a)) in [
                (t[20], A::Input(Field::Fen)),
                (t[22], A::CopyFen),
                (t[23], A::Export),
                (
                    crate::analysis_ui::words(app.settings.language)[0],
                    A::Engine,
                ),
            ]
            .into_iter()
            .enumerate()
            {
                c.button(
                    (x + 12.0, 76.0 + i as f32 * 34.0, 258.0, 28.0),
                    label,
                    a,
                    false,
                );
            }
            let study = s.study.as_ref().unwrap();
            c.text(
                x + 12.0,
                236.0,
                &format!(
                    "{}/{} · {}",
                    study.cursor,
                    study.moves.len(),
                    if study.position().turn() == shakmaty::Color::White {
                        t[14]
                    } else {
                        t[15]
                    }
                ),
                258.0,
            );
            c.wrap(
                x + 12.0,
                260.0,
                &study.sans[..study.cursor].join(" "),
                258.0,
                3,
            );
            if let Some(result) = study.result() {
                c.wrap(
                    x + 12.0,
                    322.0,
                    &crate::render::workspace_result(app.settings.language, result),
                    258.0,
                    2,
                );
            } else {
                c.text(x + 12.0, 322.0, t[31], 258.0);
            }
        }
        if let Some(error) = &s.error {
            c.wrap(
                x + 12.0,
                386.0,
                &format!(
                    "{}: {error}",
                    crate::records_ui::label(app.settings.language, crate::records_ui::Text::Error)
                ),
                258.0,
                ((h - 438.0) / 17.0).max(1.0) as usize,
            );
        } else if let Some(n) = s.notice {
            c.text(x + 12.0, 386.0, t[n], 258.0);
        } else {
            c.wrap(
                x + 12.0,
                386.0,
                if s.mode == Mode::Edit { t[26] } else { t[31] },
                258.0,
                2,
            );
        }
        c.button((x + 12.0, h - 40.0, 126.0, 28.0), t[24], A::Save, false);
        c.button((x + 144.0, h - 40.0, 126.0, 28.0), t[25], A::Back, false);
    }
    // The board gets 64 px of reserved footer; navigation remains available with
    // the existing UCI sidebar open, including the minimum supported window.
    let width = (w - 324.0).clamp(280.0, 720.0);
    c.text(
        24.0,
        h - 96.0,
        &format!(
            "{}{}",
            if s.mode == Mode::Edit {
                format!("{}: ", t[27])
            } else {
                String::new()
            },
            s.text()
        ),
        width,
    );
    for (i, (label, a)) in [
        ("|<", A::First),
        ("<", A::Previous),
        (">", A::Next),
        (">|", A::Last),
    ]
    .into_iter()
    .enumerate()
    {
        c.button(
            (24.0 + i as f32 * 40.0, h - 80.0, 34.0, 28.0),
            label,
            a,
            false,
        );
    }
    c.button(
        (188.0, h - 80.0, 112.0, 28.0),
        if s.mode == Mode::Edit { t[2] } else { t[1] },
        if s.mode == Mode::Edit {
            A::Analyze
        } else {
            A::Edit
        },
        false,
    );
    c.button((24.0, h - 44.0, 84.0, 28.0), t[9], A::Flip, s.flipped);
    c.button((114.0, h - 44.0, 84.0, 28.0), t[25], A::Back, false);
    c.button(
        (204.0, h - 44.0, 96.0, 28.0),
        "UCI",
        A::Engine,
        app.analysis.panel,
    );
    if s.modal() {
        c.actions.clear();
        fill_rect_alpha(c.buf, size, (0, 0), size.0 as i32, size.1 as i32, 0, 200);
        let bw = (w - 48.0).min(700.0);
        let bh = (h - 48.0).min(340.0);
        let (mx, my) = ((w - bw) / 2.0, (h - bh) / 2.0);
        fill_rect(
            c.buf,
            size,
            mx as i32,
            my as i32,
            bw as i32,
            bh as i32,
            c.pal.panel,
        );
        if s.pending.is_some() {
            c.text(mx + 16.0, my + 32.0, t[28], bw - 32.0);
            c.wrap(mx + 16.0, my + 64.0, t[29], bw - 32.0, 2);
            if s.input.is_some() {
                c.wrap(mx + 16.0, my + 105.0, t[34], bw - 32.0, 2);
            }
            let b = (bw - 48.0) / 3.0;
            for (i, (label, a)) in [(t[24], A::Save), (t[33], A::Discard), (t[34], A::Keep)]
                .into_iter()
                .enumerate()
            {
                c.button(
                    (mx + 12.0 + i as f32 * (b + 12.0), my + bh - 44.0, b, 30.0),
                    label,
                    a,
                    false,
                );
            }
        } else if let Some(i) = &s.input {
            c.text(
                mx + 16.0,
                my + 32.0,
                t[match i.field {
                    Field::Fen => 20,
                    Field::Ep => 17,
                    Field::Halfmove => 18,
                    Field::Fullmove => 19,
                }],
                bw - 32.0,
            );
            let mut shown = i.text.clone();
            shown.insert(i.cursor, '|');
            c.wrap(mx + 16.0, my + 64.0, &shown, bw - 32.0, 6);
            let b = (bw - 48.0) / 3.0;
            for (n, (label, a)) in [
                (t[35], A::Paste),
                (t[36], A::Apply),
                (t[37], A::CancelInput),
            ]
            .into_iter()
            .enumerate()
            {
                c.button(
                    (mx + 12.0 + n as f32 * (b + 12.0), my + bh - 44.0, b, 30.0),
                    label,
                    a,
                    false,
                );
            }
        }
        if let Some(e) = &s.error {
            c.wrap(
                mx + 16.0,
                my + bh - 94.0,
                &format!(
                    "{}: {e}",
                    crate::records_ui::label(app.settings.language, crate::records_ui::Text::Error)
                ),
                bw - 32.0,
                2,
            );
        }
    }
    c.actions
}
