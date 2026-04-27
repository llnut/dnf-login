use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    English,
    #[default]
    ZhCn,
    ZhTw,
    Ja,
    Ko,
}

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::ZhCn => "简体中文",
            Language::ZhTw => "繁體中文",
            Language::Ja => "日本語",
            Language::Ko => "한국어",
        }
    }

    pub fn all() -> &'static [Language] {
        &[
            Language::English,
            Language::ZhCn,
            Language::ZhTw,
            Language::Ja,
            Language::Ko,
        ]
    }
}

/// All UI strings for a single locale.
#[derive(Clone, Copy)]
pub struct Tr {
    // App header
    pub app_title: &'static str,
    pub app_subtitle: &'static str,

    // Common
    pub username: &'static str,
    pub password: &'static str,
    pub confirm_password: &'static str,
    pub new_password: &'static str,
    pub back: &'static str,

    // Input hints
    pub hint_username: &'static str,
    pub hint_password: &'static str,

    // Login screen
    pub remember_password: &'static str,
    pub enter_game: &'static str,
    pub signing_in: &'static str,
    pub register_link: &'static str,
    pub change_password_link: &'static str,
    pub settings_link: &'static str,
    pub warn_server_not_configured: &'static str,

    // Register screen
    pub create_account_title: &'static str,
    pub qq_optional: &'static str,
    pub hint_choose_username: &'static str,
    pub hint_choose_password: &'static str,
    pub hint_re_enter_password: &'static str,
    pub hint_qq: &'static str,
    pub register_btn: &'static str,

    // Change Password screen
    pub change_password_title: &'static str,
    pub current_password: &'static str,
    pub confirm_new_password: &'static str,
    pub change_password_btn: &'static str,
    pub hint_current_password: &'static str,
    pub hint_enter_new_password: &'static str,
    pub hint_confirm_new_password: &'static str,

    // Background fill mode
    pub bg_fill_mode_label: &'static str,
    pub bg_fill_tile: &'static str,
    pub bg_fill_stretch: &'static str,
    pub bg_fill_fill: &'static str,
    pub bg_fill_center: &'static str,
    pub bg_fill_fit: &'static str,

    // Background source toggle pill
    pub bg_toggle_image: &'static str,
    pub bg_toggle_video: &'static str,

    // Game server IP
    pub game_server_ip_label: &'static str,
    pub game_server_ip_help: &'static str,

    // Plugin path
    pub plugins_path_label: &'static str,
    pub plugins_path_hint: &'static str,
    pub plugins_path_help: &'static str,
    pub plugin_inject_label: &'static str,
    pub plugin_inject_help: &'static str,

    // Background folders
    pub bg_pic_path_label: &'static str,
    pub bg_pic_path_hint: &'static str,
    pub bg_pic_path_help: &'static str,
    pub bg_vid_path_label: &'static str,
    pub bg_vid_path_hint: &'static str,
    pub bg_vid_path_help: &'static str,
    pub bg_reload_btn: &'static str,
    pub bg_reload_empty: &'static str,

    // Settings screen
    pub settings_title: &'static str,
    pub server_url_label: &'static str,
    pub server_url_hint: &'static str,
    pub server_url_help: &'static str,
    pub aes_key_label: &'static str,
    pub aes_key_hint: &'static str,
    pub aes_key_help: &'static str,
    pub saved_config_label: &'static str,
    pub not_configured: &'static str,
    pub clear_btn: &'static str,
    pub save_btn: &'static str,
    pub language_label: &'static str,
    pub warn_first_launch: &'static str,
    pub settings_saved: &'static str,

    // Validation errors
    pub err_server_not_configured: &'static str,
    pub err_enter_username_password: &'static str,
    pub err_enter_username: &'static str,
    pub err_enter_password: &'static str,
    pub err_passwords_no_match: &'static str,
    pub err_enter_old_password: &'static str,
    pub err_enter_new_password: &'static str,
    pub err_client_not_init: &'static str,

    // Server-side validation/error messages
    pub err_server_invalid_username: &'static str,
    pub err_server_invalid_password: &'static str,
    pub err_server_invalid_qq: &'static str,
    pub err_server_user_exists: &'static str,
    pub err_server_wrong_credentials: &'static str,
    pub err_server_account_banned: &'static str,
    pub err_server_unknown: &'static str,

