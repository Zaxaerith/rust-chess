//! Full-page library: bounded visible rows, metadata only. Rendering never reads PGNs.
use crate::{
    App, Screen,
    assets::{fill_rect, fill_rect_alpha},
    font::TextRenderer,
    i18n::Language,
    library::Sort,
    library_app::{Action as A, RESULTS},
    records_ui::{Action, Canvas},
};
pub fn words(lang: Language) -> [&'static str; 30] {
    match lang {
        Language::Chinese => [
            "棋谱档案馆",
            "搜索：棋手/赛事/日期/结果",
            "导入 PGN",
            "打开复盘",
            "收藏/取消",
            "仅收藏",
            "结果",
            "排序",
            "最近导入",
            "比赛日期",
            "棋手姓名",
            "收藏优先",
            "导出副本",
            "移除",
            "返回",
            "尚无匹配棋局，请导入 PGN",
            "移除馆藏副本？原文件不受影响",
            "确认移除",
            "已保存到馆藏",
            "已导入",
            "重复跳过",
            "失败",
            "保存到馆藏",
            "加入档案馆",
            "清空",
            "检查/恢复",
            "导入中",
            "取消",
            "馆藏副本",
            "半回合",
        ],
        Language::TraditionalChinese => [
            "棋譜檔案館",
            "搜尋：棋手/賽事/日期/結果",
            "匯入 PGN",
            "開啟複盤",
            "收藏/取消",
            "僅收藏",
            "結果",
            "排序",
            "最近匯入",
            "比賽日期",
            "棋手姓名",
            "收藏優先",
            "匯出副本",
            "移除",
            "返回",
            "沒有符合棋局，請匯入 PGN",
            "移除館藏副本？原檔案不受影響",
            "確認移除",
            "已儲存至館藏",
            "已匯入",
            "略過重複",
            "失敗",
            "儲存至館藏",
            "加入檔案館",
            "清空",
            "檢查/復原",
            "匯入中",
            "取消",
            "館藏副本",
            "半回合",
        ],
        Language::English => [
            "PGN Library",
            "Search: players/event/date/result",
            "Import PGN",
            "Open Replay",
            "Toggle favorite",
            "Favorites only",
            "Result",
            "Sort",
            "Recently imported",
            "Game date",
            "Players",
            "Favorites first",
            "Export copy",
            "Remove",
            "Back",
            "No matching games. Import PGN to begin.",
            "Remove library copy? Source files are kept.",
            "Confirm removal",
            "Saved to Library",
            "Imported",
            "Duplicates skipped",
            "Failed",
            "Save to Library",
            "Add to Library",
            "Clear",
            "Check/recover",
            "Importing",
            "Cancel",
            "Library copy",
            "plies",
        ],
        Language::French => [
            "Bibliothèque PGN",
            "Rechercher : joueurs/événement/date/résultat",
            "Importer PGN",
            "Ouvrir",
            "Favori oui/non",
            "Favoris seuls",
            "Résultat",
            "Tri",
            "Import récent",
            "Date de partie",
            "Joueurs",
            "Favoris en tête",
            "Exporter copie",
            "Retirer",
            "Retour",
            "Aucune partie. Importez un PGN.",
            "Retirer la copie ? La source est conservée.",
            "Confirmer",
            "Enregistré",
            "Importées",
            "Doublons ignorés",
            "Échecs",
            "Enregistrer ici",
            "Ajouter ici",
            "Effacer",
            "Vérifier/récupérer",
            "Importation",
            "Annuler",
            "Copie locale",
            "demi-coups",
        ],
        Language::Spanish => [
            "Biblioteca PGN",
            "Buscar: jugadores/evento/fecha/resultado",
            "Importar PGN",
            "Abrir",
            "Cambiar favorito",
            "Solo favoritos",
            "Resultado",
            "Orden",
            "Importación reciente",
            "Fecha de partida",
            "Jugadores",
            "Favoritos primero",
            "Exportar copia",
            "Quitar",
            "Volver",
            "Sin partidas. Importe PGN.",
            "¿Quitar copia? Se conserva el original.",
            "Confirmar",
            "Guardado",
            "Importadas",
            "Duplicados omitidos",
            "Fallos",
            "Guardar aquí",
            "Añadir aquí",
            "Limpiar",
            "Comprobar/recuperar",
            "Importando",
            "Cancelar",
            "Copia local",
            "medias jugadas",
        ],
        Language::Latin => [
            "Bibliotheca PGN",
            "Quaere: lusores/eventus/dies/exitus",
            "Importa PGN",
            "Aperi",
            "Favorem muta",
            "Favorita tantum",
            "Exitus",
            "Ordo",
            "Nuper importata",
            "Dies ludi",
            "Lusores",
            "Favorita prima",
            "Exporta exemplum",
            "Remove",
            "Redi",
            "Nulli ludi. Importa PGN.",
            "Remove exemplum? Fons servatur.",
            "Confirma",
            "Servatum",
            "Importata",
            "Duplicata omissa",
            "Errores",
            "Serva hic",
            "Adde hic",
            "Dele",
            "Examina/recupera",
            "Importatur",
            "Revoca",
            "Exemplum locale",
            "semimotus",
        ],
        Language::Japanese => [
            "棋譜ライブラリ",
            "検索：棋手/大会/日付/結果",
            "PGN取込",
            "棋譜を開く",
            "お気に入り切替",
            "お気に入りのみ",
            "結果",
            "並べ替え",
            "取込順",
            "対局日",
            "棋手名",
            "お気に入り優先",
            "コピー出力",
            "削除",
            "戻る",
            "棋譜がありません。PGNを取り込んでください",
            "保存コピーを削除？元ファイルは保持します",
            "削除確認",
            "保存済み",
            "取込済み",
            "重複をスキップ",
            "失敗",
            "館内へ保存",
            "ライブラリへ追加",
            "消去",
            "確認/復旧",
            "取込中",
            "取消",
            "保存コピー",
            "半手",
        ],
        Language::Korean => [
            "PGN 보관함",
            "검색: 선수/대회/날짜/결과",
            "PGN 가져오기",
            "복기 열기",
            "즐겨찾기 전환",
            "즐겨찾기만",
            "결과",
            "정렬",
            "최근 가져오기",
            "대국 날짜",
            "선수",
            "즐겨찾기 우선",
            "사본 내보내기",
            "제거",
            "뒤로",
            "기보가 없습니다. PGN을 가져오세요",
            "보관 사본을 제거할까요? 원본은 유지됩니다",
            "제거 확인",
            "저장됨",
            "가져옴",
            "중복 건너뜀",
            "실패",
            "보관함에 저장",
            "보관함에 추가",
            "지우기",
            "확인/복구",
            "가져오는 중",
            "취소",
            "보관 사본",
            "반수",
        ],
        Language::German => [
            "PGN-Bibliothek",
            "Suche: Spieler/Turnier/Datum/Ergebnis",
            "PGN importieren",
            "Öffnen",
            "Favorit umschalten",
            "Nur Favoriten",
            "Ergebnis",
            "Sortieren",
            "Zuletzt importiert",
            "Partiedatum",
            "Spieler",
            "Favoriten zuerst",
            "Kopie exportieren",
            "Entfernen",
            "Zurück",
            "Keine Partien. PGN importieren.",
            "Bibliothekskopie entfernen? Quelle bleibt.",
            "Bestätigen",
            "Gespeichert",
            "Importiert",
            "Duplikate übersprungen",
            "Fehler",
            "Hier speichern",
            "Hier hinzufügen",
            "Leeren",
            "Prüfen/wiederherstellen",
            "Import läuft",
            "Abbrechen",
            "Lokale Kopie",
            "Halbzüge",
        ],
        Language::Italian => [
            "Archivio PGN",
            "Cerca: giocatori/evento/data/risultato",
            "Importa PGN",
            "Apri",
            "Cambia preferito",
            "Solo preferiti",
            "Risultato",
            "Ordine",
            "Importate di recente",
            "Data partita",
            "Giocatori",
            "Preferiti prima",
            "Esporta copia",
            "Rimuovi",
            "Indietro",
            "Nessuna partita. Importa PGN.",
            "Rimuovere copia? Il file originale resta.",
            "Conferma",
            "Salvato",
            "Importate",
            "Duplicati saltati",
            "Errori",
            "Salva qui",
            "Aggiungi qui",
            "Cancella",
            "Controlla/recupera",
            "Importazione",
            "Annulla",
            "Copia locale",
            "semimosse",
        ],
        Language::Russian => [
            "Библиотека PGN",
            "Поиск: игроки/турнир/дата/результат",
            "Импорт PGN",
            "Открыть",
            "Избранное вкл/выкл",
            "Только избранное",
            "Результат",
            "Порядок",
            "Недавно добавлены",
            "Дата партии",
            "Игроки",
            "Избранное первым",
            "Экспорт копии",
            "Удалить",
            "Назад",
            "Партий нет. Импортируйте PGN.",
            "Удалить копию? Исходный файл сохранится.",
            "Подтвердить",
            "Сохранено",
            "Добавлено",
            "Дубликаты пропущены",
            "Ошибки",
            "Сохранить здесь",
            "Добавить сюда",
            "Очистить",
            "Проверка/восстановление",
            "Импорт",
            "Отмена",
            "Локальная копия",
            "полуходов",
        ],
    }
}
fn short(font: &TextRenderer, text: &str, width: f32) -> String {
    let mut out = String::new();
    for c in text.chars().take(1024) {
        if font.text_width(&format!("{out}{c}…"), 16.0) > width {
            out.push('…');
            break;
        }
        out.push(c);
    }
    out
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
            (390.0, h - 76.0, (w - 414.0).clamp(80.0, 230.0), 30.0),
            t[0],
            Action::Library(A::Enter),
        );
        return c.actions;
    }
    if app.screen != Screen::Library {
        return vec![];
    }
    fill_rect(c.buf, size, 0, 0, size.0 as i32, size.1 as i32, c.pal.bg);
    let s = &app.library;
    c.line(
        16.0,
        30.0,
        &format!("{} ({})", t[0], s.visible.len()),
        (w - 500.0).clamp(150.0, 700.0),
        c.pal.accent,
    );
    let query = if s.query.is_empty() {
        t[1].into()
    } else {
        short(font, &s.query, w - 150.0)
    };
    c.button(
        (16.0, 44.0, w - 126.0, 32.0),
        &format!("{}{}", if s.focus { "> " } else { "" }, query),
        Action::Library(A::Search),
    );
    c.button(
        (w - 102.0, 44.0, 86.0, 32.0),
        t[24],
        Action::Library(A::Clear),
    );
    let col = (w - 56.0) / 4.0;
    for (i, (label, action)) in [
        (
            format!("{} {}", if s.favorites { "★" } else { "☆" }, t[5]),
            A::Favorites,
        ),
        (format!("{} {}", t[6], RESULTS[s.result]), A::Result),
        (
            format!(
                "{}: {}",
                t[7],
                t[match s.sort {
                    Sort::Recent => 8,
                    Sort::Date => 9,
                    Sort::Players => 10,
                    Sort::Favorite => 11,
                }]
            ),
            A::Sort,
        ),
        (t[25].into(), A::Check),
    ]
    .into_iter()
    .enumerate()
    {
        c.button(
            (16.0 + i as f32 * (col + 8.0), 84.0, col, 30.0),
            &label,
            Action::Library(action),
        );
    }
    let status = if s.busy() {
        format!(
            "{} {} / ≤200",
            t[26],
            s.progress.load(std::sync::atomic::Ordering::Relaxed)
        )
    } else {
        s.notice.clone().unwrap_or_default()
    };
    c.line(
        16.0,
        137.0,
        &short(font, &status, w - 80.0),
        w - 80.0,
        c.pal.text,
    );
    if let Some(e) = &s.error {
        c.line(
            16.0,
            158.0,
            &short(font, e, w - 32.0),
            w - 32.0,
            c.pal.accent,
        );
    }
    let rows = ((h - 300.0) / 54.0).max(1.0) as usize;
    if s.visible.is_empty() {
        c.line(16.0, 196.0, t[15], w - 32.0, c.pal.text);
    }
    if let Some(store) = &s.store {
        for (row, i) in (s.scroll..s.visible.len()).take(rows).enumerate() {
            let Ok(e) = store.entry(&s.visible[i]) else {
                continue;
            };
            let y = 174.0 + row as f32 * 54.0;
            c.button((16.0, y, w - 72.0, 50.0), "", Action::Library(A::Select(i)));
            let line = format!(
                "{} {}{} vs {} · {}",
                if s.selected.as_ref() == Some(&e.id) {
                    ">"
                } else {
                    ""
                },
                if e.favorite { "★ " } else { "" },
                e.white,
                e.black,
                e.result
            );
            c.line(
                24.0,
                y + 20.0,
                &short(font, &line, w - 94.0),
                w - 94.0,
                c.pal.text,
            );
            c.line(
                24.0,
                y + 41.0,
                &short(
                    font,
                    &format!("{} · {} · {} {}", e.date, e.event, e.plies, t[29]),
                    w - 94.0,
                ),
                w - 94.0,
                c.pal.text,
            );
        }
    }
    c.button((w - 48.0, 174.0, 32.0, 28.0), "↑", Action::Library(A::Up));
    c.button((w - 48.0, 210.0, 32.0, 28.0), "↓", Action::Library(A::Down));
    let col = (w - 48.0) / 3.0;
    for (i, (index, a)) in [
        (2, A::Import),
        (3, A::Open),
        (4, A::Favorite),
        (12, A::Export),
        (13, A::Remove),
        (14, A::Back),
    ]
    .into_iter()
    .enumerate()
    {
        c.button(
            (
                16.0 + (i % 3) as f32 * (col + 8.0),
                h - 92.0 + (i / 3) as f32 * 40.0,
                col,
                32.0,
            ),
            t[index],
            Action::Library(a),
        );
    }
    if s.error.is_some() {
        c.button(
            (w - 70.0, 120.0, 54.0, 38.0),
            "! …",
            Action::Library(A::Details),
        );
    }
    if s.busy() {
        c.actions.clear();
    }
    if s.remove.is_some() {
        c.actions.clear();
        fill_rect_alpha(c.buf, size, (0, 0), size.0 as i32, size.1 as i32, 0, 190);
        let bw = (w - 48.0).min(600.0);
        let x = (w - bw) / 2.0;
        let y = h / 2.0 - 80.0;
        fill_rect(c.buf, size, x as i32, y as i32, bw as i32, 160, c.pal.panel);
        c.wrapped(x + 16.0, y + 35.0, t[16], bw - 32.0, 3);
        c.button(
            (x + 16.0, y + 110.0, (bw - 48.0) / 2.0, 32.0),
            t[17],
            Action::Library(A::ConfirmRemove),
        );
        c.button(
            (x + bw / 2.0 + 8.0, y + 110.0, (bw - 48.0) / 2.0, 32.0),
            t[27],
            Action::Library(A::CancelRemove),
        );
    }
    c.actions
}
