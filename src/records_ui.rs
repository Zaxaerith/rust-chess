use crate::{
    App, Instant, Key, Position, Screen,
    assets::{fill_rect, fill_rect_alpha},
    font::TextRenderer,
    i18n::Language,
    render::{UiAction, ViewState},
    theme::Palette,
};

#[derive(Clone, Copy, Debug)]
pub enum Action {
    OpenPgn,
    ExportPgn,
    CopyFen,
    InputFen,
    ApplyFen,
    PasteFen,
    Cancel,
    Replay,
    Back,
    First,
    Previous,
    Next,
    Last,
    Jump(usize),
    Resume,
    New,
    Later,
    Dismiss,
}
#[derive(Clone, Copy)]
pub enum Text {
    Open,
    Export,
    Replay,
    CopyFen,
    InputFen,
    Back,
    Resume,
    New,
    Later,
    Apply,
    Cancel,
    Paste,
    Error,
    MainlineOnly,
    ReadOnly,
    Live,
    Notice,
    SaveFailed,
}

pub fn label(lang: Language, text: Text) -> &'static str {
    let words = match lang {
        Language::Chinese => [
            "打开棋谱",
            "导出 PGN",
            "复盘",
            "复制 FEN",
            "导入 FEN",
            "返回",
            "继续上次对局",
            "开始新局",
            "暂不恢复",
            "应用局面",
            "取消",
            "粘贴",
            "操作失败",
            "仅载入主线；注释、评价与变化未保留，导出也不包含它们。",
            "外部只读棋谱",
            "当前对局复盘",
            "提示",
            "自动存档失败",
        ],
        Language::TraditionalChinese => [
            "開啟棋譜",
            "匯出 PGN",
            "復盤",
            "複製 FEN",
            "匯入 FEN",
            "返回",
            "繼續上次對局",
            "開始新局",
            "暫不恢復",
            "套用局面",
            "取消",
            "貼上",
            "操作失敗",
            "僅載入主線；註解、評價與變化未保留，匯出亦不包含。",
            "外部唯讀棋譜",
            "目前對局復盤",
            "提示",
            "自動存檔失敗",
        ],
        Language::English => [
            "Open PGN",
            "Export PGN",
            "Replay",
            "Copy FEN",
            "Load FEN",
            "Back",
            "Resume game",
            "New game",
            "Not now",
            "Apply position",
            "Cancel",
            "Paste",
            "Operation failed",
            "Mainline only: comments, NAGs and variations were omitted and will not be exported.",
            "Imported, read only",
            "Live game replay",
            "Notice",
            "Autosave failed",
        ],
        Language::French => [
            "Ouvrir PGN",
            "Exporter PGN",
            "Revoir",
            "Copier FEN",
            "Charger FEN",
            "Retour",
            "Reprendre",
            "Nouvelle partie",
            "Plus tard",
            "Appliquer",
            "Annuler",
            "Coller",
            "Échec",
            "Ligne principale seule : commentaires, NAG et variantes omis, y compris à l'export.",
            "Import en lecture seule",
            "Relecture de partie",
            "Information",
            "Échec sauvegarde",
        ],
        Language::Spanish => [
            "Abrir PGN",
            "Exportar PGN",
            "Repetición",
            "Copiar FEN",
            "Cargar FEN",
            "Volver",
            "Continuar partida",
            "Nueva partida",
            "Ahora no",
            "Aplicar",
            "Cancelar",
            "Pegar",
            "Error",
            "Solo línea principal: comentarios, NAG y variantes omitidos, también al exportar.",
            "Importado, solo lectura",
            "Repetición de partida",
            "Aviso",
            "Error de guardado",
        ],
        Language::Latin => [
            "Aperire PGN",
            "Exportare PGN",
            "Repetitio",
            "Copiare FEN",
            "Legere FEN",
            "Redire",
            "Pergere",
            "Novus ludus",
            "Non nunc",
            "Applicare",
            "Revocare",
            "Inserere",
            "Error",
            "Sola series principalis: commentaria et variationes omissa nec exportantur.",
            "Importatum, solum legere",
            "Repetitio ludi",
            "Notitia",
            "Error servandi",
        ],
        Language::Japanese => [
            "PGNを開く",
            "PGN出力",
            "棋譜再生",
            "FENコピー",
            "FEN読込",
            "戻る",
            "対局を再開",
            "新しい対局",
            "後で",
            "局面を適用",
            "キャンセル",
            "貼り付け",
            "操作失敗",
            "本譜のみ読込：コメント・評価・変化は省略し、出力にも含みません。",
            "外部棋譜・読取専用",
            "現在の対局の再生",
            "通知",
            "自動保存失敗",
        ],
        Language::Korean => [
            "PGN 열기",
            "PGN 내보내기",
            "기보 재생",
            "FEN 복사",
            "FEN 불러오기",
            "뒤로",
            "대국 계속",
            "새 대국",
            "나중에",
            "국면 적용",
            "취소",
            "붙여넣기",
            "작업 실패",
            "본선만 불러옵니다. 주석, 평가와 변화는 생략되며 내보내지 않습니다.",
            "외부 읽기 전용 기보",
            "현재 대국 재생",
            "알림",
            "자동 저장 실패",
        ],
        Language::German => [
            "PGN öffnen",
            "PGN speichern",
            "Wiedergabe",
            "FEN kopieren",
            "FEN laden",
            "Zurück",
            "Fortsetzen",
            "Neue Partie",
            "Später",
            "Übernehmen",
            "Abbrechen",
            "Einfügen",
            "Fehler",
            "Nur Hauptlinie: Kommentare, NAGs und Varianten entfallen auch beim Export.",
            "Import, schreibgeschützt",
            "Partiewiedergabe",
            "Hinweis",
            "Speichern fehlgeschlagen",
        ],
        Language::Italian => [
            "Apri PGN",
            "Esporta PGN",
            "Rivedi",
            "Copia FEN",
            "Carica FEN",
            "Indietro",
            "Riprendi",
            "Nuova partita",
            "Più tardi",
            "Applica",
            "Annulla",
            "Incolla",
            "Errore",
            "Solo linea principale: commenti, NAG e varianti omessi anche nell'esportazione.",
            "Importato, sola lettura",
            "Replay della partita",
            "Avviso",
            "Salvataggio fallito",
        ],
        Language::Russian => [
            "Открыть PGN",
            "Экспорт PGN",
            "Повтор",
            "Копировать FEN",
            "Загрузить FEN",
            "Назад",
            "Продолжить",
            "Новая партия",
            "Позже",
            "Применить",
            "Отмена",
            "Вставить",
            "Ошибка",
            "Только главная линия: комментарии, NAG и варианты пропущены и не экспортируются.",
            "Импорт, только чтение",
            "Повтор текущей партии",
            "Сообщение",
            "Ошибка автосохранения",
        ],
    };
    words[text as usize]
}