    // Dynamic error prefixes (caller appends ": " + detail)
    pub err_mac_prefix: &'static str,
    pub err_config_prefix: &'static str,
    pub err_save_prefix: &'static str,
    pub err_network_prefix: &'static str,
    pub err_launch_prefix: &'static str,

    // Success messages
    pub login_success: &'static str,
    pub register_success: &'static str,
    pub change_password_success: &'static str,

    // Confirm close dialog
    pub confirm_close_dnf_title: &'static str,
    pub confirm_close_dnf_msg: &'static str,
    pub confirm_close_yes: &'static str,
    pub confirm_close_no: &'static str,
    pub closing_dnf: &'static str,

    // About screen
    pub about_link: &'static str,
    pub about_title: &'static str,
    pub about_launcher_name_label: &'static str,
    pub about_version_label: &'static str,
    pub about_repo_label: &'static str,
    pub about_author_label: &'static str,
}

pub fn translations(lang: Language) -> Tr {
    match lang {
        Language::English => EN,
        Language::ZhCn => ZH_CN,
        Language::ZhTw => ZH_TW,
        Language::Ja => JA,
        Language::Ko => KO,
    }
}

// English
const EN: Tr = Tr {
    app_title: "DNF Launcher",
    app_subtitle: "DUNGEON & FIGHTER",

    username: "USERNAME",
    password: "PASSWORD",
    confirm_password: "CONFIRM PASSWORD",
    new_password: "NEW PASSWORD",
    back: "\u{2190} Back",

    hint_username: "Enter username",
    hint_password: "Enter password",

    remember_password: "Remember password",
    enter_game: "ENTER GAME",
    signing_in: "Signing in\u{2026}",
    register_link: "Register",
    change_password_link: "Change Password",
    settings_link: "Settings",
    warn_server_not_configured: "Server not configured \u{2014} open Settings to connect.",

    create_account_title: "Create Account",
    qq_optional: "QQ NUMBER  (optional)",
    hint_choose_username: "Enter username",
    hint_choose_password: "Enter password",
    hint_re_enter_password: "Re-enter password",
    hint_qq: "Enter QQ number",
    register_btn: "REGISTER",

    change_password_title: "Change Password",
    current_password: "CURRENT PASSWORD",
    confirm_new_password: "CONFIRM NEW PASSWORD",
    change_password_btn: "CHANGE PASSWORD",
    hint_current_password: "Enter current password",
    hint_enter_new_password: "Enter new password",
    hint_confirm_new_password: "Re-enter new password",

    bg_fill_mode_label: "FILL MODE",
    bg_fill_tile: "Tile",
    bg_fill_stretch: "Stretch",
    bg_fill_fill: "Fill",
    bg_fill_center: "Center",
    bg_fill_fit: "Fit",

    bg_toggle_image: "Image",
    bg_toggle_video: "Video",

    game_server_ip_label: "Set GAME_SERVER_IP on launch",
    game_server_ip_help: "When enabled, the launcher fetches the game server IP and passes it to DNF.exe via the GAME_SERVER_IP environment variable.",

    plugins_path_label: "PLUGIN PATH",
    plugins_path_hint: "e.g. plugins",
    plugins_path_help: "DLL files in this folder are injected into DNF.exe at launch. Path is relative to the launcher executable.",
    plugin_inject_label: "Enable DLL injection",
    plugin_inject_help: "When enabled, DLL files in the plugin path are injected into DNF.exe at launch.",

    bg_pic_path_label: "PICTURE FOLDER",
    bg_pic_path_hint: "e.g. assets/pic",
    bg_pic_path_help: "All JPG files in this folder are used as wallpapers. Path is relative to the launcher executable.",
    bg_vid_path_label: "VIDEO FOLDER",
    bg_vid_path_hint: "e.g. assets/vid",
    bg_vid_path_help: "All WebP and MP4 files in this folder are used as background videos. Path is relative to the launcher executable.",
    bg_reload_btn: "RELOAD",
    bg_reload_empty: "No images or videos found in the configured folders.",

    settings_title: "Settings",
    server_url_label: "SERVER URL",
    server_url_hint: "e.g. http://192.168.200.131:5505",
    server_url_help: "Contact the server administrator for the connection address.",
    aes_key_label: "AES KEY",
    aes_key_hint: "64 hexadecimal characters (32 bytes)",
    aes_key_help: "Must be exactly 64 hex characters (0\u{2013}9, a\u{2013}f), representing 32 bytes.",
    saved_config_label: "SAVED CONFIGURATION",
    not_configured: "Not configured",
    clear_btn: "Clear",
    save_btn: "SAVE",
    language_label: "LANGUAGE",
    warn_first_launch: "First launch \u{2014} enter server address and key in Settings.",
    settings_saved: "Settings saved.",

    err_server_not_configured: "Server not configured. Please set the URL and key in Settings.",
    err_enter_username_password: "Please enter your username and password.",
    err_enter_username: "Please enter a username.",
    err_enter_password: "Please enter a password.",
    err_passwords_no_match: "Passwords do not match.",
    err_enter_old_password: "Please enter your current password.",
    err_enter_new_password: "Please enter a new password.",
    err_client_not_init: "Client not initialized. Please save server settings first.",

    err_server_invalid_username: "Username must be 4\u{2013}32 characters, using only letters, numbers, and underscores.",
    err_server_invalid_password: "Invalid password format.",
    err_server_invalid_qq: "QQ number must be 5\u{2013}12 digits.",
    err_server_user_exists: "This username is already taken.",
    err_server_wrong_credentials: "Incorrect username or password.",
    err_server_account_banned: "This account has been banned.",
    err_server_unknown: "Operation failed. Please try again.",

    err_mac_prefix: "Failed to get MAC address",
    err_config_prefix: "Invalid configuration",
    err_save_prefix: "Failed to save",
    err_network_prefix: "Network error",
    err_launch_prefix: "Failed to launch game",

    login_success: "Login successful. Launching game\u{2026}",
    register_success: "Registration successful. Please log in.",
    change_password_success: "Password changed successfully. Please log in again.",

    confirm_close_dnf_title: "Game Already Running",
    confirm_close_dnf_msg: "DNF.exe is already running.\nClose it and launch the game?",
    confirm_close_yes: "CLOSE AND CONTINUE",
    confirm_close_no: "Cancel",
    closing_dnf: "Closing DNF.exe\u{2026}",

    about_link: "About",
    about_title: "About",
    about_launcher_name_label: "LAUNCHER",
    about_version_label: "VERSION",
    about_repo_label: "REPOSITORY",
    about_author_label: "AUTHOR",
};

