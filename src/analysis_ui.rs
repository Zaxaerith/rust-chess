//! Compact sidebar overlay; board geometry and real game state stay unchanged.
use crate::{
    App, Screen,
    assets::fill_rect,
    font::TextRenderer,
    i18n::Language,
    uci::Status,
    uci_protocol::{Bound, Score},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Toggle,
    Choose,
    Enable,
    Disable,
    Analyze,
    Stop,
    Time,
    Threads,
    Hash,
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
    app.records.fen_input.is_some() || app.records.message.is_some() || app.records.resume_prompt
}
pub fn captures(app: &App, size: (usize, usize), mouse: Option<(f32, f32)>, pressed: bool) -> bool {
    !modal(app)
        && pressed
        && matches!(
            app.screen,
            Screen::Game | Screen::Replay | Screen::Menu | Screen::Settings
        )
        && mouse.is_some_and(|(x, y)| {
            x >= size.0 as f32 - if app.analysis.panel { 292.0 } else { 136.0 }
                && (app.analysis.panel || y <= 32.0)
        })
}
impl App {
    pub(crate) fn handle_analysis_action(&mut self, action: Action) {
        let result = (|| {
            match action {
                Action::Toggle => self.analysis.panel = !self.analysis.panel,
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
struct Canvas<'a> {
    font: &'a TextRenderer,
    buf: &'a mut [u32],
    size: (usize, usize),
    mouse: Option<(f32, f32)>,
    pressed: bool,
    palette: crate::theme::Palette,
    actions: Vec<Action>,
}
impl Canvas<'_> {
    fn text(&mut self, x: f32, y: f32, value: &str, width: f32) {
        let text: String = value.chars().take(100).collect();
        let size = (14.0 * (width / self.font.text_width(&text, 14.0).max(1.0))).min(14.0);
        self.font
            .draw_text(self.buf, self.size, (x, y), &text, self.palette.text, size);
    }
    fn button(&mut self, rect: (f32, f32, f32, f32), text: &str, action: Action) {
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
        58.0,
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
    c.text(x + 12.0, 80.0, words[status], 252.0);
    c.text(
        x + 12.0,
        102.0,
        &app.analysis
            .config
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        252.0,
    );
    c.button((x + 12.0, 112.0, 252.0, 28.0), words[1], Action::Choose);
    let enabled = app.analysis.controller.is_some();
    c.button(
        (x + 12.0, 146.0, 252.0, 28.0),
        if enabled { words[3] } else { words[2] },
        if enabled {
            Action::Disable
        } else {
            Action::Enable
        },
    );
    c.button(
        (x + 12.0, 180.0, 252.0, 28.0),
        &format!("{}: {} ms", words[6], app.analysis.config.movetime),
        Action::Time,
    );
    c.button(
        (x + 12.0, 214.0, 122.0, 28.0),
        &format!("{}: {}", words[7], app.analysis.config.threads),
        Action::Threads,
    );
    c.button(
        (x + 140.0, 214.0, 124.0, 28.0),
        &format!("{}: {} MiB", words[8], app.analysis.config.hash),
        Action::Hash,
    );
    c.button((x + 12.0, 248.0, 154.0, 28.0), words[4], Action::Analyze);
    c.button((x + 172.0, 248.0, 92.0, 28.0), words[5], Action::Stop);
    let score = r
        .info
        .score
        .map(|(score, bound)| {
            format!(
                "{}{}",
                match bound {
                    Bound::Exact => "",
                    Bound::Lower => ">=",
                    Bound::Upper => "<=",
                },
                match score {
                    Score::Cp(cp) => format!("{:+.2}", f64::from(cp) / 100.0),
                    Score::Mate(m) => format!("#{m:+}"),
                }
            )
        })
        .unwrap_or_else(|| "—".into());
    c.text(x + 12.0, 300.0, &format!("{}: {score}", words[9]), 252.0);
    c.text(
        x + 12.0,
        322.0,
        &format!(
            "{}: {}",
            words[10],
            r.info
                .depth
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".into())
        ),
        252.0,
    );
    let san = r
        .best
        .as_deref()
        .and_then(|mv| app.analysis_san(mv))
        .unwrap_or_else(|| "—".into());
    c.text(x + 12.0, 344.0, &format!("{}: {san}", words[11]), 252.0);
    let pv = r
        .info
        .pv
        .as_deref()
        .and_then(|mv| app.analysis_san(mv))
        .unwrap_or_else(|| "—".into());
    c.text(x + 12.0, 366.0, &format!("{}: {pv}", words[12]), 252.0);
    let error = app.analysis.error.as_ref().or(r.error.as_ref());
    if let Some(e) = error {
        c.text(x + 12.0, 390.0, &format!("{}: {e}", words[19]), 252.0);
    } else if !r.warnings.is_empty() {
        c.text(x + 12.0, 390.0, words[22], 252.0);
    } else if !r.author.is_empty() {
        c.text(x + 12.0, 390.0, &r.author, 252.0);
    }
    if app.screen == Screen::Game {
        c.text(
            x + 12.0,
            414.0,
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
                (x + 12.0 + i as f32 * 64.0, h - 40.0, 58.0, 28.0),
                text,
                action,
            );
        }
    }
    c.actions
}