pub fn record_action(action: UiAction) -> Option<Action> {
    if let UiAction::Record(action) = action {
        Some(action)
    } else {
        None
    }
}
pub struct CharacterInput(pub std::sync::Arc<std::sync::Mutex<Vec<char>>>);
impl minifb::InputCallback for CharacterInput {
    fn add_char(&mut self, c: u32) {
        if let Some(c) = char::from_u32(c)
            && let Ok(mut queue) = self.0.lock()
            && queue.len() < 512
        {
            queue.push(c);
        }
    }
}

impl App {
    pub(crate) fn handle_record_action(&mut self, action: Action) {
        let result: Result<(), String> = (|| {
            match action {
                Action::OpenPgn => self.import_pgn_dialog()?,
                Action::ExportPgn => self.export_pgn_dialog()?,
                Action::CopyFen => {
                    crate::native_files::copy_text(&self.display_fen(), self.records.owner)?
                }
                Action::InputFen => {
                    self.tick_clock();
                    let text = self.display_fen();
                    self.records.fen_cursor = text.len();
                    self.records.fen_input = Some(text);
                }
                Action::ApplyFen => {
                    if let Some(text) = self.records.fen_input.clone() {
                        self.import_fen(&text)?;
                    }
                }
                Action::PasteFen => {
                    let text = crate::native_files::paste_text(self.records.owner)?;
                    if !text.is_ascii() || text.len() > 256 {
                        return Err("FEN requires ASCII text, maximum 256 bytes".into());
                    }
                    self.records.fen_cursor = text.trim().len();
                    self.records.fen_input = Some(text.trim().into());
                }
                Action::Cancel => {
                    self.records.fen_input = None;
                    self.clock_last_tick = Instant::now();
                }
                Action::Dismiss => self.records.message = None,
                Action::Replay => {
                    let game = self.record().validate()?;
                    self.enter_replay(game, false);
                }
                Action::Back => self.leave_replay(),
                Action::First
                | Action::Previous
                | Action::Next
                | Action::Last
                | Action::Jump(_) => {
                    self.cancel_analysis();
                    if let Some(r) = &mut self.records.replay {
                        match action {
                            Action::First => r.jump(0),
                            Action::Previous => r.step(-1),
                            Action::Next => r.step(1),
                            Action::Last => r.jump(r.game.moves.len()),
                            Action::Jump(ply) => r.jump(ply),
                            _ => {}
                        }
                    }
                }
                Action::Resume => {
                    if let Some(save) = self.records.resume.clone() {
                        self.restore_snapshot(save)?;
                    } else if self.records.has_game {
                        self.screen = Screen::Game;
                        self.clock_last_tick = Instant::now();
                        self.maybe_spawn_ai();
                    }
                }
                Action::New => {
                    self.records.resume_prompt = false;
                    self.start_game();
                }
                Action::Later => self.records.resume_prompt = false,
            }
            Ok(())
        })();
        if let Err(e) = result {
            self.records.message = Some(format!(
                "{}: {e}",
                label(self.settings.language, Text::Error)
            ));
        }
    }
    pub(crate) fn record_keys(&mut self, keys: &[Key], chars: &[char], ctrl: bool) {
        if self.records.message.is_some() {
            if keys.contains(&Key::Enter) || keys.contains(&Key::Escape) {
                self.records.message = None;
            }
            return;
        }
        if self.records.resume_prompt {
            if keys.contains(&Key::Escape) {
                self.records.resume_prompt = false;
            }
            return;
        }
        if self.records.fen_input.is_some() {
            if keys.contains(&Key::Escape) {
                self.handle_record_action(Action::Cancel);
                return;
            }
            if keys.contains(&Key::Enter) {
                self.handle_record_action(Action::ApplyFen);
                return;
            }
            if ctrl && keys.contains(&Key::V) {
                self.handle_record_action(Action::PasteFen);
                return;
            }
            if let Some(text) = &mut self.records.fen_input {
                let cursor = &mut self.records.fen_cursor;
                *cursor = (*cursor).min(text.len());
                // FEN is ASCII; keep input caret navigation separate from replay shortcuts.
                if ctrl && keys.contains(&Key::A) {
                    text.clear();
                    *cursor = 0;
                }
                if keys.contains(&Key::Home) {
                    *cursor = 0;
                }
                if keys.contains(&Key::End) {
                    *cursor = text.len();
                }
                if keys.contains(&Key::Left) {
                    *cursor = cursor.saturating_sub(1);
                }
                if keys.contains(&Key::Right) {
                    *cursor = (*cursor + 1).min(text.len());
                }
                if keys.contains(&Key::Backspace) && *cursor > 0 {
                    *cursor -= 1;
                    text.remove(*cursor);
                }
                if keys.contains(&Key::Delete) && *cursor < text.len() {
                    text.remove(*cursor);
                }
                if !ctrl {
                    for c in chars {
                        if c.is_ascii() && !c.is_control() && text.len() < 256 {
                            text.insert(*cursor, *c);
                            *cursor += 1;
                        }
                    }
                }
            }
            return;
        }
        if self.screen == Screen::Replay {
            for key in keys {
                match key {
                    Key::Left => self.handle_record_action(Action::Previous),
                    Key::Right => self.handle_record_action(Action::Next),
                    Key::Home => self.handle_record_action(Action::First),
                    Key::End => self.handle_record_action(Action::Last),
                    Key::Escape => self.handle_record_action(Action::Back),
                    _ => {}
                }
            }
        } else if keys.contains(&Key::Escape) {
            self.exit_requested = true;
        }
    }
    pub(crate) fn replay_view(&self, view: &mut ViewState) {
        if let Some(r) = &self.records.replay {
            view.pos = r.game.positions[r.cursor].clone();
            view.history_sans = r.game.sans.clone();
            view.selected = None;
            view.last_move = r.cursor.checked_sub(1).map(|i| {
                let mv = &r.game.moves[i];
                (mv.from().expect("standard chess"), mv.to())
            });
            view.legal_targets.clear();
            view.ai_thinking = false;
            view.hint_thinking = false;
            view.suggestion = None;
            view.promotion = None;
            view.move_confirmation_pending = false;
            view.animations.clear();
            view.dragging_from = None;
            view.game_over_progress = 0.0;
            view.result = None;
        }
    }
}

