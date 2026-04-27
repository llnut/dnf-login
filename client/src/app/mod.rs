use anyhow::Result;
use eframe::egui;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Instant;

use crate::{
    config::AppConfig, i18n::translations, network::DnfClient, storage::CredentialStorage,
};

mod bg;
mod handlers;
mod render;
mod screens;
mod theme;
mod video;
mod widgets;

use video::{VideoEntry, VideoLoadData, VideoWorkerHandle};

pub(super) const THUMB_W: u32 = 64;
pub(super) const THUMB_H: u32 = 36;

/// Decoded pixel data for a single background, produced by worker threads and
/// consumed on the main thread to upload GPU textures.
pub(super) struct BgImageData {
    pub(super) index: usize,
    pub(super) full_image: egui::ColorImage,
    pub(super) thumb_image: egui::ColorImage,
}

#[derive(PartialEq, Clone, Copy)]
pub(super) enum AppState {
    Login,
    Register,
    ChangePassword,
    Settings,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum TaskType {
    Login,
    Register,
    ChangePassword,
    CloseDnf,
}

pub(super) enum TaskResult {
    Login(Result<crate::network::LoginResponse>, Option<String>),
    Register(Result<crate::network::RegisterResponse>),
    ChangePassword(Result<crate::network::SimpleResponse>),
    CloseDnf(Result<()>),
}

pub(super) struct PendingLaunch {
    pub(super) token: String,
    pub(super) plugins_path: String,
    pub(super) inject_enabled: bool,
    pub(super) server_ip: String,
}

pub struct DnfLoginApp {
    pub(super) config: AppConfig,
    pub(super) client: Option<DnfClient>,
    pub(super) storage: CredentialStorage,
    pub(super) runtime: tokio::runtime::Runtime,
    pub(super) task_rx: Receiver<TaskResult>,
    pub(super) task_tx: Sender<TaskResult>,

    // Translation table for the active language; refreshed on language change.
    pub(super) tr: crate::i18n::Tr,
    // Application icon texture decoded from the embedded ICO file.
    pub(super) app_icon: Option<egui::TextureHandle>,

    // Full-resolution background textures scaled to the window size.
    pub(super) bgs: Vec<Option<egui::TextureHandle>>,
    // Thumbnail textures for the background switcher strip.
    pub(super) bg_thumbs: Vec<Option<egui::TextureHandle>>,
    // Index of the currently displayed background.
    pub(super) current_bg: usize,
    // Horizontal scroll offset (pixels) of the thumbnail strip.
    pub(super) thumb_scroll_offset: f32,
    // Scroll velocity of the thumbnail strip.
    pub(super) thumb_velocity: f32,
    // Whether a drag is active on the thumbnail strip.
    pub(super) thumb_drag_active: bool,
    // Total pointer movement during the current drag.
    pub(super) thumb_drag_distance: f32,
    // Receives decoded background images from worker threads for GPU upload.
    // `None` indicates a failed decode; one message is sent per task.
    pub(super) img_rx: Receiver<Option<BgImageData>>,
    // Number of background decode tasks still in flight.
    pub(super) bg_pending: usize,
    // Set to true after the first frame triggers background loading.
    pub(super) bg_loading_started: bool,

    pub(super) videos: Vec<VideoEntry>,
    pub(super) video_thumbs: Vec<Option<egui::TextureHandle>>,
    pub(super) video_texture: Option<egui::TextureHandle>,
    pub(super) video_worker: Option<VideoWorkerHandle>,
    // Index of the video the worker is currently decoding.
    pub(super) video_worker_idx: Option<usize>,
    pub(super) current_video: usize,
    // Wall-clock instant at which the next frame should be displayed.
    pub(super) next_video_frame_at: Option<Instant>,
    // Receives async-loaded video bytes and thumbnail for population.
    pub(super) video_load_rx: Receiver<Option<VideoLoadData>>,
    // Number of video load tasks still in flight.
    pub(super) video_pending: usize,
    // Set to true after the first frame triggers video loading.
    pub(super) video_loading_started: bool,

    pub(super) state: AppState,
    pub(super) current_task: Option<TaskType>,

    pub(super) settings_server_url: String,
    pub(super) settings_aes_key: String,
    pub(super) settings_pic_path: String,
    pub(super) settings_vid_path: String,
    pub(super) settings_plugins_path: String,
    pub(super) settings_plugin_inject_enabled: bool,
    pub(super) settings_game_server_ip_enabled: bool,

    pub(super) username: String,
    pub(super) password: String,
    pub(super) remember_password: bool,

    pub(super) register_username: String,
    pub(super) register_password: String,
    pub(super) register_password_confirm: String,
    pub(super) register_qq: String,