// Simplified Chinese
const ZH_CN: Tr = Tr {
    app_title: "DNF 启动器",
    app_subtitle: "地下城与勇士",

    username: "用户名",
    password: "密码",
    confirm_password: "确认密码",
    new_password: "新密码",
    back: "\u{2190} 返回",

    hint_username: "输入用户名",
    hint_password: "输入密码",

    remember_password: "记住密码",
    enter_game: "进入游戏",
    signing_in: "登录中\u{2026}",
    register_link: "注册",
    change_password_link: "修改密码",
    settings_link: "设置",
    warn_server_not_configured: "服务器未配置，请前往设置填写连接信息。",

    create_account_title: "创建账号",
    qq_optional: "QQ 号（选填）",
    hint_choose_username: "输入用户名",
    hint_choose_password: "输入密码",
    hint_re_enter_password: "再次输入密码",
    hint_qq: "输入 QQ 号",
    register_btn: "立即注册",

    change_password_title: "修改密码",
    current_password: "原密码",
    confirm_new_password: "确认新密码",
    change_password_btn: "确认修改",
    hint_current_password: "输入原密码",
    hint_enter_new_password: "输入新密码",
    hint_confirm_new_password: "再次输入新密码",

    bg_fill_mode_label: "显示方式",
    bg_fill_tile: "平铺",
    bg_fill_stretch: "拉伸",
    bg_fill_fill: "填充",
    bg_fill_center: "居中",
    bg_fill_fit: "适应",

    bg_toggle_image: "图片",
    bg_toggle_video: "视频",

    game_server_ip_label: "启动时设置 GAME_SERVER_IP",
    game_server_ip_help: "开启后，启动器将获取游戏服务器 IP 并通过环境变量 GAME_SERVER_IP 传递给 DNF.exe。",

    plugins_path_label: "插件路径",
    plugins_path_hint: "例：plugins",
    plugins_path_help: "该目录中的 DLL 文件将在游戏启动时注入 DNF.exe，路径相对于启动器所在目录。",
    plugin_inject_label: "启用 DLL 注入",
    plugin_inject_help: "开启后，游戏启动时将自动把插件路径中的 DLL 文件注入 DNF.exe。",

    bg_pic_path_label: "图片目录",
    bg_pic_path_hint: "例：assets/pic",
    bg_pic_path_help: "此目录下的所有 JPG 文件都会用作背景，路径相对于程序所在目录。",
    bg_vid_path_label: "视频目录",
    bg_vid_path_hint: "例：assets/vid",
    bg_vid_path_help: "此目录下的所有 WebP 和 MP4 文件都会用作背景视频，路径相对于程序所在目录。",
    bg_reload_btn: "重新加载",
    bg_reload_empty: "指定目录下没有找到任何图片或视频。",

    settings_title: "设置",
    server_url_label: "服务器地址",
    server_url_hint: "例：http://192.168.200.131:5505",
    server_url_help: "请向服务器管理员获取连接地址。",
    aes_key_label: "AES 密钥",
    aes_key_hint: "64 位十六进制字符（32 字节）",
    aes_key_help: "格式：64 个十六进制字符（0\u{2013}9，a\u{2013}f），对应 32 字节。",
    saved_config_label: "已保存的配置",
    not_configured: "尚未配置",
    clear_btn: "清除",
    save_btn: "保存",
    language_label: "语言",
    warn_first_launch: "首次使用，请在此填写服务器地址和密钥。",
    settings_saved: "设置已保存。",

    err_server_not_configured: "服务器未配置，请在设置中填写地址和密钥。",
    err_enter_username_password: "请输入用户名和密码。",
    err_enter_username: "请输入用户名。",
    err_enter_password: "请输入密码。",
    err_passwords_no_match: "两次输入的密码不一致。",
    err_enter_old_password: "请输入原密码。",
    err_enter_new_password: "请输入新密码。",
    err_client_not_init: "客户端未初始化，请先保存服务器设置。",

    err_server_invalid_username: "用户名须为 4\u{2013}32 个字符，仅支持字母、数字和下划线。",
    err_server_invalid_password: "密码格式无效。",
    err_server_invalid_qq: "QQ 号须为 5\u{2013}12 位数字。",
    err_server_user_exists: "该用户名已被注册。",
    err_server_wrong_credentials: "用户名或密码错误。",
    err_server_account_banned: "该账号已被封禁。",
    err_server_unknown: "操作失败，请重试。",

    err_mac_prefix: "获取 MAC 地址失败",
    err_config_prefix: "配置无效",
    err_save_prefix: "保存失败",
    err_network_prefix: "网络错误",
    err_launch_prefix: "游戏启动失败",

    login_success: "登录成功，正在启动游戏\u{2026}",
    register_success: "注册成功，请登录。",
    change_password_success: "密码已修改，请重新登录。",

    confirm_close_dnf_title: "游戏正在运行",
    confirm_close_dnf_msg: "DNF.exe 正在运行，\n是否关闭后继续启动？",
    confirm_close_yes: "关闭并继续",
    confirm_close_no: "取消",
    closing_dnf: "正在关闭 DNF.exe\u{2026}",

    about_link: "关于",
    about_title: "关于",
    about_launcher_name_label: "启动器",
    about_version_label: "版本",
    about_repo_label: "仓库",
    about_author_label: "作者",
};