struct Canvas<'a> {
    font: &'a TextRenderer,
    buf: &'a mut [u32],
    size: (usize, usize),
    pal: Palette,
    mouse: Option<(f32, f32)>,
    pressed: bool,
    actions: Vec<Action>,
}
impl Canvas<'_> {
    fn button(&mut self, rect: (f32, f32, f32, f32), text: &str, action: Action) {
        let (x, y, w, h) = rect;
        let hover = self
            .mouse
            .is_some_and(|(mx, my)| mx >= x && mx <= x + w && my >= y && my <= y + h);
        fill_rect(
            self.buf,
            self.size,
            x as i32,
            y as i32,
            w as i32,
            h as i32,
            if hover {
                self.pal.button_hover
            } else {
                self.pal.button
            },
        );
        let size = (14.0 * ((w - 12.0) / self.font.text_width(text, 14.0).max(1.0))).min(14.0);
        self.font.draw_text(
            self.buf,
            self.size,
            (x + 6.0, y + h * 0.67),
            text,
            self.pal.text,
            size,
        );
        if hover && self.pressed {
            self.actions.push(action);
        }
    }
    fn line(&mut self, x: f32, y: f32, text: &str, width: f32, color: u32) {
        let size = 16.0 * (width / self.font.text_width(text, 16.0).max(1.0)).min(1.0);
        self.font
            .draw_text(self.buf, self.size, (x, y), text, color, size);
    }
    fn wrapped(&mut self, x: f32, mut y: f32, text: &str, width: f32, max_lines: usize) {
        let mut line = String::new();
        let mut lines = 0;
        for c in text.chars().take(2048) {
            if c == '\n' || self.font.text_width(&format!("{line}{c}"), 16.0) > width {
                self.font
                    .draw_text(self.buf, self.size, (x, y), &line, self.pal.text, 16.0);
                line.clear();
                y += 23.0;
                lines += 1;
                if lines >= max_lines {
                    return;
                }
            }
            if c != '\n' {
                line.push(c);
            }
        }
        self.font
            .draw_text(self.buf, self.size, (x, y), &line, self.pal.text, 16.0);
    }
}