    pub(super) changepwd_username: String,
    pub(super) changepwd_old_password: String,
    pub(super) changepwd_new_password: String,
    pub(super) changepwd_confirm: String,

    pub(super) message: Option<String>,
    pub(super) message_is_error: bool,

    pub(super) show_confirm_close_dnf: bool,
    pub(super) pending_launch: Option<PendingLaunch>,
}

// Setup
impl DnfLoginApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::try_load_cjk_fonts(&cc.egui_ctx);

        let mut style = (*cc.egui_ctx.global_style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(14.5, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(14.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(22.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(11.5, egui::FontFamily::Proportional),
        );
        style.spacing.item_spacing = egui::vec2(8.0, 5.0);
        style.spacing.button_padding = egui::vec2(14.0, 6.0);
        style.spacing.text_edit_width = 220.0;
        cc.egui_ctx.set_global_style(style);

        cc.egui_ctx.set_theme(egui::ThemePreference::Dark);

        let mut v = egui::Visuals::dark();
        v.panel_fill = Self::c_bg();
        v.window_fill = Self::c_bg();
        v.extreme_bg_color = Self::c_input_bg();
        v.faint_bg_color = Self::c_card();

        v.widgets.noninteractive.bg_fill = Self::c_card();
        v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, Self::c_border());
        v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, Self::c_text2());

        v.widgets.inactive.bg_fill = Self::c_input_bg();
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, Self::c_border_dim());
        v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, Self::c_text2());

        v.widgets.hovered.bg_fill = egui::Color32::from_rgb(26, 30, 40);
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, Self::c_accent());
        v.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, Self::c_text());

        v.widgets.active.bg_fill = Self::c_accent_faint();
        v.widgets.active.bg_stroke = egui::Stroke::new(1.5, Self::c_accent());
        v.widgets.active.fg_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);

        v.selection.bg_fill = Self::c_accent_faint();
        v.selection.stroke = egui::Stroke::new(1.0, Self::c_accent());

        v.hyperlink_color = Self::c_accent();
        v.text_cursor.stroke.color = Self::c_accent();
        // Explicitly target the dark theme style so the override survives system-theme events.
        cc.egui_ctx.set_visuals_of(egui::Theme::Dark, v);

        let config = AppConfig::load().unwrap_or_else(|e| {
            tracing::warn!("Failed to load config, using default: {}", e);
            AppConfig::default()
        });

        let client: Option<DnfClient> = if config.is_configured() {
            match config.get_aes_key_bytes() {
                Ok(key) => DnfClient::new(config.server_url.clone(), &key).ok(),
                Err(e) => {
                    tracing::error!("AES key parse error: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let storage = CredentialStorage::new().expect("Failed to create credential storage");
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        let (task_tx, task_rx) = channel();

        let (username, password, remember_password) = if storage.has_saved_credentials() {
            if let Ok((u, p)) = storage.load() {
                (u, p, true)
            } else {
                (String::new(), String::new(), false)
            }
        } else {
            (String::new(), String::new(), false)
        };

        let settings_server_url = config.server_url.clone();
        let settings_aes_key = config.aes_key.clone();
        let settings_pic_path = config.bg_pic_path.clone();
        let settings_vid_path = config.bg_vid_path.clone();
        let settings_plugins_path = config.plugins_path.clone();
        let settings_plugin_inject_enabled = config.plugin_inject_enabled;
        let settings_game_server_ip_enabled = config.game_server_ip_enabled;
        let bg_index = config.bg_index;
        let tr = translations(config.language);
        let app_icon = Self::load_app_icon(&cc.egui_ctx);

        // Image vectors start empty; `start_bg_loading` resizes them on the
        // first frame after scanning the picture directory.
        let bgs: Vec<Option<egui::TextureHandle>> = Vec::new();
        let bg_thumbs: Vec<Option<egui::TextureHandle>> = Vec::new();
        // Sender is dropped immediately, leaving a disconnected receiver.
        // `start_bg_loading` replaces it on the first update frame.
        let (_, img_rx) = channel::<Option<BgImageData>>();
        // Same trick for the video load channel.
        let (_, video_load_rx) = channel::<Option<VideoLoadData>>();
        let current_video = config.bg_video_index;

        Self {
            config,
            client,
            storage,
            runtime,
            task_tx,
            task_rx,
            tr,
            app_icon,
            bgs,
            bg_thumbs,
            current_bg: bg_index,
            thumb_scroll_offset: 0.0,
            thumb_velocity: 0.0,
            thumb_drag_active: false,
            thumb_drag_distance: 0.0,
            img_rx,
            bg_pending: 0,
            bg_loading_started: false,
            videos: Vec::new(),
            video_thumbs: Vec::new(),
            video_texture: None,
            video_worker: None,
            video_worker_idx: None,
            current_video,
            next_video_frame_at: None,
            video_load_rx,
            video_pending: 0,
            video_loading_started: false,
            state: AppState::Login,
            current_task: None,
            settings_server_url,
            settings_aes_key,
            settings_pic_path,
            settings_vid_path,
            settings_plugins_path,
            settings_plugin_inject_enabled,
            settings_game_server_ip_enabled,
            username,
            password,
            remember_password,
            register_username: String::new(),
            register_password: String::new(),
            register_password_confirm: String::new(),
            register_qq: String::new(),
            changepwd_username: String::new(),
            changepwd_old_password: String::new(),
            changepwd_new_password: String::new(),
            changepwd_confirm: String::new(),
            message: None,
            message_is_error: false,
            show_confirm_close_dnf: false,
            pending_launch: None,
        }
    }
}

impl Drop for DnfLoginApp {
    /// Stop the video worker before any field is dropped.
    /// Field drop order would otherwise destroy the tokio runtime first. The
    /// runtime then blocks waiting for the worker thread, but the worker is
    /// blocked sending into a full sync channel whose receiver still lives on
    /// the not-yet-dropped handle.
    fn drop(&mut self) {
        self.video_worker = None;
    }
}

impl eframe::App for DnfLoginApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(result) = self.task_rx.try_recv() {
            self.handle_task_result(result);
        }

        // Start background loading on the first frame. eframe already skips
        // `ui()` when the window is hidden, so no extra visibility gate is needed.
        if !self.bg_loading_started {
            self.bg_loading_started = true;
            self.start_bg_loading();
        }
        if !self.video_loading_started {
            self.video_loading_started = true;
            self.start_video_loading();
        }

        // Workers send Some on success or None on failure, so bg_pending drops regardless.
        let mut loaded_any = false;
        while let Ok(msg) = self.img_rx.try_recv() {
            self.bg_pending = self.bg_pending.saturating_sub(1);
            if let Some(data) = msg {
                let i = data.index;
                if i < self.bgs.len() {
                    self.bgs[i] = Some(ctx.load_texture(
                        format!("bg_{i}"),
                        data.full_image,
                        egui::TextureOptions::LINEAR,
                    ));
                    self.bg_thumbs[i] = Some(ctx.load_texture(
                        format!("thumb_{i}"),
                        data.thumb_image,
                        egui::TextureOptions::LINEAR,
                    ));
                    loaded_any = true;
                }
            }
        }

        while let Ok(msg) = self.video_load_rx.try_recv() {
            self.video_pending = self.video_pending.saturating_sub(1);
            if let Some(data) = msg {
                let i = data.index;
                if i < self.videos.len() {
                    self.videos[i].bytes = data.bytes;
                    self.video_thumbs[i] = Some(ctx.load_texture(
                        format!("video_thumb_{i}"),
                        data.thumb_image,
                        egui::TextureOptions::LINEAR,
                    ));
                    loaded_any = true;
                }
            }
        }

        // Pauses the worker while in image mode instead of dropping it, so
        // the last decoded frame stays on screen.
        self.tick_video(ctx);

        // Request a repaint while decode or foreground tasks are still active.
        if loaded_any
            || self.bg_pending > 0
            || self.video_pending > 0
            || self.current_task.is_some()
        {
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // egui::Context is Arc-wrapped, so clone is a refcount bump.
        let ctx = ui.ctx().clone();
        let screen = ctx.viewport_rect();

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                self.paint_background(ui, screen);
                self.draw_thumbnail_strip(ui, screen);
            });

        let glass = egui::Frame::new()
            .fill(Self::c_glass_fill())
            .stroke(egui::Stroke::new(1.0, Self::c_glass_border()))
            .corner_radius(egui::CornerRadius::same(14))
            .inner_margin(egui::vec2(24.0, 18.0));

        let max_card_h = screen.height() - 68.0;

        egui::Window::new("dnf_card")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .min_width(400.0)
            .max_width(400.0)
            .max_height(max_card_h)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -22.0))
            .frame(glass)
            .show(&ctx, |ui| {
                match self.state {
                    AppState::Login => self.show_login_screen(ui),
                    AppState::Register => self.show_register_screen(ui),
                    AppState::ChangePassword => self.show_change_password_screen(ui),
                    AppState::Settings => self.show_settings_screen(ui),
                    AppState::About => self.show_about_screen(ui),
                }
                self.show_message(ui);
            });

        if self.show_confirm_close_dnf {
            self.show_confirm_close_dialog(&ctx);
        }
    }
}
