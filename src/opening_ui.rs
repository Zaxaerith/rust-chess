//! Explorer-only controls. The board is drawn by the existing Renderer.
use crate::{
    App, Screen,
    assets::fill_rect,
    font::TextRenderer,
    game::position_key,
    i18n::Language,
    opening_app::{Action as A, Status},
    opening_stats::Counts,
    records_ui::{Action, Canvas},
};
pub fn words(lang: Language) -> [&'static str; 30] {
    match lang {
        Language::Chinese => [
            "开局浏览器",
            "未识别",
            "后续着法",
            "开局搜索",
            "关联棋局",
            "构建统计",
            "取消统计",
            "返回档案馆",
            "初始局面",
            "上一着",
            "下一着",
            "馆藏无样本",
            "总样本",
            "未完成",
            "有结果 W/D/L",
            "非标准 FEN",
            "参考路线，无比赛样本",
            "候选名称",
            "搜索 ECO / 名称",
            "清空",
            "空闲",
            "加载中",
            "统计中",
            "取消中",
            "完成",
            "已取消",
            "失败",
            "局面分类；W/D/L 为馆藏历史，并非引擎预测胜率",
            "已过期，请重建",
            "详情",
        ],
        Language::TraditionalChinese => [
            "開局瀏覽器",
            "未識別",
            "後續著法",
            "開局搜尋",
            "關聯棋局",
            "建立統計",
            "取消統計",
            "返回檔案館",
            "初始局面",
            "上一著",
            "下一著",
            "館藏無樣本",
            "總樣本",
            "未完成",
            "有結果 W/D/L",
            "非標準 FEN",
            "參考路線，無比賽樣本",
            "候選名稱",
            "搜尋 ECO / 名稱",
            "清空",
            "閒置",
            "載入中",
            "統計中",
            "取消中",
            "完成",
            "已取消",
            "失敗",
            "局面分類；W/D/L 為館藏歷史，非引擎預測勝率",
            "已過期，請重建",
            "詳情",
        ],
        Language::English => [
            "Opening Explorer",
            "Unknown",
            "Next moves",
            "Find opening",
            "Related games",
            "Build statistics",
            "Cancel build",
            "Back to Library",
            "Start position",
            "Previous",
            "Next",
            "No library samples",
            "Total games",
            "Unfinished",
            "Decided W/D/L",
            "Nonstandard FEN",
            "Reference only; no samples",
            "Candidate names",
            "Search ECO / name",
            "Clear",
            "Idle",
            "Loading",
            "Building",
            "Cancelling",
            "Ready",
            "Cancelled",
            "Failed",
            "Position classification; W/D/L is library history, not engine prediction",
            "Stale; rebuild",
            "Details",
        ],
        Language::French => [
            "Explorateur",
            "Inconnue",
            "Coups suivants",
            "Ouvertures",
            "Parties liées",
            "Construire",
            "Annuler",
            "Bibliothèque",
            "Position initiale",
            "Précédent",
            "Suivant",
            "Aucun échantillon local",
            "Total parties",
            "Inachevées",
            "Résolues G/N/P",
            "FEN spéciale",
            "Référence sans parties",
            "Noms possibles",
            "ECO / nom",
            "Effacer",
            "Inactif",
            "Chargement",
            "Construction",
            "Annulation",
            "Prêt",
            "Annulé",
            "Échec",
            "Classement par position ; résultats locaux, pas une prédiction",
            "Périmé : reconstruire",
            "Détails",
        ],
        Language::Spanish => [
            "Explorador",
            "Desconocida",
            "Jugadas",
            "Buscar apertura",
            "Partidas",
            "Crear estadísticas",
            "Cancelar",
            "Biblioteca",
            "Posición inicial",
            "Anterior",
            "Siguiente",
            "Sin muestras locales",
            "Total partidas",
            "Sin terminar",
            "Finalizadas G/T/P",
            "FEN especial",
            "Solo referencia",
            "Nombres posibles",
            "ECO / nombre",
            "Limpiar",
            "Inactivo",
            "Cargando",
            "Calculando",
            "Cancelando",
            "Listo",
            "Cancelado",
            "Error",
            "Clasificación por posición; historial local, no predicción",
            "Obsoleto: reconstruir",
            "Detalles",
        ],
        Language::Latin => [
            "Explorator",
            "Ignota",
            "Motus sequentes",
            "Quaere initium",
            "Ludi conexi",
            "Computa",
            "Revoca",
            "Bibliotheca",
            "Initium",
            "Prior",
            "Proximus",
            "Nulla exempla localia",
            "Ludi omnes",
            "Infecti",
            "Finita V/A/P",
            "FEN singularis",
            "Exemplum tantum",
            "Nomina possibilia",
            "ECO / nomen",
            "Dele",
            "Iners",
            "Legitur",
            "Computatur",
            "Revocatur",
            "Paratum",
            "Revocatum",
            "Error",
            "Classificatio loci; eventus locales, non praedictio",
            "Obsoletum: computa",
            "Singula",
        ],
        Language::Japanese => [
            "オープニング",
            "不明",
            "次の手",
            "定跡検索",
            "関連棋譜",
            "統計を構築",
            "取消",
            "ライブラリへ",
            "初期局面",
            "前へ",
            "次へ",
            "館内の標本なし",
            "総対局数",
            "未完了",
            "結果あり W/D/L",
            "特殊 FEN",
            "参考手順のみ",
            "候補名",
            "ECO / 名前を検索",
            "消去",
            "待機",
            "読込中",
            "集計中",
            "取消中",
            "完了",
            "取消済み",
            "失敗",
            "局面分類。W/D/Lは館内履歴で予測勝率ではありません",
            "更新が必要",
            "詳細",
        ],
        Language::Korean => [
            "오프닝 탐색",
            "알 수 없음",
            "다음 수",
            "오프닝 검색",
            "관련 대국",
            "통계 만들기",
            "취소",
            "보관함으로",
            "초기 국면",
            "이전",
            "다음",
            "보관함 표본 없음",
            "총 대국",
            "미완료",
            "결과 W/D/L",
            "특수 FEN",
            "참고 수순만",
            "후보 이름",
            "ECO / 이름 검색",
            "지우기",
            "대기",
            "읽는 중",
            "집계 중",
            "취소 중",
            "완료",
            "취소됨",
            "실패",
            "국면 분류; W/D/L은 보관함 기록이며 예측 승률이 아닙니다",
            "오래됨: 재구축",
            "상세",
        ],
        Language::German => [
            "Eröffnungen",
            "Unbekannt",
            "Folgezüge",
            "Eröffnung suchen",
            "Partien",
            "Statistik erstellen",
            "Abbrechen",
            "Bibliothek",
            "Startposition",
            "Zurück",
            "Weiter",
            "Keine lokalen Partien",
            "Partien gesamt",
            "Unbeendet",
            "Beendet S/R/N",
            "Sonder-FEN",
            "Nur Referenz",
            "Mögliche Namen",
            "ECO / Name suchen",
            "Leeren",
            "Inaktiv",
            "Laden",
            "Berechnen",
            "Abbruch",
            "Fertig",
            "Abgebrochen",
            "Fehler",
            "Stellungsklasse; lokale Ergebnisse, keine Engine-Prognose",
            "Veraltet: neu erstellen",
            "Details",
        ],
        Language::Italian => [
            "Esplora aperture",
            "Sconosciuta",
            "Mosse seguenti",
            "Cerca apertura",
            "Partite",
            "Crea statistiche",
            "Annulla",
            "Archivio",
            "Posizione iniziale",
            "Precedente",
            "Successivo",
            "Nessun campione locale",
            "Partite totali",
            "Incompiute",
            "Concluse V/P/S",
            "FEN speciale",
            "Solo riferimento",
            "Nomi possibili",
            "Cerca ECO / nome",
            "Cancella",
            "Inattivo",
            "Caricamento",
            "Calcolo",
            "Annullamento",
            "Pronto",
            "Annullato",
            "Errore",
            "Classificazione per posizione; storia locale, non previsione",
            "Obsoleto: ricostruire",
            "Dettagli",
        ],
        Language::Russian => [
            "Дебюты",
            "Неизвестно",
            "Следующие ходы",
            "Поиск дебюта",
            "Партии",
            "Построить",
            "Отмена",
            "Библиотека",
            "Начальная позиция",
            "Назад",
            "Вперёд",
            "Нет местных партий",
            "Всего партий",
            "Неоконченные",
            "С результатом В/Н/П",
            "Особая FEN",
            "Только справка",
            "Варианты названий",
            "Поиск ECO / имени",
            "Очистить",
            "Ожидание",
            "Загрузка",
            "Построение",
            "Отмена",
            "Готово",
            "Отменено",
            "Ошибка",
            "Классификация позиции; местная история, не прогноз движка",
            "Устарело: перестроить",
            "Сведения",
        ],
    }
}
fn short(font: &TextRenderer, text: &str, width: f32) -> String {
    let mut out = String::new();
    for c in text.chars().take(1000) {
        if font.text_width(&format!("{out}{c}…"), 14.0) > width {
            out.push('…');
            break;
        }
        out.push(c);
    }
    out
}
fn counts(c: &Counts) -> String {
    format!("{}/{}/{} · *{}", c.white, c.draws, c.black, c.unfinished)
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
            (390.0, h - 40.0, (w - 414.0).clamp(80.0, 230.0), 30.0),
            t[0],
            Action::Opening(A::Enter),
        );
        return c.actions;
    }
    if app.screen == Screen::Library {
        c.button(
            (w - 238.0, 6.0, 222.0, 30.0),
            t[0],
            Action::Opening(A::Enter),
        );
        return c.actions;
    }
    if app.screen != Screen::Opening {
        return vec![];
    }
    let o = &app.opening;
    let x = w - 292.0;
    fill_rect(c.buf, size, x as i32, 0, 292, size.1 as i32, c.pal.panel);
    let title = if let (Some(b), Some(m)) = (&o.book, &o.named) {
        let e = &b.entries[m.primary];
        format!("{} {} [{} {}]", e.eco, e.name, t[17], m.candidates)
    } else {
        t[1].into()
    };
    c.line(
        12.0,
        24.0,
        &short(font, &title, x - 20.0),
        x - 20.0,
        c.pal.text,
    );
    c.line(
        12.0,
        h - 53.0,
        &short(font, &format!("{}: {}", o.cursor, o.path), x - 20.0),
        x - 20.0,
        c.pal.text,
    );
    let col = (x - 40.0) / 3.0;
    for (i, (label, a)) in [(t[9], A::Previous), (t[10], A::Next), (t[8], A::First)]
        .into_iter()
        .enumerate()
    {
        c.button(
            (12.0 + i as f32 * (col + 8.0), h - 38.0, col, 28.0),
            label,
            Action::Opening(a),
        );
    }
    let state = t[match o.status {
        Status::Idle => 20,
        Status::Loading => 21,
        Status::Running => 22,
        Status::Cancelling => 23,
        Status::Ready => 24,
        Status::Cancelled => 25,
        Status::Failed => 26,
        Status::Stale => 28,
    }];
    c.line(
        x + 10.0,
        24.0,
        &short(
            font,
            &format!(
                "{} {}/{}{}",
                state,
                o.progress.load(std::sync::atomic::Ordering::Relaxed),
                o.total,
                if o.cached { " ✓" } else { "" }
            ),
            270.0,
        ),
        270.0,
        c.pal.text,
    );
    c.button(
        (x + 10.0, 34.0, 126.0, 28.0),
        if o.busy() { t[6] } else { t[5] },
        Action::Opening(if o.busy() { A::Cancel } else { A::Build }),
    );
    c.button(
        (x + 144.0, 34.0, 136.0, 28.0),
        &format!("{}{}", if o.error.is_some() { "! " } else { "" }, t[29]),
        Action::Opening(A::Info),
    );
    let node = o
        .current_stats()
        .and_then(|s| s.nodes.get(&position_key(o.position())));
    if let Some(n) = node {
        c.line(
            x + 10.0,
            82.0,
            &format!(
                "{} {} · {} {}",
                t[12],
                n.counts.total(),
                t[13],
                n.counts.unfinished
            ),
            270.0,
            c.pal.text,
        );
        let perc = n
            .counts
            .percentages()
            .map(|p| {
                format!(
                    "{:.0}/{:.0}/{:.0}% (n={})",
                    p[0],
                    p[1],
                    p[2],
                    n.counts.decided()
                )
            })
            .unwrap_or_else(|| "— (n=0)".into());
        c.line(
            x + 10.0,
            102.0,
            &short(font, &format!("{} {}", t[14], perc), 270.0),
            270.0,
            c.pal.text,
        );
        c.line(
            x + 10.0,
            122.0,
            &short(
                font,
                &format!(
                    "W/D/L {} · {} {}",
                    counts(&n.counts),
                    t[15],
                    n.counts.nonstandard
                ),
                270.0,
            ),
            270.0,
            c.pal.text,
        );
    } else {
        c.line(x + 10.0, 94.0, t[11], 270.0, c.pal.text);
    }
    for (i, (label, a)) in [(t[2], A::Moves), (t[3], A::Search), (t[4], A::Games)]
        .into_iter()
        .enumerate()
    {
        c.button(
            (x + 10.0 + i as f32 * 91.0, 134.0, 86.0, 28.0),
            label,
            Action::Opening(a),
        );
    }
    if o.tab == 1 {
        c.button(
            (x + 10.0, 170.0, 224.0, 28.0),
            &short(
                font,
                if o.query.is_empty() { t[18] } else { &o.query },
                206.0,
            ),
            Action::Opening(A::Focus),
        );
        c.button(
            (x + 240.0, 170.0, 40.0, 28.0),
            "×",
            Action::Opening(A::Clear),
        );
    } else {
        c.line(
            x + 10.0,
            190.0,
            &short(font, if let Some(e) = &o.error { e } else { t[27] }, 270.0),
            270.0,
            c.pal.accent,
        );
    }
    let rows = ((h - 302.0) / 50.0).max(1.0) as usize;
    let next_total = node
        .map(|n| n.moves.values().map(Counts::total).sum::<u32>())
        .unwrap_or(0);
    for (row, i) in (o.scroll..app.opening_list_len()).take(rows).enumerate() {
        let y = 208.0 + row as f32 * 50.0;
        let (a, l1, l2) = match o.tab {
            0 => {
                let m = &o.choices[i];
                let first = if let Some(n) = &m.counts {
                    format!(
                        "{} · {} / {} ({:.1}%)",
                        m.san,
                        n.total(),
                        next_total,
                        n.total() as f64 * 100.0 / next_total.max(1) as f64
                    )
                } else {
                    m.san.clone()
                };
                (
                    A::Choose(i),
                    first,
                    m.counts
                        .as_ref()
                        .map(counts)
                        .unwrap_or_else(|| t[16].into()),
                )
            }
            1 => {
                let id = o.found[i];
                let e = &o.book.as_ref().expect("reference book").entries[id];
                (
                    A::Reference(id),
                    format!("{} {}", e.eco, e.name),
                    t[16].into(),
                )
            }
            _ => {
                let (id, ply) = &o.related[i];
                let entry = app.library.store.as_ref().and_then(|l| l.entry(id).ok());
                let title = entry
                    .map(|e| format!("{} vs {}", e.white, e.black))
                    .unwrap_or_else(|| id.clone());
                let eco = o
                    .current_stats()
                    .and_then(|s| s.classified.get(id))
                    .and_then(|v| *v)
                    .and_then(|id| o.book.as_ref()?.entries.get(id))
                    .map(|e| e.eco.as_str())
                    .unwrap_or(t[1]);
                (A::Game(i), title, format!("#{ply} {eco}"))
            }
        };
        c.button((x + 10.0, y, 244.0, 46.0), "", Action::Opening(a));
        c.line(
            x + 16.0,
            y + 19.0,
            &short(font, &l1, 228.0),
            228.0,
            c.pal.text,
        );
        c.line(
            x + 16.0,
            y + 39.0,
            &short(font, &l2, 228.0),
            228.0,
            c.pal.text,
        );
    }
    c.button((x + 260.0, 208.0, 20.0, 28.0), "↑", Action::Opening(A::Up));
    c.button(
        (x + 260.0, 244.0, 20.0, 28.0),
        "↓",
        Action::Opening(A::Down),
    );
    c.button(
        (x + 10.0, h - 42.0, 270.0, 30.0),
        t[7],
        Action::Opening(A::Back),
    );
    c.actions
}
