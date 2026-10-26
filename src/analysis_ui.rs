//! Compact sidebar overlay; board geometry and real game state stay unchanged.
use crate::{App, Screen, assets::fill_rect, font::TextRenderer, i18n::Language, uci::Status};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Review(crate::review_ui::ReviewAction),
    Toggle,
    Choose,
    Enable,
    Disable,
    Analyze,
    Stop,
    Time,
    Threads,
    Hash,
    MultiPV,
    Select(usize),
    First,
    Previous,
    Next,
    Last,
}
pub fn words(language: Language) -> [&'static str; 23] {
    match language {
        Language::Chinese => [
            "UCI 分析",
            "选择引擎",
            "启用引擎",
            "关闭引擎",
            "分析当前局面",
            "停止分析",
            "时间",
            "线程",
            "Hash",
            "评分（白方）",
            "深度",
            "最佳着法",
            "主变化",
            "未启用",
            "初始化",
            "就绪",
            "分析中",
            "已完成",
            "已停止",
            "失败",
            "返回棋谱",
            "外部引擎",
            "引擎参数降级",
        ],
        Language::TraditionalChinese => [
            "UCI 分析",
            "選擇引擎",
            "啟用引擎",
            "關閉引擎",
            "分析目前局面",
            "停止分析",
            "時間",
            "執行緒",
            "Hash",
            "評分（白方）",
            "深度",
            "最佳著法",
            "主變化",
            "未啟用",
            "初始化",
            "就緒",
            "分析中",
            "已完成",
            "已停止",
            "失敗",
            "返回棋譜",
            "外部引擎",
            "引擎參數降級",
        ],
        Language::English => [
            "UCI analysis",
            "Choose engine",
            "Enable engine",
            "Close engine",
            "Analyze position",
            "Stop analysis",
            "Time",
            "Threads",
            "Hash",
            "Score (White)",
            "Depth",
            "Best move",
            "PV",
            "Disabled",
            "Initializing",
            "Ready",
            "Analyzing",
            "Complete",
            "Stopped",
            "Failed",
            "Back to record",
            "External engine",
            "Engine option fallback",
        ],
        Language::French => [
            "Analyse UCI",
            "Choisir moteur",
            "Activer moteur",
            "Fermer moteur",
            "Analyser position",
            "Arrêter analyse",
            "Temps",
            "Threads",
            "Hash",
            "Score (Blancs)",
            "Profondeur",
            "Meilleur coup",
            "Variante",
            "Désactivé",
            "Initialisation",
            "Prêt",
            "Analyse",
            "Terminé",
            "Arrêté",
            "Échec",
            "Retour partie",
            "Moteur externe",
            "Options par défaut",
        ],
        Language::Spanish => [
            "Análisis UCI",
            "Elegir motor",
            "Activar motor",
            "Cerrar motor",
            "Analizar posición",
            "Detener análisis",
            "Tiempo",
            "Hilos",
            "Hash",
            "Puntos (blancas)",
            "Profundidad",
            "Mejor jugada",
            "Variante",
            "Desactivado",
            "Iniciando",
            "Listo",
            "Analizando",
            "Completo",
            "Detenido",
            "Error",
            "Volver a partida",
            "Motor externo",
            "Opciones por defecto",
        ],
        Language::Latin => [
            "Analysis UCI",
            "Elige machinam",
            "Activa machinam",
            "Claude machinam",
            "Explora positionem",
            "Siste analysin",
            "Tempus",
            "Fila",
            "Hash",
            "Valor (Albi)",
            "Altitudo",
            "Optimus motus",
            "Variatio",
            "Inactiva",
            "Initium",
            "Parata",
            "Explorans",
            "Perfecta",
            "Sistit",
            "Error",
            "Ad historiam",
            "Machina externa",
            "Optiones ordinariae",
        ],
        Language::Japanese => [
            "UCI 解析",
            "エンジン選択",
            "起動",
            "エンジン終了",
            "現在局面を解析",
            "解析停止",
            "時間",
            "スレッド",
            "Hash",
            "評価（白視点）",
            "深さ",
            "最善手",
            "読み筋",
            "無効",
            "初期化中",
            "準備完了",
            "解析中",
            "完了",
            "停止",
            "失敗",
            "棋譜に戻る",
            "外部エンジン",
            "既定の設定を使用",
        ],
        Language::Korean => [
            "UCI 분석",
            "엔진 선택",
            "엔진 시작",
            "엔진 종료",
            "현재 국면 분석",
            "분석 중지",
            "시간",
            "스레드",
            "Hash",
            "평가 (백 기준)",
            "깊이",
            "최선의 수",
            "주 변형",
            "비활성",
            "초기화",
            "준비",
            "분석 중",
            "완료",
            "중지됨",
            "실패",
            "기보로 돌아가기",
            "외부 엔진",
            "기본 설정 사용",
        ],
        Language::German => [
            "UCI-Analyse",
            "Engine wählen",
            "Engine starten",
            "Engine schließen",
            "Stellung analysieren",
            "Analyse stoppen",
            "Zeit",
            "Threads",
            "Hash",
            "Wertung (Weiß)",
            "Tiefe",
            "Bester Zug",
            "Variante",
            "Deaktiviert",
            "Initialisierung",
            "Bereit",
            "Analyse läuft",
            "Fertig",
            "Gestoppt",
            "Fehler",
            "Zur Partie",
            "Externe Engine",
            "Standardoptionen",
        ],
        Language::Italian => [
            "Analisi UCI",
            "Scegli motore",
            "Avvia motore",
            "Chiudi motore",
            "Analizza posizione",
            "Ferma analisi",
            "Tempo",
            "Thread",
            "Hash",
            "Valore (Bianco)",
            "Profondità",
            "Mossa migliore",
            "Variante",
            "Disabilitato",
            "Avvio",
            "Pronto",
            "Analisi",
            "Completato",
            "Fermato",
            "Errore",
            "Torna alla partita",
            "Motore esterno",
            "Opzioni predefinite",
        ],
        Language::Russian => [
            "Анализ UCI",
            "Выбрать движок",
            "Запустить движок",
            "Закрыть движок",
            "Анализ позиции",
            "Остановить анализ",
            "Время",
            "Потоки",
            "Hash",
            "Оценка (белые)",
            "Глубина",
            "Лучший ход",
            "Вариант",
            "Отключён",
            "Запуск",
            "Готов",
            "Анализ",
            "Завершён",
            "Остановлен",
            "Ошибка",
            "К партии",
            "Внешний движок",
            "Стандартные опции",
        ],
    }
}
fn modal(app: &App) -> bool {
    app.advanced_modal()
        || app.records.fen_input.is_some()
        || app.records.message.is_some()
        || app.records.resume_prompt
}
pub fn captures(app: &App, size: (usize, usize), mouse: Option<(f32, f32)>, pressed: bool) -> bool {
    !modal(app)
        && pressed
        && matches!(
            app.screen,
            Screen::Game | Screen::Replay | Screen::Menu | Screen::Settings
        )
        && mouse.is_some_and(|(x, y)| {
            x >= size.0 as f32
                - if app.analysis.panel || app.screen == Screen::Replay {
                    292.0
                } else {
                    136.0
                }
                && (app.analysis.panel || y <= 32.0)
        })
}
impl App {
    pub(crate) fn handle_analysis_action(&mut self, action: Action) {
        if self.review.owns_engine()
            && matches!(
                action,
                Action::Choose
                    | Action::Analyze
                    | Action::Stop
                    | Action::Time
                    | Action::Threads
                    | Action::Hash
                    | Action::MultiPV
            )
        {
            self.analysis.error = Some(
                "Game Review owns engine; cancel it before manual analysis/configuration".into(),
            );
            return;
        }
        let result = (|| {
            match action {
                Action::Review(action) => self.handle_review_action(action),
                Action::Toggle => {
                    self.analysis.panel = !self.analysis.panel;
                    if !self.analysis.panel {
                        self.cancel_analysis();
                    }
                }
                Action::Choose => {
                    if let Some(path) = crate::native_files::choose_engine(self.records.owner)? {
                        self.close_analysis_engine();
                        self.analysis.config.path = path;
                        self.analysis.error = None;
                        self.save_analysis_config();
                    }
                }
                Action::Enable => self.enable_analysis()?,
                Action::Disable => {
                    self.close_analysis_engine();
                    self.analysis.error = None;
                }
                Action::Analyze => self.request_analysis()?,
                Action::Stop => {
                    self.cancel_analysis();
                    self.analysis.error = None;
                }
                Action::Select(i) => {
                    if self
                        .analysis
                        .report
                        .lines
                        .get(i)
                        .is_some_and(|l| l.is_some())
                    {
                        self.analysis.selected_line = i;
                    }
                }
                Action::MultiPV => {
                    let again = matches!(
                        self.analysis.report.status,
                        Status::Searching | Status::Complete
                    );
                    self.cancel_analysis();
                    self.analysis.config.multipv = self.analysis.config.multipv % 3 + 1;
                    if let Some(c) = &self.analysis.controller {
                        c.set_multipv(self.analysis.config.multipv);
                    }
                    self.save_analysis_config();
                    if again && self.analysis.controller.is_some() {
                        self.request_analysis()?;
                    }
                }
                Action::Time | Action::Threads | Action::Hash => {
                    self.close_analysis_engine();
                    let (value, values): (&mut u32, &[u32]) = match action {
                        Action::Time => (
                            &mut self.analysis.config.movetime,
                            &[100, 500, 1000, 3000, 10000],
                        ),
                        Action::Threads => (&mut self.analysis.config.threads, &[1, 2, 4, 8]),
                        _ => (&mut self.analysis.config.hash, &[16, 32, 64, 128, 256]),
                    };
                    *value = values
                        .iter()
                        .copied()
                        .find(|n| *n > *value)
                        .unwrap_or(values[0]);
                    self.analysis.error = None;
                    self.save_analysis_config();
                }
                action => {
                    let action = match action {
                        Action::First => crate::records_ui::Action::First,
                        Action::Previous => crate::records_ui::Action::Previous,
                        Action::Next => crate::records_ui::Action::Next,
                        _ => crate::records_ui::Action::Last,
                    };
                    self.handle_record_action(action);
                }
            }
            Ok::<_, String>(())
        })();
        if let Err(e) = result {
            self.analysis.error = Some(e);
        }
    }
}
pub(crate) struct Canvas<'a> {
    pub(crate) font: &'a TextRenderer,
    pub(crate) buf: &'a mut [u32],
    pub(crate) size: (usize, usize),
    pub(crate) mouse: Option<(f32, f32)>,
    pub(crate) pressed: bool,
    pub(crate) palette: crate::theme::Palette,
    pub(crate) actions: Vec<Action>,
}
impl Canvas<'_> {
    pub(crate) fn text(&mut self, x: f32, y: f32, value: &str, width: f32) {
        let mut text: String = value.chars().take(200).collect();
        let size = (14.0 * (width / self.font.text_width(&text, 14.0).max(1.0))).clamp(11.0, 14.0);
        if self.font.text_width(&text, size) > width {
            while !text.is_empty() && self.font.text_width(&format!("{text}…"), size) > width {
                text.pop();
            }
            text.push('…');
        }
        self.font
            .draw_text(self.buf, self.size, (x, y), &text, self.palette.text, size);
    }

    pub(crate) fn button(&mut self, rect: (f32, f32, f32, f32), text: &str, action: Action) {
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
            if hover {
                self.palette.button_hover
            } else {
                self.palette.button
            },
        );
        self.text(x + 6.0, y + h * 0.70, text, w - 12.0);
        if hover && self.pressed {
            self.actions.push(action);
        }
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
    if modal(app) {
        return vec![];
    }
    if app.review.panel && app.screen == Screen::Replay {
        return crate::review_ui::draw(font, buf, size, app, mouse, pressed);
    }
    let words = words(app.settings.language);
    let mut c = Canvas {
        font,
        buf,
        size,
        mouse,
        pressed,
        palette: app.settings.theme.palette(),
        actions: vec![],
    };
    let w = size.0 as f32;
    let h = size.1 as f32;
    if app.screen == Screen::Replay {
        c.button(
            (w - 280.0, 4.0, 124.0, 28.0),
            crate::review_ui::words(app.settings.language)[0],
            Action::Review(crate::review_ui::ReviewAction::Open),
        );
    }
    c.button(
        (w - 136.0, 4.0, 124.0, 28.0),
        if app.analysis.panel {
            words[20]
        } else {
            words[0]
        },
        Action::Toggle,
    );
    if !app.analysis.panel {
        return c.actions;
    }
    let x = w - 292.0;
    fill_rect(
        c.buf,
        size,
        x as i32,
        36,
        276,
        (h - 40.0).max(0.0) as i32,
        c.palette.panel,
    );
    let r = &app.analysis.report;
    c.text(
        x + 12.0,
        55.0,
        &format!(
            "{}: {}",
            words[21],
            if r.name.is_empty() { "—" } else { &r.name }
        ),
        252.0,
    );
    let status = if app.analysis.error.is_some() {
        19
    } else {
        match r.status {
            Status::Off => 13,
            Status::Starting => 14,
            Status::Ready => 15,
            Status::Searching => 16,
            Status::Complete => 17,
            Status::Stopped => 18,
            Status::Failed => 19,
        }
    };
    let more = extra_words(app.settings.language);
    c.text(x + 12.0, 76.0, words[status], 252.0);
    let enabled = app.analysis.controller.is_some();
    c.button((x + 12.0, 85.0, 122.0, 27.0), words[1], Action::Choose);
    c.button(
        (x + 140.0, 85.0, 124.0, 27.0),
        if enabled { words[3] } else { words[2] },
        if enabled {
            Action::Disable
        } else {
            Action::Enable
        },
    );
    for (i, (label, value, action)) in [
        (
            words[6],
            format!("{}ms", app.analysis.config.movetime),
            Action::Time,
        ),
        (
            words[7],
            app.analysis.config.threads.to_string(),
            Action::Threads,
        ),
        (
            words[8],
            format!("{}M", app.analysis.config.hash),
            Action::Hash,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        c.button(
            (x + 12.0 + i as f32 * 86.0, 118.0, 80.0, 27.0),
            &format!("{label} {value}"),
            action,
        );
    }
    c.button(
        (x + 12.0, 151.0, 78.0, 27.0),
        &format!("MultiPV {}", app.analysis.config.multipv),
        Action::MultiPV,
    );
    c.button((x + 96.0, 151.0, 92.0, 27.0), words[4], Action::Analyze);
    c.button((x + 194.0, 151.0, 70.0, 27.0), words[5], Action::Stop);
    c.text(
        x + 12.0,
        198.0,
        &format!(
            "{}: {}",
            words[9],
            crate::analysis_board::score_label(
                r.lines
                    .first()
                    .and_then(|v| v.as_ref())
                    .and_then(|v| v.score)
            )
        ),
        252.0,
    );
    let san = r
        .best
        .as_deref()
        .and_then(|mv| app.analysis_san(mv))
        .unwrap_or_else(|| "—".into());
    c.text(
        x + 12.0,
        217.0,
        &format!(
            "{}: {san}  {}: {}/{}",
            words[11],
            more[0],
            r.lines.iter().flatten().count(),
            r.multipv.max(1)
        ),
        252.0,
    );
    if let Some(e) = app.analysis.error.as_ref().or(r.error.as_ref()) {
        c.text(x + 12.0, 237.0, &format!("{}: {e}", words[19]), 252.0);
    } else if r.warnings.iter().any(|s| s.starts_with("MultiPV:")) {
        c.text(x + 12.0, 237.0, more[3], 252.0);
    } else if !r.warnings.is_empty() {
        c.text(x + 12.0, 237.0, words[22], 252.0);
    }
    for i in 0..3 {
        let Some(line) = &r.lines[i] else {
            continue;
        }; // Never invent missing candidates.
        let rect = candidate_rect(size, i);
        let selected = i == app.analysis.selected_line;
        fill_rect(
            c.buf,
            size,
            rect.0 as i32,
            rect.1 as i32,
            rect.2 as i32,
            rect.3 as i32,
            if selected {
                c.palette.button_active
            } else {
                c.palette.button
            },
        );
        if c.pressed
            && c.mouse.is_some_and(|(mx, my)| {
                mx >= rect.0 && mx < rect.0 + rect.2 && my >= rect.1 && my < rect.1 + rect.3
            })
        {
            c.actions.push(Action::Select(i));
        }
        let depth = line
            .depth
            .map(|d| d.to_string())
            .unwrap_or_else(|| "—".into());
        c.text(
            rect.0 + 5.0,
            rect.1 + 15.0,
            &format!(
                "{}#{} {}  {} {}",
                if selected { "> " } else { "" },
                line.rank,
                crate::analysis_board::score_label(line.score),
                words[10],
                depth
            ),
            rect.2 - 10.0,
        );
        let variation = app.analysis.variations[i].as_ref();
        let pv = variation.map(|v| v.san.as_str()).unwrap_or(more[1]);
        let rows = ((rect.3 - 22.0) / 15.0).max(1.0) as usize;
        let value = if variation.is_some_and(|v| v.error.is_some()) {
            format!("{}: {pv}", more[2])
        } else {
            pv.to_owned()
        };
        for (n, text) in wrap_pv(font, &value, rect.2 - 10.0, rows)
            .iter()
            .enumerate()
        {
            c.text(
                rect.0 + 5.0,
                rect.1 + 31.0 + n as f32 * 15.0,
                text,
                rect.2 - 10.0,
            );
        }
    }
    if app.screen == Screen::Replay {
        for (i, (text, action)) in [
            ("|<", Action::First),
            ("<", Action::Previous),
            (">", Action::Next),
            (">|", Action::Last),
        ]
        .into_iter()
        .enumerate()
        {
            c.button(
                (x + 12.0 + i as f32 * 64.0, h - 36.0, 58.0, 26.0),
                text,
                action,
            );
        }
    } else if app.screen == Screen::Game {
        c.text(
            x + 12.0,
            h - 17.0,
            &format!(
                "{} {}s / {} {}s",
                app.settings.language.text().white,
                app.clocks.white.as_secs(),
                app.settings.language.text().black,
                app.clocks.black.as_secs()
            ),
            252.0,
        );
    }
    c.actions
}