// Traditional Chinese
const ZH_TW: Tr = Tr {
    app_title: "DNF 啟動器",
    app_subtitle: "地下城與勇士",

    username: "帳號",
    password: "密碼",
    confirm_password: "確認密碼",
    new_password: "新密碼",
    back: "\u{2190} 返回",

    hint_username: "輸入帳號",
    hint_password: "輸入密碼",

    remember_password: "記住密碼",
    enter_game: "進入遊戲",
    signing_in: "登入中\u{2026}",
    register_link: "註冊",
    change_password_link: "修改密碼",
    settings_link: "設定",
    warn_server_not_configured: "伺服器未設定，請前往設定填寫連線資訊。",

    create_account_title: "建立帳號",
    qq_optional: "QQ 號碼（選填）",
    hint_choose_username: "輸入帳號",
    hint_choose_password: "輸入密碼",
    hint_re_enter_password: "再次輸入密碼",
    hint_qq: "輸入 QQ 號碼",
    register_btn: "立即註冊",

    change_password_title: "修改密碼",
    current_password: "舊密碼",
    confirm_new_password: "確認新密碼",
    change_password_btn: "確認修改",
    hint_current_password: "輸入舊密碼",
    hint_enter_new_password: "輸入新密碼",
    hint_confirm_new_password: "再次輸入新密碼",

    bg_fill_mode_label: "顯示方式",
    bg_fill_tile: "並排",
    bg_fill_stretch: "延展",
    bg_fill_fill: "填滿",
    bg_fill_center: "置中",
    bg_fill_fit: "適合",

    bg_toggle_image: "圖片",
    bg_toggle_video: "影片",

    game_server_ip_label: "啟動時設定 GAME_SERVER_IP",
    game_server_ip_help: "啟用後，啟動器會取得遊戲伺服器 IP，並透過環境變數 GAME_SERVER_IP 傳遞給 DNF.exe。",

    plugins_path_label: "插件路徑",
    plugins_path_hint: "例：plugins",
    plugins_path_help: "該目錄中的 DLL 檔案將在遊戲啟動時注入 DNF.exe，路徑相對於啟動器所在目錄。",
    plugin_inject_label: "啟用 DLL 注入",
    plugin_inject_help: "開啟後，遊戲啟動時將自動把插件路徑中的 DLL 檔案注入 DNF.exe。",

    bg_pic_path_label: "圖片資料夾",
    bg_pic_path_hint: "例：assets/pic",
    bg_pic_path_help: "此資料夾中的所有 JPG 檔案皆會作為背景，路徑相對於程式所在目錄。",
    bg_vid_path_label: "影片資料夾",
    bg_vid_path_hint: "例：assets/vid",
    bg_vid_path_help: "此資料夾中的所有 WebP 與 MP4 檔案皆會作為背景影片，路徑相對於程式所在目錄。",
    bg_reload_btn: "重新載入",
    bg_reload_empty: "指定資料夾中找不到任何圖片或影片。",

    settings_title: "設定",
    server_url_label: "伺服器位址",
    server_url_hint: "例：http://192.168.200.131:5505",
    server_url_help: "請向伺服器管理員取得連線位址。",
    aes_key_label: "AES 金鑰",
    aes_key_hint: "64 位十六進位字元（32 位元組）",
    aes_key_help: "格式：64 個十六進位字元（0\u{2013}9，a\u{2013}f），對應 32 位元組。",
    saved_config_label: "已儲存的設定",
    not_configured: "尚未設定",
    clear_btn: "清除",
    save_btn: "儲存",
    language_label: "語言",
    warn_first_launch: "首次使用，請在此填寫伺服器位址與金鑰。",
    settings_saved: "設定已儲存。",

    err_server_not_configured: "伺服器未設定，請在設定頁填寫位址與金鑰。",
    err_enter_username_password: "請輸入帳號與密碼。",
    err_enter_username: "請輸入帳號。",
    err_enter_password: "請輸入密碼。",
    err_passwords_no_match: "兩次輸入的密碼不一致。",
    err_enter_old_password: "請輸入舊密碼。",
    err_enter_new_password: "請輸入新密碼。",
    err_client_not_init: "用戶端未初始化，請先儲存伺服器設定。",

    err_server_invalid_username: "帳號須為 4\u{2013}32 個字元，僅支援字母、數字及底線。",
    err_server_invalid_password: "密碼格式無效。",
    err_server_invalid_qq: "QQ 號碼須為 5\u{2013}12 位數字。",
    err_server_user_exists: "此帳號已被註冊。",
    err_server_wrong_credentials: "帳號或密碼錯誤。",
    err_server_account_banned: "此帳號已被停權。",
    err_server_unknown: "操作失敗，請重試。",

    err_mac_prefix: "取得 MAC 位址失敗",
    err_config_prefix: "設定無效",
    err_save_prefix: "儲存失敗",
    err_network_prefix: "網路錯誤",
    err_launch_prefix: "遊戲啟動失敗",

    login_success: "登入成功，正在啟動遊戲\u{2026}",
    register_success: "註冊成功，請登入。",
    change_password_success: "密碼已修改，請重新登入。",

    confirm_close_dnf_title: "遊戲正在執行",
    confirm_close_dnf_msg: "DNF.exe 正在執行，\n是否關閉後繼續啟動？",
    confirm_close_yes: "關閉並繼續",
    confirm_close_no: "取消",
    closing_dnf: "正在關閉 DNF.exe\u{2026}",

    about_link: "關於",
    about_title: "關於",
    about_launcher_name_label: "啟動器",
    about_version_label: "版本",
    about_repo_label: "儲存庫",
    about_author_label: "作者",
};

