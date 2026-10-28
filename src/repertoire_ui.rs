//! Repertoire controls use the existing pixel canvas. No window or engine ownership.
use crate::{
    App, Screen,
    assets::{fill_rect, fill_rect_alpha},
    font::TextRenderer,
    i18n::Language,
    records_ui::{Action, Canvas},
    repertoire_app::{Action as A, Confirm, Page},
};

/// 0..56: controls; 57..75: training feedback/statistics/progress guard. User text and ECO names stay unchanged.
pub fn words(lang: Language) -> [&'static str; 76] {
    match lang {
        Language::Chinese => [
            "个人开局库",
            "新建白方库",
            "新建黑方库",
            "修改名称",
            "编辑变化",
            "开始训练",
            "删除库",
            "返回",
            "变化路线",
            "尚无开局库",
            "训练方",
            "保存路线",
            "创建分支",
            "删除路线",
            "初始局面",
            "上一步",
            "下一步",
            "提示",
            "查看准备路线",
            "重试",
            "下一项",
            "训练模式",
            "今日待复习",
            "新路线",
            "自由练习",
            "错误重练",
            "导出 JSON",
            "导入 JSON",
            "新建独立库",
            "合并到所选库",
            "取消",
            "应用",
            "开局库说明",
            "路线名称",
            "路线注释",
            "尚未保存",
            "修改尚未保存，是否离开？",
            "放弃修改",
            "继续编辑",
            "确认删除选中项目？",
            "请选择目标开局库",
            "加入所选开局库",
            "导入半回合",
            "增加",
            "减少",
            "未识别",
            "学习统计",
            "详情",
            "输入名称或说明",
            "个人准备，并非引擎推荐",
            "自己的准备着法",
            "预设对手回应",
            "今日已完成",
            "复习已保存",
            "仅导入实际主线开局",
            "说明",
            "路线结束",
            "等待你的准备着法",
            "预设路线推进",
            "本轮结束",
            "今日无待训练节点",
            "记忆正确",
            "不属于当前准备路线，并非棋理差着",
            "非法着法",
            "已使用提示",
            "已显示准备路线",
            "已学习",
            "未学习",
            "待复习",
            "今日已复习",
            "近期记忆正确率",
            "错误次数",
            "提示次数",
            "重试保存进度",
            "放弃未保存进度",
            "训练结果尚未保存",
        ],
        Language::TraditionalChinese => [
            "個人開局庫",
            "新增白方庫",
            "新增黑方庫",
            "修改名稱",
            "編輯變化",
            "開始訓練",
            "刪除開局庫",
            "返回",
            "變化路線",
            "尚無開局庫",
            "訓練方",
            "儲存路線",
            "建立分支",
            "刪除路線",
            "初始局面",
            "上一步",
            "下一步",
            "提示",
            "查看準備路線",
            "重試",
            "下一項",
            "訓練模式",
            "今日待複習",
            "新路線",
            "自由練習",
            "錯誤重練",
            "匯出 JSON",
            "匯入 JSON",
            "新增獨立庫",
            "合併至所選庫",
            "取消",
            "套用",
            "開局庫說明",
            "路線名稱",
            "路線註解",
            "尚未儲存",
            "修改尚未儲存，是否離開？",
            "放棄修改",
            "繼續編輯",
            "確認刪除所選項目？",
            "請選擇目標開局庫",
            "加入所選開局庫",
            "匯入半回合",
            "增加",
            "減少",
            "未識別",
            "學習統計",
            "詳情",
            "輸入名稱或說明",
            "個人準備，非引擎推薦",
            "自己的準備著法",
            "預設對手回應",
            "今日已完成",
            "複習已儲存",
            "僅匯入實際主線開局",
            "說明",
            "路線結束",
            "等待你的準備著法",
            "預設路線推進",
            "本輪結束",
            "今日無待訓練節點",
            "記憶正確",
            "不屬於目前準備路線，非棋理差著",
            "非法著法",
            "已使用提示",
            "已顯示準備路線",
            "已學習",
            "未學習",
            "待複習",
            "今日已複習",
            "近期記憶正確率",
            "錯誤次數",
            "提示次數",
            "重試儲存進度",
            "放棄未儲存進度",
            "訓練結果尚未儲存",
        ],
        Language::English => [
            "Opening Repertoire",
            "New White repertoire",
            "New Black repertoire",
            "Rename",
            "Edit routes",
            "Start training",
            "Delete repertoire",
            "Back",
            "Routes",
            "No repertoires yet",
            "Training side",
            "Save route",
            "Create branch",
            "Delete route",
            "Initial position",
            "Previous",
            "Next",
            "Hint",
            "Show prepared line",
            "Retry",
            "Next item",
            "Training mode",
            "Due today",
            "New routes",
            "Free practice",
            "Retry errors",
            "Export JSON",
            "Import JSON",
            "Create independent",
            "Merge into selected",
            "Cancel",
            "Apply",
            "Repertoire note",
            "Route name",
            "Route note",
            "Unsaved changes",
            "Changes are not saved. Leave?",
            "Discard changes",
            "Keep editing",
            "Delete the selected item?",
            "Choose target repertoire",
            "Add to selected",
            "Import plies",
            "More",
            "Less",
            "Unknown",
            "Learning statistics",
            "Details",
            "Enter name or note",
            "Personal preparation, not engine advice",
            "Your prepared moves",
            "Scripted opponent replies",
            "Today completed",
            "Review saved",
            "Actual mainline opening only",
            "Notes",
            "End of prepared line",
            "Waiting for your prepared move",
            "Prepared continuation",
            "Session finished",
            "No eligible training nodes today",
            "Correct recall",
            "Outside this preparation, not a chess-quality judgment",
            "Illegal move",
            "Hint used",
            "Prepared line revealed",
            "Learned",
            "Unlearned",
            "Due",
            "Reviewed today",
            "Recent recall accuracy",
            "Errors",
            "Hints",
            "Retry saving progress",
            "Discard unsaved progress",
            "Training result is not saved",
        ],
        Language::French => [
            "Répertoire personnel",
            "Nouveau répertoire blanc",
            "Nouveau répertoire noir",
            "Renommer",
            "Modifier variantes",
            "Entraînement",
            "Supprimer répertoire",
            "Retour",
            "Variantes",
            "Aucun répertoire",
            "Camp étudié",
            "Enregistrer variante",
            "Créer branche",
            "Supprimer variante",
            "Position initiale",
            "Précédent",
            "Suivant",
            "Indice",
            "Voir ligne préparée",
            "Réessayer",
            "Élément suivant",
            "Mode",
            "À réviser aujourd’hui",
            "Nouvelles variantes",
            "Pratique libre",
            "Revoir erreurs",
            "Exporter JSON",
            "Importer JSON",
            "Créer indépendant",
            "Fusionner sélection",
            "Annuler",
            "Appliquer",
            "Note du répertoire",
            "Nom de variante",
            "Note de variante",
            "Non enregistré",
            "Modifications non enregistrées. Quitter ?",
            "Abandonner",
            "Continuer édition",
            "Supprimer la sélection ?",
            "Choisir répertoire cible",
            "Ajouter à la sélection",
            "Demi-coups importés",
            "Plus",
            "Moins",
            "Inconnu",
            "Statistiques",
            "Détails",
            "Saisir nom ou note",
            "Préparation personnelle, pas conseil moteur",
            "Vos coups préparés",
            "Réponses adverses préparées",
            "Révisions terminées",
            "Révision enregistrée",
            "Début de ligne principale seulement",
            "Notes",
            "Fin de variante",
            "Votre coup préparé",
            "Suite préparée",
            "Séance terminée",
            "Aucun point à étudier aujourd’hui",
            "Rappel correct",
            "Hors préparation, sans jugement de qualité",
            "Coup illégal",
            "Indice utilisé",
            "Préparation révélée",
            "Appris",
            "À apprendre",
            "À réviser",
            "Révisés aujourd’hui",
            "Exactitude récente du rappel",
            "Erreurs",
            "Indices",
            "Réessayer enregistrement",
            "Abandonner progrès non enregistrés",
            "Résultat non enregistré",
        ],
        Language::Spanish => [
            "Repertorio personal",
            "Nuevo repertorio blanco",
            "Nuevo repertorio negro",
            "Renombrar",
            "Editar variantes",
            "Entrenar",
            "Eliminar repertorio",
            "Volver",
            "Variantes",
            "Sin repertorios",
            "Bando de estudio",
            "Guardar variante",
            "Crear rama",
            "Eliminar variante",
            "Posición inicial",
            "Anterior",
            "Siguiente",
            "Pista",
            "Ver línea preparada",
            "Reintentar",
            "Siguiente elemento",
            "Modo",
            "Para hoy",
            "Variantes nuevas",
            "Práctica libre",
            "Repetir errores",
            "Exportar JSON",
            "Importar JSON",
            "Crear independiente",
            "Fusionar seleccionado",
            "Cancelar",
            "Aplicar",
            "Nota del repertorio",
            "Nombre de variante",
            "Nota de variante",
            "Sin guardar",
            "Cambios sin guardar. ¿Salir?",
            "Descartar cambios",
            "Seguir editando",
            "¿Eliminar selección?",
            "Elige repertorio destino",
            "Añadir al seleccionado",
            "Medias jugadas",
            "Más",
            "Menos",
            "Desconocido",
            "Estadísticas",
            "Detalles",
            "Escribe nombre o nota",
            "Preparación personal, no consejo de motor",
            "Tus jugadas preparadas",
            "Respuestas rivales preparadas",
            "Revisión de hoy terminada",
            "Revisión guardada",
            "Solo apertura de la línea principal",
            "Notas",
            "Fin de la preparación",
            "Esperando tu jugada preparada",
            "Continuación preparada",
            "Sesión terminada",
            "Sin nodos para hoy",
            "Recuerdo correcto",
            "Fuera de preparación, sin juicio ajedrecístico",
            "Jugada ilegal",
            "Pista usada",
            "Preparación revelada",
            "Aprendidos",
            "Sin aprender",
            "Pendientes",
            "Repasados hoy",
            "Acierto reciente de memoria",
            "Errores",
            "Pistas",
            "Reintentar guardar progreso",
            "Descartar progreso sin guardar",
            "Resultado sin guardar",
        ],
        Language::Latin => [
            "Repertorium",
            "Novum album",
            "Novum nigrum",
            "Nomen mutare",
            "Series edere",
            "Exercere",
            "Repertorium delere",
            "Redire",
            "Series",
            "Nullum repertorium",
            "Color discendus",
            "Servare seriem",
            "Ramus novus",
            "Seriem delere",
            "Initium",
            "Ante",
            "Post",
            "Indicium",
            "Videre seriem",
            "Iterare",
            "Proximum",
            "Modus",
            "Hodie repetenda",
            "Novae series",
            "Exercitatio libera",
            "Errores iterare",
            "Exportare JSON",
            "Importare JSON",
            "Novum separatum",
            "Coniungere electo",
            "Revocare",
            "Applicare",
            "Nota repertorii",
            "Nomen seriei",
            "Nota seriei",
            "Nondum servatum",
            "Mutationes non servatae. Exire?",
            "Mutationes omittere",
            "Pergere edendo",
            "Delere electum?",
            "Elige repertorium",
            "Addere electo",
            "Semimotus",
            "Plus",
            "Minus",
            "Ignotum",
            "Numeri studii",
            "Notitia",
            "Scribe nomen vel notam",
            "Praeparatio propria, non consilium machinae",
            "Tui motus parati",
            "Responsa parata",
            "Hodie completum",
            "Repetitio servata",
            "Sola series principalis",
            "Notae",
            "Finis seriei",
            "Exspectat tuum motum paratum",
            "Continuatio parata",
            "Exercitatio finita",
            "Nullus nodus hodie",
            "Memoria recta",
            "Extra praeparationem, non iudicium scaccorum",
            "Motus illicitus",
            "Indicium adhibitum",
            "Series ostensa",
            "Discitum",
            "Non discitum",
            "Repetenda",
            "Hodie repetita",
            "Recens memoriae rectitudo",
            "Errores",
            "Indicia",
            "Iterum servare progressum",
            "Progressum non servatum omittere",
            "Exitus nondum servatus",
        ],
        Language::Japanese => [
            "個人オープニング集",
            "白番用を作成",
            "黒番用を作成",
            "名前を変更",
            "手順を編集",
            "練習開始",
            "開局集を削除",
            "戻る",
            "手順",
            "開局集はありません",
            "練習する側",
            "手順を保存",
            "分岐を作成",
            "手順を削除",
            "初期局面",
            "前へ",
            "次へ",
            "ヒント",
            "準備手順を表示",
            "再挑戦",
            "次の項目",
            "練習モード",
            "今日の復習",
            "新しい手順",
            "自由練習",
            "誤答の復習",
            "JSON出力",
            "JSON読込",
            "独立して新規作成",
            "選択集へ統合",
            "キャンセル",
            "適用",
            "開局集の説明",
            "手順の名前",
            "手順の注釈",
            "未保存",
            "変更が未保存です。離れますか？",
            "変更を破棄",
            "編集を続ける",
            "選択項目を削除しますか？",
            "追加先を選択",
            "選択集に追加",
            "読込半手数",
            "増やす",
            "減らす",
            "不明",
            "学習統計",
            "詳細",
            "名前または説明を入力",
            "自分の準備・エンジン推奨ではありません",
            "自分の準備手",
            "保存された相手の応手",
            "今日の復習完了",
            "復習を保存",
            "実際の本譜の序盤のみ",
            "説明",
            "準備手順の終端",
            "準備した手を指してください",
            "保存された手順を進行",
            "今回の練習は終了",
            "今日の対象ノードはありません",
            "正しく記憶",
            "準備外の手です・棋力上の悪手判定ではありません",
            "非合法手",
            "ヒント使用済み",
            "準備手順を表示済み",
            "学習済み",
            "未学習",
            "復習対象",
            "今日の復習済み",
            "最近の記憶正答率",
            "誤答回数",
            "ヒント回数",
            "進捗の保存を再試行",
            "未保存の進捗を破棄",
            "練習結果が未保存です",
        ],
        Language::Korean => [
            "개인 오프닝 모음",
            "백 모음 만들기",
            "흑 모음 만들기",
            "이름 변경",
            "변화 편집",
            "훈련 시작",
            "모음 삭제",
            "뒤로",
            "변화",
            "모음 없음",
            "훈련 진영",
            "변화 저장",
            "분기 만들기",
            "변화 삭제",
            "초기 위치",
            "이전",
            "다음",
            "힌트",
            "준비 수순 보기",
            "재시도",
            "다음 항목",
            "훈련 모드",
            "오늘 복습",
            "새 수순",
            "자유 연습",
            "오답 복습",
            "JSON 내보내기",
            "JSON 가져오기",
            "별도 모음 생성",
            "선택 모음과 병합",
            "취소",
            "적용",
            "모음 설명",
            "수순 이름",
            "수순 주석",
            "저장되지 않음",
            "변경을 저장하지 않았습니다. 나갈까요?",
            "변경 버리기",
            "편집 계속",
            "선택 항목을 삭제할까요?",
            "대상 모음 선택",
            "선택 모음에 추가",
            "가져올 반수",
            "늘리기",
            "줄이기",
            "미확인",
            "학습 통계",
            "상세",
            "이름 또는 설명 입력",
            "개인 준비이며 엔진 권장이 아님",
            "내 준비 수",
            "저장된 상대 응수",
            "오늘 복습 완료",
            "복습 저장됨",
            "실제 본선의 오프닝만",
            "설명",
            "준비 수순 끝",
            "준비한 수를 기다리는 중",
            "준비 수순 진행",
            "이번 훈련 종료",
            "오늘 훈련할 노드 없음",
            "정확한 기억",
            "준비 밖의 수이며 나쁜 수 판정이 아님",
            "불법 수",
            "힌트 사용함",
            "준비 수순 공개됨",
            "학습함",
            "미학습",
            "복습 대상",
            "오늘 복습함",
            "최근 기억 정답률",
            "오류 횟수",
            "힌트 횟수",
            "진행 저장 재시도",
            "저장하지 않은 진행 버리기",
            "훈련 결과가 저장되지 않음",
        ],
        Language::German => [
            "Eigenes Repertoire",
            "Neues Weißrepertoire",
            "Neues Schwarzrepertoire",
            "Umbenennen",
            "Varianten bearbeiten",
            "Training starten",
            "Repertoire löschen",
            "Zurück",
            "Varianten",
            "Keine Repertoires",
            "Trainingsfarbe",
            "Variante speichern",
            "Abzweigung erstellen",
            "Variante löschen",
            "Anfangsstellung",
            "Zurück",
            "Weiter",
            "Hinweis",
            "Vorbereitung zeigen",
            "Wiederholen",
            "Nächste Aufgabe",
            "Trainingsmodus",
            "Heute fällig",
            "Neue Varianten",
            "Freies Training",
            "Fehler üben",
            "JSON exportieren",
            "JSON importieren",
            "Unabhängig anlegen",
            "Mit Auswahl verbinden",
            "Abbrechen",
            "Anwenden",
            "Repertoire-Notiz",
            "Variantenname",
            "Varianten-Notiz",
            "Ungespeichert",
            "Änderungen ungespeichert. Verlassen?",
            "Verwerfen",
            "Weiter bearbeiten",
            "Auswahl löschen?",
            "Zielrepertoire wählen",
            "Zur Auswahl hinzufügen",
            "Halbzüge importieren",
            "Mehr",
            "Weniger",
            "Unbekannt",
            "Lernstatistik",
            "Details",
            "Name oder Notiz eingeben",
            "Eigene Vorbereitung, kein Engine-Rat",
            "Eigene vorbereitete Züge",
            "Gespeicherte Gegenzüge",
            "Heute abgeschlossen",
            "Wiederholung gespeichert",
            "Nur tatsächliche Hauptlinie",
            "Notizen",
            "Ende der Vorbereitung",
            "Dein vorbereiteter Zug",
            "Gespeicherte Fortsetzung",
            "Sitzung beendet",
            "Heute keine passenden Lernknoten",
            "Richtig erinnert",
            "Außerhalb der Vorbereitung, kein Qualitätsurteil",
            "Illegaler Zug",
            "Hinweis genutzt",
            "Vorbereitung aufgedeckt",
            "Gelernt",
            "Ungelernt",
            "Fällig",
            "Heute wiederholt",
            "Aktuelle Erinnerungsquote",
            "Fehler",
            "Hinweise",
            "Fortschritt erneut speichern",
            "Ungespeicherten Fortschritt verwerfen",
            "Trainingsergebnis ungespeichert",
        ],
        Language::Italian => [
            "Repertorio personale",
            "Nuovo repertorio bianco",
            "Nuovo repertorio nero",
            "Rinomina",
            "Modifica varianti",
            "Inizia allenamento",
            "Elimina repertorio",
            "Indietro",
            "Varianti",
            "Nessun repertorio",
            "Colore allenato",
            "Salva variante",
            "Crea ramo",
            "Elimina variante",
            "Posizione iniziale",
            "Indietro",
            "Avanti",
            "Indizio",
            "Mostra preparazione",
            "Riprova",
            "Prossimo",
            "Modalità",
            "Da ripassare oggi",
            "Nuove varianti",
            "Pratica libera",
            "Ripassa errori",
            "Esporta JSON",
            "Importa JSON",
            "Crea indipendente",
            "Unisci al selezionato",
            "Annulla",
            "Applica",
            "Nota repertorio",
            "Nome variante",
            "Nota variante",
            "Non salvato",
            "Modifiche non salvate. Uscire?",
            "Scarta modifiche",
            "Continua modifica",
            "Eliminare selezione?",
            "Scegli repertorio destinazione",
            "Aggiungi al selezionato",
            "Semimosse importate",
            "Più",
            "Meno",
            "Sconosciuto",
            "Statistiche",
            "Dettagli",
            "Inserisci nome o nota",
            "Preparazione personale, non consiglio del motore",
            "Mosse proprie preparate",
            "Risposte avversarie salvate",
            "Ripasso completato",
            "Ripasso salvato",
            "Solo apertura della linea principale",
            "Note",
            "Fine della preparazione",
            "Attesa della tua mossa preparata",
            "Continuazione preparata",
            "Sessione terminata",
            "Nessun nodo da studiare oggi",
            "Ricordo corretto",
            "Fuori preparazione, non giudizio sulla mossa",
            "Mossa illegale",
            "Indizio usato",
            "Preparazione rivelata",
            "Appresi",
            "Da apprendere",
            "Da ripassare",
            "Ripassati oggi",
            "Precisione recente del ricordo",
            "Errori",
            "Indizi",
            "Riprova salvataggio progressi",
            "Scarta progressi non salvati",
            "Risultato non salvato",
        ],
        Language::Russian => [
            "Личный репертуар",
            "Новый за белых",
            "Новый за чёрных",
            "Переименовать",
            "Изменить варианты",
            "Начать тренировку",
            "Удалить репертуар",
            "Назад",
            "Варианты",
            "Нет репертуаров",
            "Сторона обучения",
            "Сохранить вариант",
            "Создать ветку",
            "Удалить вариант",
            "Начальная позиция",
            "Назад",
            "Далее",
            "Подсказка",
            "Показать подготовку",
            "Повторить",
            "Следующее",
            "Режим",
            "Повторить сегодня",
            "Новые варианты",
            "Свободная практика",
            "Повтор ошибок",
            "Экспорт JSON",
            "Импорт JSON",
            "Создать отдельно",
            "Объединить с выбранным",
            "Отмена",
            "Применить",
            "Описание репертуара",
            "Название варианта",
            "Примечание варианта",
            "Не сохранено",
            "Изменения не сохранены. Выйти?",
            "Отбросить изменения",
            "Продолжить правку",
            "Удалить выбранное?",
            "Выберите репертуар",
            "Добавить в выбранный",
            "Импорт полуходов",
            "Больше",
            "Меньше",
            "Неизвестно",
            "Учебная статистика",
            "Сведения",
            "Введите имя или примечание",
            "Личная подготовка, не совет движка",
            "Ваши подготовленные ходы",
            "Сохранённые ответы соперника",
            "На сегодня завершено",
            "Повторение сохранено",
            "Только дебют основной линии",
            "Примечания",
            "Конец подготовки",
            "Ожидается подготовленный ход",
            "Подготовленное продолжение",
            "Сеанс завершён",
            "Сегодня нет подходящих узлов",
            "Верно вспомнили",
            "Вне подготовки, не оценка качества хода",
            "Нелегальный ход",
            "Подсказка использована",
            "Подготовка показана",
            "Изучено",
            "Не изучено",
            "К повторению",
            "Повторено сегодня",
            "Недавняя точность памяти",
            "Ошибки",
            "Подсказки",
            "Повторить сохранение",
            "Отбросить несохранённый прогресс",
            "Результат тренировки не сохранён",
        ],
    }
}