pub(crate) fn candidate_rect(size: (usize, usize), i: usize) -> (f32, f32, f32, f32) {
    let row = ((size.1 as f32 - 296.0) / 3.0).max(50.0);
    (
        size.0 as f32 - 280.0,
        246.0 + i as f32 * row,
        252.0,
        row - 5.0,
    )
}
pub(crate) fn wrap_pv(font: &TextRenderer, text: &str, width: f32, rows: usize) -> Vec<String> {
    let mut result = vec![];
    let mut line = String::new();
    for word in text.split_whitespace() {
        let next = if line.is_empty() {
            word.into()
        } else {
            format!("{line} {word}")
        };
        if font.text_width(&next, 14.0) > width && !line.is_empty() {
            result.push(std::mem::take(&mut line));
            if result.len() == rows {
                result.last_mut().unwrap().push_str(" …");
                return result;
            }
            line.push_str(word);
        } else {
            line = next;
        }
    }
    if !line.is_empty() && result.len() < rows {
        result.push(line);
    }
    result
}
/// Candidates / waiting for PV / invalid PV / MultiPV range/unsupported.
pub fn extra_words(lang: Language) -> [&'static str; 4] {
    match lang {
        Language::Chinese => ["候选", "等待变化", "非法变化", "MultiPV 不支持或受限"],
        Language::TraditionalChinese => ["候選", "等待變化", "非法變化", "MultiPV 不支援或受限"],
        Language::English => [
            "Candidates",
            "Waiting for PV",
            "Invalid PV",
            "MultiPV unsupported or limited",
        ],
        Language::French => [
            "Variantes",
            "Attente variante",
            "Variante illégale",
            "MultiPV absent ou limité",
        ],
        Language::Spanish => [
            "Candidatas",
            "Esperando variante",
            "Variante ilegal",
            "MultiPV ausente o limitado",
        ],
        Language::Latin => [
            "Variationes",
            "Exspectans",
            "Variatio illicita",
            "MultiPV abest vel limitata",
        ],
        Language::Japanese => ["候補", "読み筋待機", "不正な読み筋", "MultiPV 非対応・制限"],
        Language::Korean => ["후보", "변형 대기", "불법 변형", "MultiPV 미지원 또는 제한"],
        Language::German => [
            "Kandidaten",
            "Warte auf Variante",
            "Ungültige Variante",
            "MultiPV fehlt oder begrenzt",
        ],
        Language::Italian => [
            "Candidate",
            "Attesa variante",
            "Variante illegale",
            "MultiPV assente o limitato",
        ],
        Language::Russian => [
            "Варианты",
            "Ожидание варианта",
            "Неверный вариант",
            "MultiPV отсутствует / ограничен",
        ],
    }
}
