//! Review is a Replay sidebar tab, never a second board/game UI.
use crate::{
    App, Screen,
    analysis_ui::{Action, Canvas},
    assets::fill_rect,
    font::TextRenderer,
    game_review::State,
    i18n::Language,
    review_chart,
    review_scoring::Quality,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAction {
    Open,
    Back,
    Start,
    Pause,
    Resume,
    Cancel,
    Time,
    Limit,
    Jump(usize),
}
pub fn words(lang: Language) -> [&'static str; 29] {
    match lang {
        Language::Chinese => [
            "整盘复盘",
            "开始分析",
            "暂停",
            "继续",
            "取消",
            "时间",
            "范围",
            "预算",
            "待开始",
            "分析中",
            "已暂停",
            "已完成",
            "已取消",
            "失败",
            "局面",
            "半回合",
            "实战",
            "建议",
            "最佳",
            "良好",
            "不精确",
            "失误",
            "严重失误",
            "未评价",
            "仅指定范围",
            "启发式优势，非胜率",
            "缓存",
            "错误",
            "证据不足",
        ],
        Language::TraditionalChinese => [
            "整盤複盤",
            "開始分析",
            "暫停",
            "繼續",
            "取消",
            "時間",
            "範圍",
            "預算",
            "待開始",
            "分析中",
            "已暫停",
            "已完成",
            "已取消",
            "失敗",
            "局面",
            "半回合",
            "實戰",
            "建議",
            "最佳",
            "良好",
            "不精確",
            "失誤",
            "嚴重失誤",
            "未評價",
            "僅指定範圍",
            "啟發式優勢，非勝率",
            "快取",
            "錯誤",
            "證據不足",
        ],
        Language::English => [
            "Game Review",
            "Start review",
            "Pause",
            "Resume",
            "Cancel",
            "Time",
            "Range",
            "Budget",
            "Idle",
            "Running",
            "Paused",
            "Completed",
            "Cancelled",
            "Failed",
            "Positions",
            "Ply",
            "Played",
            "Suggested",
            "Best",
            "Good",
            "Inaccuracy",
            "Mistake",
            "Blunder",
            "Unrated",
            "Limited range",
            "Heuristic, not win probability",
            "Cache",
            "Error",
            "Insufficient evidence",
        ],
        Language::French => [
            "Revue partie",
            "Analyser",
            "Pause",
            "Reprendre",
            "Annuler",
            "Temps",
            "Plage",
            "Budget",
            "Inactif",
            "En cours",
            "En pause",
            "Terminé",
            "Annulé",
            "Échec",
            "Positions",
            "Demi-coup",
            "Joué",
            "Conseillé",
            "Meilleur",
            "Bon",
            "Imprécision",
            "Erreur",
            "Gaffe",
            "Non évalué",
            "Plage limitée",
            "Heuristique, pas probabilité",
            "Cache",
            "Erreur",
            "Preuves insuffisantes",
        ],
        Language::Spanish => [
            "Revisión",
            "Analizar",
            "Pausa",
            "Continuar",
            "Cancelar",
            "Tiempo",
            "Rango",
            "Presupuesto",
            "Inactivo",
            "En curso",
            "Pausado",
            "Completo",
            "Cancelado",
            "Fallo",
            "Posiciones",
            "Media jugada",
            "Jugada",
            "Sugerida",
            "Mejor",
            "Buena",
            "Imprecisión",
            "Error",
            "Error grave",
            "Sin evaluar",
            "Rango limitado",
            "Heurística, no probabilidad",
            "Caché",
            "Error",
            "Evidencia insuficiente",
        ],
        Language::Latin => [
            "Recensio",
            "Explora",
            "Pausa",
            "Perge",
            "Abroga",
            "Tempus",
            "Ambitus",
            "Summa",
            "Inactiva",
            "Explorans",
            "Pausa",
            "Perfecta",
            "Abrogata",
            "Error",
            "Positiones",
            "Semimotus",
            "Actus",
            "Consilium",
            "Optimus",
            "Bonus",
            "Incertus",
            "Error",
            "Gravis error",
            "Sine nota",
            "Ambitus finitus",
            "Heuristica, non probabilitas",
            "Memoria",
            "Error",
            "Indicia desunt",
        ],
        Language::Japanese => [
            "棋譜レビュー",
            "解析開始",
            "一時停止",
            "再開",
            "取消",
            "時間",
            "範囲",
            "予算",
            "待機",
            "解析中",
            "一時停止",
            "完了",
            "取消済み",
            "失敗",
            "局面",
            "半手",
            "実戦手",
            "推奨手",
            "最善",
            "良い手",
            "不正確",
            "ミス",
            "大悪手",
            "未評価",
            "範囲限定",
            "目安、勝率ではない",
            "キャッシュ",
            "エラー",
            "根拠不足",
        ],
        Language::Korean => [
            "게임 리뷰",
            "분석 시작",
            "일시 중지",
            "계속",
            "취소",
            "시간",
            "범위",
            "예산",
            "대기",
            "분석 중",
            "중지됨",
            "완료",
            "취소됨",
            "실패",
            "국면",
            "반수",
            "실전 수",
            "추천 수",
            "최선",
            "좋음",
            "부정확",
            "실수",
            "큰 실수",
            "미평가",
            "범위 제한",
            "휴리스틱, 승률 아님",
            "캐시",
            "오류",
            "근거 부족",
        ],
        Language::German => [
            "Partieprüfung",
            "Prüfen",
            "Pause",
            "Fortsetzen",
            "Abbrechen",
            "Zeit",
            "Bereich",
            "Budget",
            "Bereit",
            "Läuft",
            "Pausiert",
            "Fertig",
            "Abgebrochen",
            "Fehler",
            "Stellungen",
            "Halbzug",
            "Gespielt",
            "Empfohlen",
            "Bester",
            "Gut",
            "Ungenauigkeit",
            "Fehler",
            "Grober Fehler",
            "Unbewertet",
            "Teilbereich",
            "Heuristik, keine Gewinnchance",
            "Cache",
            "Fehler",
            "Zu wenig Belege",
        ],
        Language::Italian => [
            "Revisione",
            "Analizza",
            "Pausa",
            "Riprendi",
            "Annulla",
            "Tempo",
            "Intervallo",
            "Budget",
            "Inattivo",
            "In corso",
            "In pausa",
            "Completato",
            "Annullato",
            "Errore",
            "Posizioni",
            "Semimossa",
            "Giocata",
            "Suggerita",
            "Migliore",
            "Buona",
            "Imprecisione",
            "Errore",
            "Errore grave",
            "Non valutata",
            "Intervallo limitato",
            "Euristica, non probabilità",
            "Cache",
            "Errore",
            "Prove insufficienti",
        ],
        Language::Russian => [
            "Обзор партии",
            "Начать",
            "Пауза",
            "Продолжить",
            "Отмена",
            "Время",
            "Диапазон",
            "Бюджет",
            "Ожидание",
            "Анализ",
            "Пауза",
            "Готово",
            "Отменено",
            "Ошибка",
            "Позиции",
            "Полуход",
            "Сыграно",
            "Совет",
            "Лучший",
            "Хороший",
            "Неточность",
            "Ошибка",
            "Зевок",
            "Без оценки",
            "Часть партии",
            "Эвристика, не вероятность",
            "Кэш",
            "Ошибка",
            "Мало данных",
        ],
    }
}
pub fn quality_index(q: Quality) -> usize {
    match q {
        Quality::Best => 18,
        Quality::Good => 19,
        Quality::Inaccuracy => 20,
        Quality::Mistake => 21,
        Quality::Blunder => 22,
        Quality::Unrated => 23,
    }
}
impl App {
    pub(crate) fn handle_review_action(&mut self, action: ReviewAction) {
        let result = (|| {
            match action {
                ReviewAction::Open => {
                    if self.screen == Screen::Replay {
                        self.cancel_analysis();
                        self.review.panel = true;
                        self.analysis.panel = true;
                    }
                }
                ReviewAction::Back => self.review.panel = false,
                ReviewAction::Start => self.start_review()?,
                ReviewAction::Pause => self.pause_review(),
                ReviewAction::Resume => self.resume_review()?,
                ReviewAction::Cancel => self.cancel_review(),
                ReviewAction::Jump(ply) => {
                    // A shared ancestor may be a mainline node while Replay's
                    // selected future still follows a variation. Curve points
                    // always select the original mainline before jumping.
                    if let Some(replay) = &mut self.records.replay {
                        replay.return_mainline();
                    }
                    self.handle_record_action(crate::records_ui::Action::Jump(ply))
                }
                ReviewAction::Time | ReviewAction::Limit => {
                    if self.review.owns_engine() {
                        return Err("Cancel review before changing its budget".into());
                    }
                    if action == ReviewAction::Time {
                        self.review.options.movetime = match self.review.options.movetime {
                            100 => 500,
                            500 => 1000,
                            1000 => 2000,
                            _ => 100,
                        };
                    } else {
                        self.review.options.max_plies = match self.review.options.max_plies {
                            40 => 80,
                            80 => 160,
                            160 => 320,
                            320 => 512,
                            _ => 40,
                        };
                    }
                    self.review.job = None;
                }
            }
            Ok::<_, String>(())
        })();
        self.review.error = result.err();
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
        mouse,
        pressed,
        palette: app.settings.theme.palette(),
        actions: vec![],
    };
    let t = words(app.settings.language);
    let x = size.0 as f32 - 292.0;
    let h = size.1 as f32;
    c.button(
        (x + 12.0, 4.0, 124.0, 28.0),
        t[0],
        Action::Review(ReviewAction::Open),
    );
    c.button(
        (x + 156.0, 4.0, 120.0, 28.0),
        "UCI",
        Action::Review(ReviewAction::Back),
    );
    fill_rect(
        c.buf,
        size,
        x as i32,
        36,
        276,
        (h - 40.0) as i32,
        c.palette.panel,
    );
    let replay = app
        .records
        .replay
        .as_ref()
        .expect("review tab requires Replay");
    let job = app
        .review
        .job
        .as_ref()
        .filter(|j| j.game_key == replay.identity);
    let state = job.map(|j| j.state).unwrap_or(State::Idle);
    let label = match state {
        State::Idle => 8,
        State::Running => 9,
        State::Paused => 10,
        State::Completed => 11,
        State::Cancelled => 12,
        State::Failed => 13,
    };
    c.text(
        x + 12.0,
        55.0,
        job.map(|j| j.engine.as_str())
            .unwrap_or(&app.analysis.report.name),
        252.0,
    );
    let limit = job
        .map(|j| j.limit)
        .unwrap_or(replay.game.moves.len().min(app.review.options.max_plies));
    let state_text = if limit < replay.game.moves.len() {
        format!("{} — {}", t[label], t[24])
    } else {
        t[label].to_owned()
    };
    c.text(x + 12.0, 76.0, &state_text, 252.0);
    c.text(
        x + 12.0,
        97.0,
        &format!(
            "{}: {}/{}  {}: {}",
            t[14],
            job.map(|j| j.done()).unwrap_or(0),
            limit + 1,
            t[15],
            job.and_then(|j| j.next()).unwrap_or(limit)
        ),
        252.0,
    );
    c.text(
        x + 12.0,
        117.0,
        &format!(
            "{}: {:.1}s  0..{} / {} {}",
            t[7],
            (limit as f64 + 1.0) * f64::from(app.review.options.movetime) / 1000.0,
            limit,
            replay.game.moves.len(),
            if limit < replay.game.moves.len() {
                t[24]
            } else {
                ""
            }
        ),
        252.0,
    );
    c.button(
        (x + 12.0, 125.0, 122.0, 27.0),
        &format!("{} {}ms", t[5], app.review.options.movetime),
        Action::Review(ReviewAction::Time),
    );
    c.button(
        (x + 140.0, 125.0, 124.0, 27.0),
        &format!("{} {}", t[6], app.review.options.max_plies),
        Action::Review(ReviewAction::Limit),
    );
    c.button(
        (x + 12.0, 158.0, 98.0, 27.0),
        t[1],
        Action::Review(ReviewAction::Start),
    );
    c.button(
        (x + 116.0, 158.0, 72.0, 27.0),
        if state == State::Paused { t[3] } else { t[2] },
        Action::Review(if state == State::Paused {
            ReviewAction::Resume
        } else {
            ReviewAction::Pause
        }),
    );
    c.button(
        (x + 194.0, 158.0, 70.0, 27.0),
        t[4],
        Action::Review(ReviewAction::Cancel),
    );
    let error = app
        .review
        .error
        .as_ref()
        .or_else(|| job.and_then(|j| j.error.as_ref()))
        .or(app.review.cache_error.as_ref());
    c.text(
        x + 12.0,
        207.0,
        error.map(|e| e.as_str()).unwrap_or(t[25]),
        252.0,
    );
    if !replay.on_mainline() {
        c.text(
            x + 12.0,
            250.0,
            crate::advanced_ui::words(app.settings.language)[20],
            252.0,
        );
    }
    if let Some(job) = job.filter(|_| replay.on_mainline()) {
        review_chart::draw(c.buf, size, font, job, replay.cursor, &c.palette);
        if pressed
            && let Some(mouse) = mouse
            && let Some(ply) = review_chart::hit(job, review_chart::rect(size), mouse)
        {
            c.actions.push(Action::Review(ReviewAction::Jump(ply)));
        }
        let y = review_chart::rect(size).1 + review_chart::rect(size).3 + 25.0;
        if let Some(m) = replay.cursor.checked_sub(1).and_then(|i| job.moves.get(i)) {
            c.text(
                x + 12.0,
                y,
                &format!(
                    "{} #{} {}: {} ({})",
                    t[16],
                    m.ply,
                    if m.side == shakmaty::Color::White {
                        app.settings.language.text().white
                    } else {
                        app.settings.language.text().black
                    },
                    m.san,
                    m.uci
                ),
                252.0,
            );
            c.text(
                x + 12.0,
                y + 19.0,
                &format!(
                    "{}: {} {}",
                    t[17],
                    m.recommended_san.as_deref().unwrap_or("—"),
                    if m.matched { "=" } else { "" }
                ),
                252.0,
            );
            c.text(
                x + 12.0,
                y + 38.0,
                &format!(
                    "{}  d{}/{}  Δ{}",
                    t[quality_index(m.quality)],
                    m.depths
                        .0
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "—".into()),
                    m.depths
                        .1
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "—".into()),
                    m.loss
                        .map(|v| format!("{v:.3}"))
                        .unwrap_or_else(|| "—".into())
                ),
                252.0,
            );
            let reason = match m.reason {
                crate::review_scoring::Reason::LowDepth => format!(
                    "{} <10 / ? — {}",
                    crate::analysis_ui::words(app.settings.language)[10],
                    t[28]
                ),
                crate::review_scoring::Reason::DepthGap => format!(
                    "Δ{} >6 — {}",
                    crate::analysis_ui::words(app.settings.language)[10],
                    t[28]
                ),
                crate::review_scoring::Reason::Mate => format!("M — {}", t[23]),
                crate::review_scoring::Reason::NonExact => format!(">= / <= — {}", t[28]),
                _ => if m.reliable { t[25] } else { t[28] }.to_owned(),
            };
            c.text(x + 12.0, y + 57.0, &reason, 252.0);
        }
    }
    for (i, (label, action)) in [
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
            label,
            action,
        );
    }
    c.actions
}
