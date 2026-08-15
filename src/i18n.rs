#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Chinese,
    English,
    French,
    Spanish,
    Latin,
}

impl Language {
    pub const ALL: [Language; 5] = [
        Language::Chinese,
        Language::English,
        Language::French,
        Language::Spanish,
        Language::Latin,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Language::Chinese => "zh-CN",
            Language::English => "en",
            Language::French => "fr",
            Language::Spanish => "es",
            Language::Latin => "la",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "zh-CN" | "zh" => Some(Language::Chinese),
            "en" => Some(Language::English),
            "fr" => Some(Language::French),
            "es" => Some(Language::Spanish),
            "la" => Some(Language::Latin),
            _ => None,
        }
    }

    pub fn native_name(self) -> &'static str {
        match self {
            Language::Chinese => "简体中文",
            Language::English => "English",
            Language::French => "Français",
            Language::Spanish => "Español",
            Language::Latin => "Latina",
        }
    }

    pub fn text(self) -> &'static Text {
        match self {
            Language::Chinese => &ZH,
            Language::English => &EN,
            Language::French => &FR,
            Language::Spanish => &ES,
            Language::Latin => &LA,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Text {
    pub title: &'static str,
    pub subtitle: &'static str,
    pub start_game: &'static str,
    pub settings: &'static str,
    pub exit: &'static str,
    pub attribution: &'static str,
    pub mode: &'static str,
    pub difficulty: &'static str,
    pub resolution: &'static str,
    pub refresh_rate: &'static str,
    pub theme: &'static str,
    pub language: &'static str,
    pub board_view: &'static str,
    pub back_to_menu: &'static str,
    pub two_players: &'static str,
    pub play_white: &'static str,
    pub play_black: &'static str,
    pub beginner: &'static str,
    pub easy: &'static str,
    pub medium: &'static str,
    pub hard: &'static str,
    pub master: &'static str,
    pub flip_on: &'static str,
    pub flip_off: &'static str,
    pub new_game: &'static str,
    pub undo: &'static str,
    pub menu: &'static str,
    pub hint: &'static str,
    pub hint_thinking: &'static str,
    pub suggested: &'static str,
    pub moves: &'static str,
    pub ai_thinking: &'static str,
    pub white: &'static str,
    pub black: &'static str,
    pub wins: &'static str,
    pub checkmate: &'static str,
    pub draw: &'static str,
    pub to_move: &'static str,
    pub check: &'static str,
    pub game_over: &'static str,
    pub play_again: &'static str,
    pub promotion: &'static str,
    pub queen: &'static str,
    pub rook: &'static str,
    pub bishop: &'static str,
    pub knight: &'static str,
}

static ZH: Text = Text {
    title: "RUST CHESS", subtitle: "Rust · 本地窗口对弈", start_game: "开始游戏",
    settings: "游戏设置", exit: "退出游戏", attribution: "素材来源：chess-viewer 开源项目 · CBurnett",
    mode: "对战模式", difficulty: "AI 难度", resolution: "窗口分辨率", refresh_rate: "刷新率",
    theme: "主题", language: "语言", board_view: "执黑时翻转棋盘", back_to_menu: "返回主菜单",
    two_players: "双人对战", play_white: "人机 · 玩家执白", play_black: "人机 · 玩家执黑",
    beginner: "入门", easy: "简单", medium: "中等", hard: "困难", master: "大师",
    flip_on: "开启", flip_off: "关闭", new_game: "新对局", undo: "悔棋", menu: "主菜单",
    hint: "决策建议", hint_thinking: "正在分析…", suggested: "建议", moves: "棋谱",
    ai_thinking: "电脑思考中…", white: "白方", black: "黑方", wins: "获胜", checkmate: "将杀！",
    draw: "和棋", to_move: "行棋", check: "（将军！）", game_over: "本局结束", play_again: "再来一局",
    promotion: "选择升变棋子", queen: "后", rook: "车", bishop: "象", knight: "马",
};

static EN: Text = Text {
    title: "RUST CHESS", subtitle: "Rust · Local chess", start_game: "Start game", settings: "Settings",
    exit: "Exit", attribution: "Pieces: chess-viewer · CBurnett", mode: "Game mode",
    difficulty: "AI difficulty", resolution: "Window resolution", refresh_rate: "Refresh rate",
    theme: "Theme", language: "Language", board_view: "Flip board when playing Black", back_to_menu: "Back to menu",
    two_players: "Two players", play_white: "Vs AI · Play White", play_black: "Vs AI · Play Black",
    beginner: "Beginner", easy: "Easy", medium: "Medium", hard: "Hard", master: "Master",
    flip_on: "On", flip_off: "Off", new_game: "New game", undo: "Undo", menu: "Menu",
    hint: "Hint", hint_thinking: "Analysing…", suggested: "Suggested", moves: "Moves",
    ai_thinking: "Computer thinking…", white: "White", black: "Black", wins: "wins", checkmate: "Checkmate! ",
    draw: "Draw", to_move: " to move", check: " (check!)", game_over: "Game over", play_again: "Play again",
    promotion: "Choose promotion", queen: "Queen", rook: "Rook", bishop: "Bishop", knight: "Knight",
};

static FR: Text = Text {
    title: "RUST CHESS", subtitle: "Rust · Partie locale", start_game: "Commencer", settings: "Paramètres",
    exit: "Quitter", attribution: "Pièces : chess-viewer · CBurnett", mode: "Mode de jeu",
    difficulty: "Difficulté de l’IA", resolution: "Résolution", refresh_rate: "Fréquence",
    theme: "Thème", language: "Langue", board_view: "Retourner avec les Noirs", back_to_menu: "Retour au menu",
    two_players: "Deux joueurs", play_white: "Contre l’IA · Blancs", play_black: "Contre l’IA · Noirs",
    beginner: "Débutant", easy: "Facile", medium: "Moyen", hard: "Difficile", master: "Maître",
    flip_on: "Activé", flip_off: "Désactivé", new_game: "Nouvelle partie", undo: "Annuler", menu: "Menu",
    hint: "Conseil", hint_thinking: "Analyse…", suggested: "Conseil", moves: "Coups",
    ai_thinking: "L’ordinateur réfléchit…", white: "Les Blancs", black: "Les Noirs", wins: " gagnent", checkmate: "Échec et mat ! ",
    draw: "Partie nulle", to_move: " jouent", check: " (échec !)", game_over: "Partie terminée", play_again: "Rejouer",
    promotion: "Choisir la promotion", queen: "Dame", rook: "Tour", bishop: "Fou", knight: "Cavalier",
};

static ES: Text = Text {
    title: "RUST CHESS", subtitle: "Rust · Partida local", start_game: "Jugar", settings: "Configuración",
    exit: "Salir", attribution: "Piezas: chess-viewer · CBurnett", mode: "Modo de juego",
    difficulty: "Dificultad de IA", resolution: "Resolución", refresh_rate: "Frecuencia",
    theme: "Tema", language: "Idioma", board_view: "Girar al jugar con negras", back_to_menu: "Volver al menú",
    two_players: "Dos jugadores", play_white: "Contra IA · Blancas", play_black: "Contra IA · Negras",
    beginner: "Principiante", easy: "Fácil", medium: "Medio", hard: "Difícil", master: "Maestro",
    flip_on: "Activado", flip_off: "Desactivado", new_game: "Nueva partida", undo: "Deshacer", menu: "Menú",
    hint: "Sugerencia", hint_thinking: "Analizando…", suggested: "Sugerencia", moves: "Jugadas",
    ai_thinking: "La computadora piensa…", white: "Blancas", black: "Negras", wins: " ganan", checkmate: "¡Jaque mate! ",
    draw: "Tablas", to_move: " juegan", check: " (¡jaque!)", game_over: "Fin de la partida", play_again: "Jugar de nuevo",
    promotion: "Elegir promoción", queen: "Dama", rook: "Torre", bishop: "Alfil", knight: "Caballo",
};

static LA: Text = Text {
    title: "RUST CHESS", subtitle: "Rust · Ludus localis", start_game: "Ludum incipe", settings: "Optiones",
    exit: "Exire", attribution: "Figurae: chess-viewer · CBurnett", mode: "Modus ludi",
    difficulty: "Difficultas machinae", resolution: "Resolutio fenestrae", refresh_rate: "Frequentia",
    theme: "Thema", language: "Lingua", board_view: "Tabulam verte cum Nigris", back_to_menu: "Ad indicem",
    two_players: "Duo lusores", play_white: "Contra machinam · Albi", play_black: "Contra machinam · Nigri",
    beginner: "Tiro", easy: "Facilis", medium: "Media", hard: "Difficilis", master: "Magister",
    flip_on: "Actum", flip_off: "Inactum", new_game: "Ludus novus", undo: "Revoca", menu: "Index",
    hint: "Consilium", hint_thinking: "Computatur…", suggested: "Consilium", moves: "Motus",
    ai_thinking: "Machina cogitat…", white: "Albi", black: "Nigri", wins: " vincunt", checkmate: "Rex captus! ",
    draw: "Aequitas", to_move: " movent", check: " (rex petitus!)", game_over: "Ludus finitus", play_again: "Iterum lude",
    promotion: "Promotionem elige", queen: "Regina", rook: "Turris", bishop: "Episcopus", knight: "Eques",
};

#[cfg(test)]
mod tests {
    use super::Language;

    #[test]
    fn language_keys_round_trip() {
        for language in Language::ALL {
            assert_eq!(Language::from_key(language.key()), Some(language));
            assert!(!language.native_name().is_empty());
            assert!(!language.text().title.is_empty());
        }
    }
}