fn short(font: &TextRenderer, text: &str, width: f32) -> String {
    let mut out = String::new();
    for c in text.chars().take(256) {
        if font.text_width(&format!("{out}{c}…"), 14.0) > width {
            out.push('…');
            return out;
        }
        out.push(c);
    }
    if text.chars().count() > 256 {
        out.push('…');
    }
    out
}
fn button(c: &mut Canvas<'_>, rect: (f32, f32, f32, f32), label: &str, a: A) {
    c.button(rect, label, Action::Repertoire(a));
}
fn line(c: &mut Canvas<'_>, x: f32, y: f32, text: &str, width: f32) {
    c.line(x, y, &short(c.font, text, width), width, c.pal.text);
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
        button(&mut c, (36.0, 24.0, 230.0, 32.0), t[0], A::Enter);
        return c.actions;
    }
    if app.screen == Screen::Opening {
        button(
            &mut c,
            (w - 282.0, h - 78.0, 270.0, 30.0),
            t[41],
            A::FromOpening,
        );
        return c.actions;
    }
    if app.screen == Screen::Library {
        if !app.library.busy() && app.library.remove.is_none() {
            button(
                &mut c,
                ((w - 460.0).max(180.0), 6.0, 214.0, 30.0),
                t[41],
                A::FromLibrary,
            );
        }
        return c.actions;
    }
    if app.screen != Screen::Repertoire {
        return vec![];
    }
    let s = &app.repertoire;
    let x = w - 292.0;
    let dirty = if s.dirty {
        format!(" · * {}", t[35])
    } else {
        String::new()
    };
    if s.page == Page::Home {
        fill_rect(c.buf, size, 0, 0, size.0 as i32, size.1 as i32, c.pal.bg);
        fill_rect(c.buf, size, x as i32, 0, 292, size.1 as i32, c.pal.panel);
        line(
            &mut c,
            16.0,
            28.0,
            &format!("{} ({})", t[0], s.content.repertoires.len()),
            x - 32.0,
        );
        line(&mut c, 16.0, 53.0, &s.stats, x - 32.0);
        let top = if s.pending_route.is_some() {
            112.0
        } else {
            76.0
        };
        if s.pending_route.is_some() {
            line(&mut c, 16.0, 86.0, t[40], x - 32.0);
        }
        let rows = ((h - top - 130.0) / 58.0).max(1.0) as usize;
        if s.content.repertoires.is_empty() {
            line(&mut c, 16.0, top + 24.0, t[9], x - 32.0);
        }
        for (row, i) in (s.scroll..s.content.repertoires.len())
            .take(rows)
            .enumerate()
        {
            let b = &s.content.repertoires[i];
            let y = top + row as f32 * 58.0;
            button(&mut c, (16.0, y, x - 54.0, 52.0), "", A::SelectBook(i));
            let selected = if s.selected.as_ref() == Some(&b.id) {
                "> "
            } else {
                ""
            };
            line(
                &mut c,
                24.0,
                y + 20.0,
                &format!("{selected}{}", b.name),
                x - 74.0,
            );
            let color = if b.side.color() == shakmaty::Color::White {
                app.settings.language.text().white
            } else {
                app.settings.language.text().black
            };
            line(
                &mut c,
                24.0,
                y + 43.0,
                &format!(
                    "{} · {} {} · {}",
                    color,
                    t[8],
                    b.routes.len(),
                    s.book_stats.get(&b.id).map(String::as_str).unwrap_or("")
                ),
                x - 74.0,
            );
        }
        button(&mut c, (x - 34.0, top, 22.0, 28.0), "↑", A::Up);
        button(&mut c, (x - 34.0, top + 36.0, 22.0, 28.0), "↓", A::Down);
        for (i, (label, a)) in [
            (t[1], A::NewWhite),
            (t[2], A::NewBlack),
            (t[3], A::Rename),
            (t[32], A::BookNote),
            (t[4], A::Edit),
            (t[5], A::Train),
            (t[6], A::RemoveBook),
            (t[26], A::Export),
            (t[27], A::Import),
        ]
        .into_iter()
        .enumerate()
        {
            button(
                &mut c,
                (x + 10.0, 16.0 + i as f32 * 35.0, 270.0, 30.0),
                label,
                a,
            );
        }
        button(
            &mut c,
            (x + 10.0, 336.0, 270.0, 30.0),
            &format!("{}: {}", t[21], t[22 + s.mode.min(3)]),
            A::Mode,
        );
        button(
            &mut c,
            (x + 10.0, 374.0, 270.0, 30.0),
            &format!("{}{}", if s.error.is_some() { "! " } else { "" }, t[47]),
            A::Details,
        );
        if s.pending_route.is_some() {
            line(
                &mut c,
                16.0,
                h - 118.0,
                &format!("{}: {} · {}", t[42], s.import_plies, t[54]),
                x - 32.0,
            );
            button(
                &mut c,
                (16.0, h - 102.0, (x - 120.0).max(80.0), 30.0),
                t[41],
                A::AddPending,
            );
            button(
                &mut c,
                (x - 96.0, h - 102.0, 38.0, 30.0),
                "−",
                A::PrefixLess,
            );
            button(
                &mut c,
                (x - 50.0, h - 102.0, 38.0, 30.0),
                "+",
                A::PrefixMore,
            );
        } else if let Some(notice) = &s.notice {
            line(&mut c, 16.0, h - 87.0, notice, x - 32.0);
        }
        button(&mut c, (16.0, h - 40.0, x - 32.0, 30.0), t[7], A::Back);
    } else {
        fill_rect(c.buf, size, x as i32, 0, 292, size.1 as i32, c.pal.panel);
        let name = s.book().map(|b| b.name.as_str()).unwrap_or(t[9]);
        line(&mut c, x + 10.0, 24.0, &format!("{name}{dirty}"), 270.0);
        line(
            &mut c,
            12.0,
            24.0,
            &format!(
                "{}{}",
                if s.error.is_some() { "! " } else { "" },
                if s.eco.is_empty() { t[45] } else { &s.eco }
            ),
            x - 24.0,
        );
        let ply = if s.page == Page::Train {
            s.session
                .as_ref()
                .map(|session| session.path.len())
                .unwrap_or(0)
        } else {
            s.cursor
        };
        line(
            &mut c,
            12.0,
            h - 53.0,
            &format!(
                "{}{} · {}",
                s.notice
                    .as_ref()
                    .map(|v| format!("{v} · "))
                    .unwrap_or_default(),
                ply,
                s.path
            ),
            x - 24.0,
        );
        if s.page == Page::Edit {
            for (i, (label, a)) in [
                (t[11], A::SaveRoute),
                (t[12], A::Branch),
                (t[33], A::RouteName),
                (t[34], A::RouteNote),
                (t[13], A::RemoveRoute),
                (t[32], A::BookNote),
            ]
            .into_iter()
            .enumerate()
            {
                button(
                    &mut c,
                    (
                        x + 10.0 + (i % 2) as f32 * 138.0,
                        34.0 + (i / 2) as f32 * 32.0,
                        132.0,
                        28.0,
                    ),
                    label,
                    a,
                );
            }
            line(&mut c, x + 10.0, 146.0, t[8], 245.0);
            if let Some(b) = s.book() {
                let rows = ((h - 300.0) / 42.0).max(1.0) as usize;
                for (row, i) in (s.scroll..b.routes.len()).take(rows).enumerate() {
                    let r = &b.routes[i];
                    let y = 154.0 + row as f32 * 42.0;
                    let label = format!(
                        "{}{} · {}",
                        if s.route.as_ref() == Some(&r.id) {
                            "> "
                        } else {
                            ""
                        },
                        if r.name.is_empty() {
                            format!("{} {}", t[8], i + 1)
                        } else {
                            r.name.clone()
                        },
                        r.moves.len()
                    );
                    button(
                        &mut c,
                        (x + 10.0, y, 244.0, 38.0),
                        &short(font, &label, 230.0),
                        A::SelectRoute(i),
                    );
                }
            }
            button(&mut c, (x + 260.0, 154.0, 20.0, 28.0), "↑", A::Up);
            button(&mut c, (x + 260.0, 190.0, 20.0, 28.0), "↓", A::Down);
            button(
                &mut c,
                (x + 10.0, h - 104.0, 132.0, 28.0),
                &format!("{}{}", if s.error.is_some() { "! " } else { "" }, t[47]),
                A::Details,
            );
            button(&mut c, (x + 148.0, h - 104.0, 132.0, 28.0), t[5], A::Train);
            line(&mut c, x + 10.0, h - 58.0, t[49], 270.0);
            button(&mut c, (x + 10.0, h - 40.0, 270.0, 28.0), t[7], A::Back);
            let col = (x - 40.0) / 3.0;
            for (i, (label, a)) in [(t[14], A::First), (t[15], A::Previous), (t[16], A::Next)]
                .into_iter()
                .enumerate()
            {
                button(
                    &mut c,
                    (12.0 + i as f32 * (col + 8.0), h - 38.0, col, 28.0),
                    label,
                    a,
                );
            }
        } else {
            button(
                &mut c,
                (x + 10.0, 34.0, 270.0, 28.0),
                &format!("{}: {}", t[21], t[22 + s.mode.min(3)]),
                A::Mode,
            );
            for (i, (label, a)) in [
                (t[17], A::Hint),
                (t[18], A::Reveal),
                (t[19], A::Retry),
                (t[20], A::NextItem),
            ]
            .into_iter()
            .enumerate()
            {
                button(
                    &mut c,
                    (
                        x + 10.0 + (i % 2) as f32 * 138.0,
                        68.0 + (i / 2) as f32 * 32.0,
                        132.0,
                        28.0,
                    ),
                    label,
                    a,
                );
            }
            line(&mut c, x + 10.0, 153.0, &s.training_status, 270.0);
            c.wrapped(x + 10.0, 181.0, &s.training_detail, 270.0, 5);
            line(&mut c, x + 10.0, 312.0, t[46], 270.0);
            c.wrapped(x + 10.0, 336.0, &s.stats, 270.0, 3);
            button(
                &mut c,
                (x + 10.0, h - 76.0, 270.0, 28.0),
                &format!("{}{}", if s.error.is_some() { "! " } else { "" }, t[47]),
                A::Details,
            );
            button(&mut c, (x + 10.0, h - 40.0, 270.0, 28.0), t[7], A::Back);
        }
    }
    if s.pending_backup.is_some() || s.input.is_some() || s.confirm.is_some() {
        c.actions.clear();
        fill_rect_alpha(c.buf, size, (0, 0), size.0 as i32, size.1 as i32, 0, 190);
        let bw = (w - 48.0).min(650.0);
        let bh = (h - 48.0).min(350.0);
        let mx = (w - bw) / 2.0;
        let my = (h - bh) / 2.0;
        fill_rect(
            c.buf,
            size,
            mx as i32,
            my as i32,
            bw as i32,
            bh as i32,
            c.pal.panel,
        );
        // Exit guards must remain visible even while a text editor is open.
        if let Some(confirm) = s.confirm {
            if confirm == Confirm::Progress {
                c.wrapped(mx + 16.0, my + 36.0, t[75], bw - 32.0, 3);
                for (i, (label, a)) in [
                    (t[73], A::SaveProgress),
                    (t[74], A::DiscardProgress),
                    (t[38], A::KeepEdit),
                ]
                .into_iter()
                .enumerate()
                {
                    button(
                        &mut c,
                        (mx + 16.0, my + 112.0 + i as f32 * 48.0, bw - 32.0, 36.0),
                        label,
                        a,
                    );
                }
            } else if confirm == Confirm::Leave {
                c.wrapped(mx + 16.0, my + 36.0, t[36], bw - 32.0, 3);
                for (i, (label, a)) in [
                    (t[11], A::SaveRoute),
                    (t[37], A::DiscardEdit),
                    (t[38], A::KeepEdit),
                ]
                .into_iter()
                .enumerate()
                {
                    button(
                        &mut c,
                        (mx + 16.0, my + 112.0 + i as f32 * 48.0, bw - 32.0, 36.0),
                        label,
                        a,
                    );
                }
            } else {
                c.wrapped(mx + 16.0, my + 36.0, t[39], bw - 32.0, 3);
                button(
                    &mut c,
                    (mx + 16.0, my + bh - 52.0, (bw - 48.0) / 2.0, 34.0),
                    if confirm == Confirm::Route {
                        t[13]
                    } else {
                        t[6]
                    },
                    A::ConfirmRemove,
                );
                button(
                    &mut c,
                    (mx + bw / 2.0 + 8.0, my + bh - 52.0, (bw - 48.0) / 2.0, 34.0),
                    t[30],
                    A::CancelRemove,
                );
            }
        } else if let Some(input) = &s.input {
            line(&mut c, mx + 16.0, my + 30.0, t[48], bw - 32.0);
            let before = input.text.get(..input.cursor).unwrap_or(&input.text);
            let after = input.text.get(input.cursor..).unwrap_or("");
            let nearby: String = before
                .rsplit('\n')
                .take(3)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            let left: String = nearby
                .chars()
                .rev()
                .take(120)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let right: String = after.chars().take(80).collect();
            c.wrapped(
                mx + 16.0,
                my + 64.0,
                &format!("{left}▏{right}"),
                bw - 32.0,
                8,
            );
            button(
                &mut c,
                (mx + 16.0, my + bh - 48.0, (bw - 48.0) / 2.0, 32.0),
                t[31],
                A::ApplyText,
            );
            button(
                &mut c,
                (mx + bw / 2.0 + 8.0, my + bh - 48.0, (bw - 48.0) / 2.0, 32.0),
                t[30],
                A::CancelText,
            );
        } else if let Some(incoming) = &s.pending_backup {
            line(&mut c, mx + 16.0, my + 30.0, t[27], bw - 32.0);
            line(
                &mut c,
                mx + 16.0,
                my + 60.0,
                &format!("{}: {}", t[0], incoming.repertoires.len()),
                bw - 32.0,
            );
            let target = s
                .book()
                .map(|b| {
                    format!(
                        "{} · {}",
                        b.name,
                        if b.side.color() == shakmaty::Color::White {
                            app.settings.language.text().white
                        } else {
                            app.settings.language.text().black
                        }
                    )
                })
                .unwrap_or_else(|| t[40].into());
            line(&mut c, mx + 16.0, my + 90.0, &target, bw - 32.0);
            for (i, (label, a)) in [
                (t[28], A::ImportNew),
                (t[29], A::ImportMerge),
                (t[30], A::CancelImport),
            ]
            .into_iter()
            .enumerate()
            {
                button(
                    &mut c,
                    (mx + 16.0, my + 125.0 + i as f32 * 48.0, bw - 32.0, 36.0),
                    label,
                    a,
                );
            }
        }
        if let Some(error) = &s.error {
            let y = if s.confirm.is_some() {
                my + 94.0
            } else if s.input.is_some() {
                my + bh - 62.0
            } else {
                my + 112.0
            };
            c.line(
                mx + 16.0,
                y,
                &short(font, &format!("! {error}"), bw - 32.0),
                bw - 32.0,
                c.pal.accent,
            );
        }
    }

    c.actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assets::PieceImages,
        render::{Layout, Renderer},
        repertoire::{Collection, Side, validate_route},
        repertoire_train::{Mode, Session},
        uci_tests,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn app() -> App {
        static N: AtomicUsize = AtomicUsize::new(0);
        let mut a = uci_tests::app();
        let root = std::env::current_dir()
            .unwrap()
            .join("target/v028-qa/ui")
            .join(format!(
                "{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
        a.records.save_path = Some(root.join("autosave-v1.json"));
        a.repertoire.root = Some(root);
        let mut content = Collection::default();
        let white = content
            .create("Unicode 白方 repertoire", Side::White, 1)
            .unwrap();
        let black = content
            .create("Unicode 黑方 repertoire", Side::Black, 1)
            .unwrap();
        for id in [&white, &black] {
            for last in ["f8c5", "g8f6"] {
                content
                    .add_route(
                        id,
                        ["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", last]
                            .into_iter()
                            .map(str::to_owned)
                            .collect(),
                        "Italian variation",
                        "Unicode 注释",
                        2,
                    )
                    .unwrap();
            }
        }
        a.repertoire.content = content;
        a.repertoire.selected = Some(white);
        a.repertoire.page = Page::Edit;
        a.screen = Screen::Repertoire;
        set_editor(&mut a);
        a
    }
    fn set_editor(a: &mut App) {
        let route = a.repertoire.book().unwrap().routes[0].clone();
        let validated = validate_route(&route).unwrap();
        a.repertoire.route = Some(route.id);
        a.repertoire.positions = validated.positions;
        a.repertoire.moves = validated.moves;
        a.repertoire.cursor = 3;
        a.refresh_repertoire();
    }
    fn click(a: &App, font: &TextRenderer, size: (usize, usize), point: (f32, f32), expected: A) {
        let mut buf = vec![0; size.0 * size.1];
        let actions = draw(font, &mut buf, size, a, Some(point), true);
        assert!(
            actions
                .iter()
                .any(|action| matches!(action,Action::Repertoire(found) if *found==expected)),
            "expected {expected:?}, got {actions:?}"
        );
    }
    #[test]
    fn all_repertoire_labels_have_eleven_language_text() {
        for language in crate::i18n::Language::ALL {
            assert!(super::words(language).iter().all(|s| !s.is_empty()));
        }
    }
    #[test]
    fn home_editor_training_eleven_languages_two_sizes_keep_board_and_coordinates_clear() {
        let mut a = app();
        let images = PieceImages::load();
        let font = TextRenderer::load();
        let renderer = Renderer::new(&images, &font);
        let live = crate::fen::write(&a.pos);
        let clocks = a.clocks;
        for lang in Language::ALL {
            a.settings.language = lang;
            for size in [(640, 480), (1280, 720)] {
                for page in [Page::Home, Page::Edit, Page::Train] {
                    a.repertoire.page = page;
                    if page == Page::Train {
                        let book = a.repertoire.book().unwrap().clone();
                        a.repertoire.session = Some(
                            Session::new(&book, Mode::All, &a.repertoire.progress, 100, 0).unwrap(),
                        );
                    }
                    a.refresh_repertoire();
                    let mut view = a.view_state(None, false, false, false);
                    a.repertoire_view(&mut view);
                    let mut buf = vec![0; size.0 * size.1];
                    assert!(renderer.render(&mut buf, size.0, size.1, &view).is_empty());
                    let baseline = buf.clone();
                    assert!(draw(&font, &mut buf, size, &a, None, false).is_empty());
                    if page == Page::Train {
                        // The hidden editor cursor must not change the displayed training ply.
                        let editor_cursor = a.repertoire.cursor;
                        a.repertoire.cursor = 0;
                        let mut independent_cursor = baseline.clone();
                        draw(&font, &mut independent_cursor, size, &a, None, false);
                        assert_eq!(buf, independent_cursor, "training inherited editor cursor");
                        a.repertoire.cursor = editor_cursor;
                    }
                    if page != Page::Home {
                        let layout = Layout::for_screen(size.0, size.1, Screen::Repertoire, false);
                        for y in layout.board_top as usize
                            ..(layout.board_top + layout.board_size + 24.0) as usize
                        {
                            let start = y * size.0 + layout.board_left as usize;
                            let end = start + layout.board_size as usize;
                            assert_eq!(
                                &buf[start..end],
                                &baseline[start..end],
                                "{lang:?} {size:?} {page:?} y={y}"
                            );
                        }
                    }
                }
                for i in 0..a.repertoire.content.repertoires.len() {
                    a.repertoire.selected = Some(a.repertoire.content.repertoires[i].id.clone());
                    a.repertoire.page = Page::Train;
                    let book = a.repertoire.book().unwrap().clone();
                    a.repertoire.session = Some(
                        Session::new(&book, Mode::All, &a.repertoire.progress, 100, 0).unwrap(),
                    );
                    let mut view = a.view_state(None, false, false, false);
                    a.repertoire_view(&mut view);
                    assert_eq!(view.settings.mode, Some(!book.side.color()));
                    assert!(view.settings.flip_for_black);
                }
            }
        }
        assert_eq!(crate::fen::write(&a.pos), live);
        assert_eq!(a.clocks, clocks);
        assert!(a.analysis.controller.is_none());
    }
    #[test]
    fn main_controls_import_sources_and_pending_prefix_hit_expected_actions() {
        let mut a = app();
        let font = TextRenderer::load();
        for size in [(640, 480), (1280, 720)] {
            let (w, h) = (size.0 as f32, size.1 as f32);
            let x = w - 292.0;
            a.screen = Screen::Menu;
            click(&a, &font, size, (42.0, 30.0), A::Enter);
            a.screen = Screen::Opening;
            click(&a, &font, size, (w - 272.0, h - 70.0), A::FromOpening);
            a.screen = Screen::Library;
            click(
                &a,
                &font,
                size,
                ((w - 460.0).max(180.0) + 6.0, 12.0),
                A::FromLibrary,
            );
            a.screen = Screen::Repertoire;
            a.repertoire.page = Page::Home;
            for (i, expected) in [
                A::NewWhite,
                A::NewBlack,
                A::Rename,
                A::BookNote,
                A::Edit,
                A::Train,
                A::RemoveBook,
                A::Export,
                A::Import,
            ]
            .into_iter()
            .enumerate()
            {
                click(
                    &a,
                    &font,
                    size,
                    (x + 20.0, 20.0 + i as f32 * 35.0),
                    expected,
                );
            }
            click(&a, &font, size, (x + 20.0, 342.0), A::Mode);
            click(&a, &font, size, (x + 20.0, 380.0), A::Details);
            click(&a, &font, size, (20.0, 84.0), A::SelectBook(0));
            click(&a, &font, size, (x - 25.0, 82.0), A::Up);
            click(&a, &font, size, (x - 25.0, 118.0), A::Down);
            click(&a, &font, size, (20.0, h - 34.0), A::Back);
            a.repertoire.pending_route = Some(vec!["e2e4".into()]);
            click(&a, &font, size, (20.0, h - 96.0), A::AddPending);
            click(&a, &font, size, (x - 90.0, h - 96.0), A::PrefixLess);
            click(&a, &font, size, (x - 44.0, h - 96.0), A::PrefixMore);
            a.repertoire.pending_route = None;
            a.repertoire.page = Page::Edit;
            for (i, expected) in [
                A::SaveRoute,
                A::Branch,
                A::RouteName,
                A::RouteNote,
                A::RemoveRoute,
                A::BookNote,
            ]
            .into_iter()
            .enumerate()
            {
                click(
                    &a,
                    &font,
                    size,
                    (
                        x + 20.0 + (i % 2) as f32 * 138.0,
                        40.0 + (i / 2) as f32 * 32.0,
                    ),
                    expected,
                );
            }
            click(&a, &font, size, (x + 20.0, 160.0), A::SelectRoute(0));
            click(&a, &font, size, (x + 266.0, 160.0), A::Up);
            click(&a, &font, size, (x + 266.0, 196.0), A::Down);
            click(&a, &font, size, (x + 20.0, h - 98.0), A::Details);
            click(&a, &font, size, (x + 154.0, h - 98.0), A::Train);
            click(&a, &font, size, (x + 20.0, h - 34.0), A::Back);
            let col = (x - 40.0) / 3.0;
            for (i, expected) in [A::First, A::Previous, A::Next].into_iter().enumerate() {
                click(
                    &a,
                    &font,
                    size,
                    (18.0 + i as f32 * (col + 8.0), h - 32.0),
                    expected,
                );
            }
            a.repertoire.page = Page::Train;
            click(&a, &font, size, (x + 20.0, 40.0), A::Mode);
            for (i, expected) in [A::Hint, A::Reveal, A::Retry, A::NextItem]
                .into_iter()
                .enumerate()
            {
                click(
                    &a,
                    &font,
                    size,
                    (
                        x + 20.0 + (i % 2) as f32 * 138.0,
                        74.0 + (i / 2) as f32 * 32.0,
                    ),
                    expected,
                );
            }
            click(&a, &font, size, (x + 20.0, h - 70.0), A::Details);
            click(&a, &font, size, (x + 20.0, h - 34.0), A::Back);
        }
    }
    #[test]
    fn text_backup_delete_and_unsaved_progress_modals_only_emit_their_own_actions() {
        let mut a = app();
        let font = TextRenderer::load();
        a.handle_repertoire_action(A::Rename);
        assert!(a.repertoire.input.is_some());
        let input = a.repertoire.input.as_mut().unwrap();
        input.text = "中文 keyboard cursor".into();
        input.cursor = 3;
        for size in [(640, 480), (1280, 720)] {
            let (w, h) = (size.0 as f32, size.1 as f32);
            let bw = (w - 48.0).min(650.0);
            let bh = (h - 48.0).min(350.0);
            let (mx, my) = ((w - bw) / 2.0, (h - bh) / 2.0);
            a.repertoire.confirm = None;
            a.repertoire.pending_backup = None;
            click(&a, &font, size, (mx + 20.0, my + bh - 42.0), A::ApplyText);
            click(
                &a,
                &font,
                size,
                (mx + bw / 2.0 + 14.0, my + bh - 42.0),
                A::CancelText,
            );
            // A close guard must take precedence over a still-open text editor.
            for (confirm, expected) in [
                (Confirm::Leave, [A::SaveRoute, A::DiscardEdit, A::KeepEdit]),
                (
                    Confirm::Progress,
                    [A::SaveProgress, A::DiscardProgress, A::KeepEdit],
                ),
            ] {
                a.repertoire.confirm = Some(confirm);
                for (i, action) in expected.into_iter().enumerate() {
                    click(
                        &a,
                        &font,
                        size,
                        (mx + 20.0, my + 118.0 + i as f32 * 48.0),
                        action,
                    );
                }
                let mut buf = vec![0; size.0 * size.1];
                let got = draw(&font, &mut buf, size, &a, Some((w - 272.0, 40.0)), true);
                assert!(
                    got.iter().all(
                        |g| matches!(g,Action::Repertoire(action) if expected.contains(action))
                    )
                );
            }
            for confirm in [Confirm::Book, Confirm::Route] {
                a.repertoire.confirm = Some(confirm);
                click(
                    &a,
                    &font,
                    size,
                    (mx + 20.0, my + bh - 46.0),
                    A::ConfirmRemove,
                );
                click(
                    &a,
                    &font,
                    size,
                    (mx + bw / 2.0 + 14.0, my + bh - 46.0),
                    A::CancelRemove,
                );
            }
            a.repertoire.confirm = None;
        }
        a.repertoire.input = None;
        a.repertoire.pending_backup = Some(a.repertoire.content.clone());
        for size in [(640, 480), (1280, 720)] {
            let (w, h) = (size.0 as f32, size.1 as f32);
            let bw = (w - 48.0).min(650.0);
            let bh = (h - 48.0).min(350.0);
            let (mx, my) = ((w - bw) / 2.0, (h - bh) / 2.0);
            for (i, action) in [A::ImportNew, A::ImportMerge, A::CancelImport]
                .into_iter()
                .enumerate()
            {
                click(
                    &a,
                    &font,
                    size,
                    (mx + 20.0, my + 131.0 + i as f32 * 48.0),
                    action,
                );
            }
            let mut buf = vec![0; size.0 * size.1];
            let actions = draw(&font, &mut buf, size, &a, Some((w - 272.0, 40.0)), true);
            assert!(
                actions.is_empty(),
                "underlying edit controls leaked through backup modal: {actions:?}"
            );
        }
    }
}
