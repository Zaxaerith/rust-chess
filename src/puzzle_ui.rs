use crate::{
    App, Screen,
    assets::fill_rect,
    font::TextRenderer,
    i18n::Language,
    puzzle_app::{Action as A, RANGES, THEMES},
    puzzle_session::{Outcome, State},
    records_ui::{Action, Canvas},
};
pub fn words(lang: Language) -> [&'static str; 36] {
    match lang {
        Language::Chinese => [
            "战术训练",
            "返回",
            "下一题",
            "重试",
            "提示",
            "查看答案",
            "上一步",
            "下一步",
            "主题",
            "难度",
            "模式",
            "导入 CSV",
            "取消导入",
            "无符合题目",
            "首次答对",
            "辅助完成",
            "已揭示",
            "已跳过",
            "请走棋",
            "对手应答",
            "不符合参考答案；可重试",
            "非法着法",
            "已解答",
            "显示主题",
            "混合",
            "未做优先",
            "错题重练",
            "进度",
            "训练时长",
            "尝试",
            "成绩",
            "处理中",
            "详情 / 答案",
            "已用提示",
            "答案复盘",
            "Lichess 题目评分",
        ],
        Language::TraditionalChinese => [
            "戰術訓練",
            "返回",
            "下一題",
            "重試",
            "提示",
            "查看答案",
            "上一步",
            "下一步",
            "主題",
            "難度",
            "模式",
            "匯入 CSV",
            "取消匯入",
            "無符合題目",
            "首次答對",
            "輔助完成",
            "已揭示",
            "已跳過",
            "請走棋",
            "對手應答",
            "不符合參考答案；可重試",
            "非法著法",
            "已解答",
            "顯示主題",
            "混合",
            "未做優先",
            "錯題重練",
            "進度",
            "訓練時長",
            "嘗試",
            "成績",
            "處理中",
            "詳情 / 答案",
            "已用提示",
            "答案複盤",
            "Lichess 題目評分",
        ],
        Language::English => [
            "Chess Puzzles",
            "Back",
            "Next puzzle",
            "Retry",
            "Hint",
            "Reveal answer",
            "Previous",
            "Next",
            "Theme",
            "Rating",
            "Mode",
            "Import CSV",
            "Cancel import",
            "No matching puzzles",
            "First try",
            "Assisted",
            "Revealed",
            "Skipped",
            "Your move",
            "Opponent reply",
            "Not reference answer; try again",
            "Illegal move",
            "Solved",
            "Show themes",
            "Mixed",
            "Unseen first",
            "Retry errors",
            "Progress",
            "Session time",
            "Attempts",
            "Results",
            "Working",
            "Details / answer",
            "Hint used",
            "Answer replay",
            "Lichess puzzle rating",
        ],
        Language::French => [
            "Tactiques",
            "Retour",
            "Suivant",
            "Réessayer",
            "Indice",
            "Révéler",
            "Précédent",
            "Suivant",
            "Thème",
            "Niveau",
            "Mode",
            "Importer CSV",
            "Annuler import",
            "Aucun problème",
            "Premier essai",
            "Avec aide",
            "Révélé",
            "Passé",
            "À vous",
            "Réponse adverse",
            "Hors solution ; réessayez",
            "Coup illégal",
            "Résolu",
            "Voir thèmes",
            "Mélange",
            "Inédits d’abord",
            "Revoir erreurs",
            "Progrès",
            "Durée",
            "Essais",
            "Résultats",
            "Traitement",
            "Détails / solution",
            "Indice utilisé",
            "Revoir solution",
            "Niveau du problème Lichess",
        ],
        Language::Spanish => [
            "Táctica",
            "Volver",
            "Siguiente",
            "Reintentar",
            "Pista",
            "Ver solución",
            "Anterior",
            "Siguiente",
            "Tema",
            "Nivel",
            "Modo",
            "Importar CSV",
            "Cancelar",
            "Sin problemas",
            "Primer intento",
            "Con ayuda",
            "Revelado",
            "Omitido",
            "Tu turno",
            "Respuesta rival",
            "Fuera de solución; reintenta",
            "Jugada ilegal",
            "Resuelto",
            "Ver temas",
            "Mixto",
            "Nuevos primero",
            "Repetir errores",
            "Progreso",
            "Tiempo",
            "Intentos",
            "Resultados",
            "Procesando",
            "Detalles / solución",
            "Pista usada",
            "Repetir solución",
            "Nivel del problema Lichess",
        ],
        Language::Latin => [
            "Exercitia",
            "Redire",
            "Proximum",
            "Iterare",
            "Indicium",
            "Solutio",
            "Ante",
            "Post",
            "Thema",
            "Gradus",
            "Modus",
            "Importare CSV",
            "Revocare",
            "Nullum exercitium",
            "Prima vice",
            "Auxilio",
            "Revelatum",
            "Omissum",
            "Tuus motus",
            "Responsum",
            "Non solutio; itera",
            "Motus illicitus",
            "Solutum",
            "Themata",
            "Mixta",
            "Nova primum",
            "Errores",
            "Progressus",
            "Tempus",
            "Conatus",
            "Exitus",
            "Laborat",
            "Notitia / solutio",
            "Indicium adhibitum",
            "Repetitio",
            "Gradus exercitii Lichess",
        ],
        Language::Japanese => [
            "戦術トレーニング",
            "戻る",
            "次の問題",
            "再挑戦",
            "ヒント",
            "解答表示",
            "前へ",
            "次へ",
            "テーマ",
            "難度",
            "モード",
            "CSV読込",
            "読込中止",
            "該当問題なし",
            "初回正解",
            "補助あり",
            "解答表示済",
            "スキップ",
            "あなたの手番",
            "相手の応手",
            "参考解答と異なります・再挑戦",
            "非合法手",
            "正解",
            "テーマ表示",
            "混合",
            "未挑戦優先",
            "誤答復習",
            "進捗",
            "練習時間",
            "試行",
            "成績",
            "処理中",
            "詳細 / 解答",
            "ヒント使用",
            "解答再生",
            "Lichess問題レート",
        ],
        Language::Korean => [
            "전술 훈련",
            "뒤로",
            "다음 문제",
            "재시도",
            "힌트",
            "정답 보기",
            "이전",
            "다음",
            "주제",
            "난도",
            "모드",
            "CSV 가져오기",
            "가져오기 취소",
            "일치 문제 없음",
            "첫 시도 정답",
            "도움 사용",
            "정답 공개",
            "건너뜀",
            "내 차례",
            "상대 응수",
            "참고 답과 다름; 재시도",
            "불법 수",
            "해결",
            "주제 표시",
            "혼합",
            "미시도 우선",
            "오답 복습",
            "진행",
            "훈련 시간",
            "시도",
            "결과",
            "처리 중",
            "상세 / 정답",
            "힌트 사용",
            "정답 재생",
            "Lichess 문제 레이팅",
        ],
        Language::German => [
            "Taktiktraining",
            "Zurück",
            "Nächste",
            "Wiederholen",
            "Hinweis",
            "Lösung zeigen",
            "Zurück",
            "Weiter",
            "Thema",
            "Stärke",
            "Modus",
            "CSV importieren",
            "Abbrechen",
            "Keine Aufgaben",
            "Erstversuch",
            "Mit Hilfe",
            "Aufgedeckt",
            "Übersprungen",
            "Dein Zug",
            "Gegenzug",
            "Nicht Referenz; erneut versuchen",
            "Illegaler Zug",
            "Gelöst",
            "Themen zeigen",
            "Gemischt",
            "Neue zuerst",
            "Fehler üben",
            "Fortschritt",
            "Trainingszeit",
            "Versuche",
            "Ergebnisse",
            "Verarbeitung",
            "Details / Lösung",
            "Hinweis genutzt",
            "Lösung abspielen",
            "Lichess Aufgabenwertung",
        ],
        Language::Italian => [
            "Tattica",
            "Indietro",
            "Successivo",
            "Riprova",
            "Indizio",
            "Mostra soluzione",
            "Indietro",
            "Avanti",
            "Tema",
            "Livello",
            "Modalità",
            "Importa CSV",
            "Annulla",
            "Nessun problema",
            "Primo tentativo",
            "Con aiuto",
            "Rivelato",
            "Saltato",
            "Tocca a te",
            "Risposta avversaria",
            "Diverso dalla soluzione; riprova",
            "Mossa illegale",
            "Risolto",
            "Mostra temi",
            "Misto",
            "Nuovi prima",
            "Ripassa errori",
            "Progresso",
            "Tempo",
            "Tentativi",
            "Risultati",
            "Elaborazione",
            "Dettagli / soluzione",
            "Indizio usato",
            "Rivedi soluzione",
            "Livello problema Lichess",
        ],
        Language::Russian => [
            "Тактика",
            "Назад",
            "Следующая",
            "Повторить",
            "Подсказка",
            "Ответ",
            "Назад",
            "Далее",
            "Тема",
            "Сложность",
            "Режим",
            "Импорт CSV",
            "Отмена",
            "Нет задач",
            "С первой попытки",
            "С помощью",
            "Ответ открыт",
            "Пропущено",
            "Ваш ход",
            "Ответ соперника",
            "Не эталонный ход; попробуйте ещё",
            "Нелегальный ход",
            "Решено",
            "Показать темы",
            "Все",
            "Новые сначала",
            "Повтор ошибок",
            "Прогресс",
            "Время",
            "Попытки",
            "Итоги",
            "Обработка",
            "Сведения / ответ",
            "Есть подсказка",
            "Просмотр ответа",
            "Рейтинг задачи Lichess",
        ],
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
    if app.screen == Screen::Menu {
        c.button(
            (390.0, h - 112.0, (w - 414.0).clamp(80.0, 230.0), 30.0),
            t[0],
            Action::Puzzle(A::Enter),
        );
        return c.actions;
    }
    if app.screen != Screen::Puzzle {
        return vec![];
    }
    let p = &app.puzzles;
    let x = w - 292.0;
    fill_rect(c.buf, size, x as i32, 0, 292, size.1 as i32, c.pal.panel);
    c.line(x + 10.0, 24.0, t[0], 270.0, c.pal.text);
    for (i, (label, a)) in [
        (
            format!(
                "{}: {}",
                t[8],
                if THEMES[p.theme].is_empty() {
                    t[24]
                } else {
                    THEMES[p.theme]
                }
            ),
            A::Theme,
        ),
        (
            format!(
                "{}: {}–{}",
                t[9], RANGES[p.difficulty].0, RANGES[p.difficulty].1
            ),
            A::Difficulty,
        ),
        (format!("{}: {}", t[10], t[24 + p.mode]), A::Mode),
        (
            format!("{} {}", if p.show_themes { "✓" } else { "□" }, t[23]),
            A::Themes,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        c.button(
            (x + 10.0, 34.0 + i as f32 * 32.0, 270.0, 28.0),
            &label,
            Action::Puzzle(a),
        );
    }
    c.line(
        x + 10.0,
        180.0,
        &format!("{} {}", t[27], p.bank.len()),
        270.0,
        c.pal.text,
    );
    if let Some(s) = &p.session {
        c.line(
            x + 10.0,
            202.0,
            &format!("#{} · {} {}", s.puzzle.data.id, t[9], s.puzzle.data.rating),
            270.0,
            c.pal.text,
        );
        let status = if let Some(outcome) = s.outcome {
            t[match outcome {
                Outcome::FirstTry => 14,
                Outcome::Assisted => 15,
                Outcome::Revealed => 16,
                Outcome::Skipped => 17,
            }]
        } else {
            t[if s.state == State::Responding { 19 } else { 18 }]
        };
        c.line(
            x + 10.0,
            224.0,
            &format!(
                "{} {}/{} {}",
                status,
                s.next,
                s.puzzle.moves.len(),
                if s.hints > 0 { t[33] } else { "" }
            ),
            270.0,
            c.pal.text,
        );
        c.line(
            x + 10.0,
            246.0,
            &format!(
                "{} · {}s",
                if s.solver() == shakmaty::Color::White {
                    app.settings.language.text().white
                } else {
                    app.settings.language.text().black
                },
                if s.outcome.is_some() {
                    s.elapsed.as_secs()
                } else {
                    s.started.elapsed().as_secs()
                }
            ),
            270.0,
            c.pal.text,
        );
        if p.show_themes {
            c.line(
                x + 10.0,
                268.0,
                &s.puzzle.data.themes.join(" · "),
                270.0,
                c.pal.text,
            );
        }
        let line = if s.state == State::Answer {
            format!(
                "{} {}/{}: {}",
                t[34],
                s.answer_cursor,
                s.puzzle.moves.len(),
                s.puzzle.sans[s.answer_cursor - 1]
            )
        } else {
            p.notice.clone().unwrap_or_default()
        };
        c.line(12.0, h - 52.0, &line, x - 20.0, c.pal.accent);
    } else {
        c.line(x + 10.0, 224.0, t[13], 270.0, c.pal.text);
    }
    for (i, (label, a)) in [
        (t[4], A::Hint),
        (t[5], A::Reveal),
        (t[3], A::Retry),
        (t[2], A::Next),
        (t[6], A::Previous),
        (t[7], A::Forward),
        (
            if p.busy() { t[12] } else { t[11] },
            if p.busy() { A::Cancel } else { A::Import },
        ),
        (t[32], A::Details),
    ]
    .into_iter()
    .enumerate()
    {
        c.button(
            (
                x + 10.0 + (i % 2) as f32 * 138.0,
                282.0 + (i / 2) as f32 * 32.0,
                132.0,
                28.0,
            ),
            label,
            Action::Puzzle(a),
        );
    }
    let [attempts, first, assisted, revealed, skipped] = p.totals;
    c.line(
        x + 10.0,
        426.0,
        &format!(
            "{} {} · {} {:.0}%",
            t[29],
            attempts,
            t[14],
            if attempts > 0 {
                100.0 * first as f64 / attempts as f64
            } else {
                0.0
            }
        ),
        270.0,
        c.pal.text,
    );
    c.line(
        x + 10.0,
        447.0,
        &format!(
            "✓{} +{} ×{} →{} · #{} / ✓#{}",
            first,
            assisted,
            revealed,
            skipped,
            p.progress.entries.len(),
            p.completed
        ),
        270.0,
        c.pal.text,
    );
    c.line(
        x + 10.0,
        469.0,
        &format!("{} {}s", t[28], p.started.elapsed().as_secs()),
        270.0,
        c.pal.text,
    );
    if p.busy() {
        c.line(
            12.0,
            24.0,
            &format!(
                "{} {}",
                t[31],
                p.work.load(std::sync::atomic::Ordering::Relaxed)
            ),
            x - 20.0,
            c.pal.text,
        );
    } else if p.error.is_some() {
        c.line(12.0, 24.0, &format!("! {}", t[32]), x - 20.0, c.pal.accent);
    }
    c.button(
        (12.0, h - 38.0, (x - 34.0).max(80.0), 28.0),
        t[1],
        Action::Puzzle(A::Back),
    );
    c.actions
}