// Japanese
const JA: Tr = Tr {
    app_title: "DNF ランチャー",
    app_subtitle: "ダンジョン＆ファイター",

    username: "ユーザー名",
    password: "パスワード",
    confirm_password: "パスワード（確認）",
    new_password: "新しいパスワード",
    back: "\u{2190} 戻る",

    hint_username: "ユーザー名を入力",
    hint_password: "パスワードを入力",

    remember_password: "パスワードを記憶",
    enter_game: "ゲームを起動",
    signing_in: "ログイン中\u{2026}",
    register_link: "新規登録",
    change_password_link: "パスワード変更",
    settings_link: "設定",
    warn_server_not_configured: "サーバーが未設定です。設定を開いて接続情報を入力してください。",

    create_account_title: "アカウント作成",
    qq_optional: "QQ番号（任意）",
    hint_choose_username: "ユーザー名を入力",
    hint_choose_password: "パスワードを入力",
    hint_re_enter_password: "パスワードをもう一度入力",
    hint_qq: "QQ番号を入力",
    register_btn: "登録する",

    change_password_title: "パスワード変更",
    current_password: "現在のパスワード",
    confirm_new_password: "新パスワード（確認）",
    change_password_btn: "変更する",
    hint_current_password: "現在のパスワードを入力",
    hint_enter_new_password: "新しいパスワードを入力",
    hint_confirm_new_password: "新しいパスワードをもう一度入力",

    bg_fill_mode_label: "表示方法",
    bg_fill_tile: "並べて表示",
    bg_fill_stretch: "拡大して表示",
    bg_fill_fill: "フィル",
    bg_fill_center: "中央に表示",
    bg_fill_fit: "画面に合わせる",

    bg_toggle_image: "画像",
    bg_toggle_video: "動画",

    game_server_ip_label: "起動時に GAME_SERVER_IP を設定",
    game_server_ip_help: "有効にすると、ランチャーがゲームサーバーの IP を取得し、環境変数 GAME_SERVER_IP として DNF.exe に渡します。",

    plugins_path_label: "プラグインパス",
    plugins_path_hint: "例：plugins",
    plugins_path_help: "フォルダ内の DLL ファイルは、ゲーム起動時に DNF.exe へ注入されます。パスはランチャー実行ファイルからの相対パスです。",
    plugin_inject_label: "DLL インジェクションを有効化",
    plugin_inject_help: "有効にすると、ゲーム起動後にプラグインパス内の DLL ファイルが DNF.exe へ自動的に注入されます。",

    bg_pic_path_label: "画像フォルダ",
    bg_pic_path_hint: "例：assets/pic",
    bg_pic_path_help: "フォルダ内のすべての JPG ファイルが背景として使われます。パスは実行ファイルからの相対パスです。",
    bg_vid_path_label: "動画フォルダ",
    bg_vid_path_hint: "例：assets/vid",
    bg_vid_path_help: "フォルダ内のすべての WebP と MP4 ファイルが背景動画として使われます。パスは実行ファイルからの相対パスです。",
    bg_reload_btn: "再読み込み",
    bg_reload_empty: "設定したフォルダに画像も動画も見つかりませんでした。",

    settings_title: "設定",
    server_url_label: "サーバーURL",
    server_url_hint: "例：http://192.168.200.131:5505",
    server_url_help: "接続先アドレスはサーバー管理者にお問い合わせください。",
    aes_key_label: "AESキー",
    aes_key_hint: "16進数64文字（32バイト）",
    aes_key_help: "形式：16進数64文字（0\u{2013}9\u{3001}a\u{2013}f）、32バイト。",
    saved_config_label: "保存済み設定",
    not_configured: "未設定",
    clear_btn: "クリア",
    save_btn: "保存",
    language_label: "言語",
    warn_first_launch: "初回起動です。設定からサーバー接続情報を入力してください。",
    settings_saved: "設定を保存しました。",

    err_server_not_configured: "サーバーが設定されていません。設定画面でURLとキーを入力してください。",
    err_enter_username_password: "ユーザー名とパスワードを入力してください。",
    err_enter_username: "ユーザー名を入力してください。",
    err_enter_password: "パスワードを入力してください。",
    err_passwords_no_match: "パスワードが一致しません。",
    err_enter_old_password: "現在のパスワードを入力してください。",
    err_enter_new_password: "新しいパスワードを入力してください。",
    err_client_not_init: "クライアントが初期化されていません。先に設定を保存してください。",

    err_server_invalid_username: "ユーザー名は4\u{301c}32文字で、英数字とアンダースコアのみ使用できます。",
    err_server_invalid_password: "パスワードの形式が正しくありません。",
    err_server_invalid_qq: "QQ番号は5\u{301c}12桁の数字で入力してください。",
    err_server_user_exists: "このユーザー名はすでに使用されています。",
    err_server_wrong_credentials: "ユーザー名またはパスワードが正しくありません。",
    err_server_account_banned: "このアカウントは利用停止されています。",
    err_server_unknown: "操作に失敗しました。もう一度お試しください。",

    err_mac_prefix: "MACアドレス取得失敗",
    err_config_prefix: "設定エラー",
    err_save_prefix: "保存失敗",
    err_network_prefix: "ネットワークエラー",
    err_launch_prefix: "ゲーム起動失敗",

    login_success: "ログイン成功。ゲームを起動中\u{2026}",
    register_success: "登録完了。ログインしてください。",
    change_password_success: "パスワードを変更しました。再度ログインしてください。",

    confirm_close_dnf_title: "ゲーム実行中",
    confirm_close_dnf_msg: "DNF.exe が実行中です。\n終了してゲームを起動しますか？",
    confirm_close_yes: "終了して続行",
    confirm_close_no: "キャンセル",
    closing_dnf: "DNF.exe を終了中\u{2026}",

    about_link: "バージョン情報",
    about_title: "バージョン情報",
    about_launcher_name_label: "ランチャー名",
    about_version_label: "バージョン",
    about_repo_label: "リポジトリ",
    about_author_label: "作者",
};