pub fn draw(
    font: &TextRenderer,
    buf: &mut [u32],
    size: (usize, usize),
    app: &App,
    mouse: Option<(f32, f32)>,
    pressed: bool,
) -> Vec<Action> {
    let mut c = Canvas {
        font,
        buf,
        size,
        pal: app.settings.theme.palette(),
        mouse,
        pressed,
        actions: vec![],
    };
    let lang = app.settings.language;
    let w = size.0 as f32;
    let h = size.1 as f32;
    if let Some(r) = &app.records.replay {
        let x = w - 292.0;
        let width = 276.0;
        fill_rect(
            c.buf,
            size,
            x as i32,
            0,
            width as i32,
            h as i32,
            c.pal.panel,
        );
        c.line(
            x + 12.0,
            28.0,
            label(
                lang,
                if r.external {
                    Text::ReadOnly
                } else {
                    Text::Live
                },
            ),
            width - 24.0,
            c.pal.accent,
        );
        let tags = &r.game.record.tags;
        let white = tags.get("White").map(String::as_str).unwrap_or("?");
        let black = tags.get("Black").map(String::as_str).unwrap_or("?");
        c.line(
            x + 12.0,
            56.0,
            &format!("{white} / {black}"),
            width - 24.0,
            c.pal.text,
        );
        c.line(
            x + 12.0,
            82.0,
            &format!(
                "{} · {}/{} · {}",
                r.game.record.result,
                r.cursor,
                r.game.moves.len(),
                if r.game.positions[r.cursor].turn() == shakmaty::Color::White {
                    lang.text().white
                } else {
                    lang.text().black
                }
            ),
            width - 24.0,
            c.pal.text,
        );
        for (i, (text, action)) in [
            ("|<", Action::First),
            ("<", Action::Previous),
            (">", Action::Next),
            (">|", Action::Last),
        ]
        .into_iter()
        .enumerate()
        {
            c.button((x + 12.0 + i as f32 * 64.0, 98.0, 58.0, 32.0), text, action);
        }
        let rows = ((h - 280.0) / 26.0).max(1.0) as usize;
        for (row, i) in (r.scroll..r.game.moves.len()).take(rows).enumerate() {
            let pos = &r.game.positions[i];
            let prefix = format!(
                "{}{}",
                pos.fullmoves(),
                if pos.turn() == shakmaty::Color::White {
                    "."
                } else {
                    "..."
                }
            );
            let text = format!(
                "{} {prefix} {}",
                if r.cursor == i + 1 { ">" } else { " " },
                r.game.sans[i]
            );
            c.button(
                (x + 12.0, 140.0 + row as f32 * 26.0, width - 24.0, 24.0),
                &text,
                Action::Jump(i + 1),
            );
        }
        c.button(
            (x + 12.0, h - 124.0, width - 24.0, 32.0),
            label(lang, Text::Export),
            Action::ExportPgn,
        );
        c.button(
            (x + 12.0, h - 84.0, width - 24.0, 32.0),
            label(lang, Text::CopyFen),
            Action::CopyFen,
        );
        c.button(
            (x + 12.0, h - 44.0, width - 24.0, 32.0),
            label(lang, Text::Back),
            Action::Back,
        );
    } else if app.screen == Screen::Game {
        let width = ((w - 320.0) / 4.0).clamp(70.0, 130.0);
        for (i, (text, action)) in [
            (Text::Export, Action::ExportPgn),
            (Text::Replay, Action::Replay),
            (Text::CopyFen, Action::CopyFen),
            (Text::InputFen, Action::InputFen),
        ]
        .into_iter()
        .enumerate()
        {
            c.button(
                (12.0 + i as f32 * (width + 6.0), 4.0, width, 28.0),
                label(lang, text),
                action,
            );
        }
    } else if app.screen == Screen::Menu {
        c.button(
            (36.0, h - 76.0, 180.0, 30.0),
            label(lang, Text::Open),
            Action::OpenPgn,
        );
        c.button(
            (224.0, h - 76.0, 150.0, 30.0),
            label(lang, Text::InputFen),
            Action::InputFen,
        );
        if app.records.resume.is_some() || app.records.has_game {
            c.button(
                (36.0, h - 40.0, 180.0, 30.0),
                label(lang, Text::Resume),
                Action::Resume,
            );
        }
    }
    if app.records.save_error.is_some() {
        c.line(
            12.0,
            h - 12.0,
            label(lang, Text::SaveFailed),
            w - 32.0,
            c.pal.accent,
        );
    }
    let modal = app.records.message.is_some()
        || app.records.fen_input.is_some()
        || app.records.resume_prompt;
    if modal {
        c.actions.clear();
        fill_rect_alpha(c.buf, size, (0, 0), w as i32, h as i32, 0, 200);
        let bw = (w - 48.0).min(650.0);
        let bh = (h - 48.0).min(330.0);
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
        if let Some(message) = &app.records.message {
            c.line(
                x + 20.0,
                y + 32.0,
                label(lang, Text::Notice),
                bw - 40.0,
                c.pal.accent,
            );
            c.wrapped(x + 20.0, y + 64.0, message, bw - 40.0, 8);
            c.button(
                (x + 20.0, y + bh - 48.0, bw - 40.0, 32.0),
                label(lang, Text::Back),
                Action::Dismiss,
            );
        } else if let Some(fen) = &app.records.fen_input {
            c.line(
                x + 20.0,
                y + 32.0,
                label(lang, Text::InputFen),
                bw - 40.0,
                c.pal.accent,
            );
            let mut shown = fen.clone();
            shown.insert(app.records.fen_cursor.min(shown.len()), '|');
            c.wrapped(x + 20.0, y + 70.0, &shown, bw - 40.0, 7);
            let width = (bw - 56.0) / 3.0;
            for (i, (text, action)) in [
                (Text::Paste, Action::PasteFen),
                (Text::Apply, Action::ApplyFen),
                (Text::Cancel, Action::Cancel),
            ]
            .into_iter()
            .enumerate()
            {
                c.button(
                    (
                        x + 16.0 + i as f32 * (width + 8.0),
                        y + bh - 48.0,
                        width,
                        32.0,
                    ),
                    label(lang, text),
                    action,
                );
            }
        } else {
            c.wrapped(x + 20.0, y + 56.0, label(lang, Text::Resume), bw - 40.0, 3);
            let width = bw - 40.0;
            for (i, (text, action)) in [
                (Text::Resume, Action::Resume),
                (Text::New, Action::New),
                (Text::Later, Action::Later),
            ]
            .into_iter()
            .enumerate()
            {
                c.button(
                    (x + 20.0, y + bh - 144.0 + i as f32 * 44.0, width, 36.0),
                    label(lang, text),
                    action,
                );
            }
        }
    }
    c.actions
}