// Korean
const KO: Tr = Tr {
    app_title: "DNF 런처",
    app_subtitle: "던전 앤 파이터",

    username: "아이디",
    password: "비밀번호",
    confirm_password: "비밀번호 확인",
    new_password: "새 비밀번호",
    back: "\u{2190} 뒤로",

    hint_username: "아이디 입력",
    hint_password: "비밀번호 입력",

    remember_password: "비밀번호 저장",
    enter_game: "게임 시작",
    signing_in: "로그인 중\u{2026}",
    register_link: "회원가입",
    change_password_link: "비밀번호 변경",
    settings_link: "설정",
    warn_server_not_configured: "서버가 설정되지 않았습니다. 설정에서 연결 정보를 입력해 주세요.",

    create_account_title: "계정 만들기",
    qq_optional: "QQ 번호 (선택)",
    hint_choose_username: "아이디 입력",
    hint_choose_password: "비밀번호 입력",
    hint_re_enter_password: "비밀번호 재입력",
    hint_qq: "QQ 번호 입력",
    register_btn: "가입하기",

    change_password_title: "비밀번호 변경",
    current_password: "기존 비밀번호",
    confirm_new_password: "새 비밀번호 확인",
    change_password_btn: "변경하기",
    hint_current_password: "기존 비밀번호 입력",
    hint_enter_new_password: "새 비밀번호 입력",
    hint_confirm_new_password: "새 비밀번호 재입력",

    bg_fill_mode_label: "채우기 방식",
    bg_fill_tile: "바둑판식",
    bg_fill_stretch: "늘이기",
    bg_fill_fill: "채우기",
    bg_fill_center: "가운데",
    bg_fill_fit: "화면에 맞춤",

    bg_toggle_image: "사진",
    bg_toggle_video: "영상",

    game_server_ip_label: "실행 시 GAME_SERVER_IP 설정",
    game_server_ip_help: "활성화하면 런처가 게임 서버 IP를 가져와 환경 변수 GAME_SERVER_IP로 DNF.exe에 전달합니다.",

    plugins_path_label: "플러그인 경로",
    plugins_path_hint: "예: plugins",
    plugins_path_help: "이 폴더의 DLL 파일은 게임 실행 시 DNF.exe에 주입됩니다. 경로는 런처 실행 파일 기준 상대 경로입니다.",
    plugin_inject_label: "DLL 주입 사용",
    plugin_inject_help: "활성화하면 게임 실행 후 플러그인 경로의 DLL 파일이 DNF.exe에 자동으로 주입됩니다.",

    bg_pic_path_label: "이미지 폴더",
    bg_pic_path_hint: "예: assets/pic",
    bg_pic_path_help: "폴더 내 모든 JPG 파일이 배경으로 사용됩니다. 경로는 실행 파일 기준 상대 경로입니다.",
    bg_vid_path_label: "동영상 폴더",
    bg_vid_path_hint: "예: assets/vid",
    bg_vid_path_help: "폴더 내 모든 WebP 와 MP4 파일이 배경 동영상으로 사용됩니다. 경로는 실행 파일 기준 상대 경로입니다.",
    bg_reload_btn: "다시 불러오기",
    bg_reload_empty: "설정한 폴더에서 이미지와 동영상을 찾을 수 없습니다.",

    settings_title: "설정",
    server_url_label: "서버 주소",
    server_url_hint: "예: http://192.168.200.131:5505",
    server_url_help: "서버 관리자에게 연결 주소를 문의하세요.",
    aes_key_label: "AES 키",
    aes_key_hint: "16진수 64자리 (32바이트)",
    aes_key_help: "형식: 16진수 64자리 (0\u{2013}9, a\u{2013}f), 32바이트.",
    saved_config_label: "저장된 설정",
    not_configured: "미설정",
    clear_btn: "지우기",
    save_btn: "저장",
    language_label: "언어",
    warn_first_launch: "처음 실행입니다. 설정에서 서버 주소와 키를 입력해 주세요.",
    settings_saved: "설정이 저장되었습니다.",

    err_server_not_configured: "서버가 설정되지 않았습니다. 설정에서 주소와 키를 입력해 주세요.",
    err_enter_username_password: "아이디와 비밀번호를 입력해 주세요.",
    err_enter_username: "아이디를 입력해 주세요.",
    err_enter_password: "비밀번호를 입력해 주세요.",
    err_passwords_no_match: "비밀번호가 일치하지 않습니다.",
    err_enter_old_password: "기존 비밀번호를 입력해 주세요.",
    err_enter_new_password: "새 비밀번호를 입력해 주세요.",
    err_client_not_init: "클라이언트가 초기화되지 않았습니다. 먼저 서버 설정을 저장해 주세요.",

    err_server_invalid_username: "아이디는 4~32자이며, 영문, 숫자, 밑줄만 사용할 수 있습니다.",
    err_server_invalid_password: "비밀번호 형식이 올바르지 않습니다.",
    err_server_invalid_qq: "QQ 번호는 5~12자리 숫자로 입력해 주세요.",
    err_server_user_exists: "이미 사용 중인 아이디입니다.",
    err_server_wrong_credentials: "아이디 또는 비밀번호가 올바르지 않습니다.",
    err_server_account_banned: "이 계정은 이용이 정지되었습니다.",
    err_server_unknown: "작업에 실패했습니다. 다시 시도해 주세요.",

    err_mac_prefix: "MAC 주소 확인 실패",
    err_config_prefix: "설정 오류",
    err_save_prefix: "저장 실패",
    err_network_prefix: "네트워크 오류",
    err_launch_prefix: "게임 실행 실패",

    login_success: "로그인 성공. 게임을 시작합니다\u{2026}",
    register_success: "가입 완료. 로그인해 주세요.",
    change_password_success: "비밀번호가 변경되었습니다. 다시 로그인해 주세요.",

    confirm_close_dnf_title: "게임 실행 중",
    confirm_close_dnf_msg: "DNF.exe가 이미 실행 중입니다.\n종료 후 새로 시작하시겠습니까?",
    confirm_close_yes: "종료 후 계속",
    confirm_close_no: "취소",
    closing_dnf: "DNF.exe 종료 중\u{2026}",

    about_link: "정보",
    about_title: "정보",
    about_launcher_name_label: "런처",
    about_version_label: "버전",
    about_repo_label: "저장소",
    about_author_label: "제작자",
};
