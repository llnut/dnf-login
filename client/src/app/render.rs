// dnf-login: a launcher for Dungeon & Fighter written in Rust.
// Copyright (C) 2026 llnut
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use eframe::egui;

use super::{DnfLoginApp, THUMB_H, THUMB_W};
use crate::config::{BgFillMode, BgMode};

/// Threshold to distinguish a drag from a click.
const DRAG_THRESHOLD: f32 = 3.0;

// App icon
impl DnfLoginApp {
    /// Draws the application icon. Falls back to a painted shape if no texture is loaded.
    pub(super) fn draw_app_icon(ui: &mut egui::Ui, icon_texture: Option<&egui::TextureHandle>) {
        let size = 100.0;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
        if !ui.is_rect_visible(rect) {
            return;
        }

        if let Some(tex) = icon_texture {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(tex.id(), rect, uv, egui::Color32::WHITE);
            return;
        }

        let p = ui.painter();
        let c = rect.center();
        let acc = Self::c_accent();
        let hi = Self::c_accent_hover();

        p.rect_filled(
            rect,
            egui::CornerRadius::same(16),
            egui::Color32::from_rgb(10, 22, 52),
        );
        p.rect_stroke(
            rect.shrink(1.0),
            egui::CornerRadius::same(15),
            egui::Stroke::new(1.5, acc),
            egui::StrokeKind::Inside,
        );
        p.rect_filled(
            egui::Rect::from_center_size(c + egui::vec2(0.0, -10.0), egui::vec2(5.0, 20.0)),
            egui::CornerRadius {
                nw: 3,
                ne: 3,
                sw: 0,
                se: 0,
            },
            acc,
        );
        p.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(c.x - 2.5, c.y - 20.0),
                egui::pos2(c.x + 2.5, c.y - 20.0),
                egui::pos2(c.x, c.y - 30.0),
            ],
            acc,
            egui::Stroke::NONE,
        ));
        p.rect_filled(
            egui::Rect::from_center_size(c, egui::vec2(26.0, 4.5)),
            egui::CornerRadius::same(2),
            acc,
        );
        p.circle_filled(c + egui::vec2(-13.0, 0.0), 3.5, hi);
        p.circle_filled(c + egui::vec2(13.0, 0.0), 3.5, hi);
        p.rect_filled(
            egui::Rect::from_center_size(c + egui::vec2(0.0, 11.5), egui::vec2(4.0, 15.0)),
            egui::CornerRadius::same(2),
            hi,
        );
        p.circle_filled(c + egui::vec2(0.0, 20.5), 5.0, acc);
    }
}

// Background & thumbnail rendering
impl DnfLoginApp {
    /// Paints the current background.
    /// Rendering follows `self.config.bg_fill_mode`.
    pub(super) fn paint_background(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let p = ui.painter();
        let source = match self.config.bg_mode {
            BgMode::Image => self.bgs.get(self.current_bg).and_then(|t| t.as_ref()),
            BgMode::Video => self.video_texture.as_ref(),
        };
        if let Some(bg) = source {
            let [tw, th] = bg.size();
            let tex_w = tw as f32;
            let tex_h = th as f32;
            let scr_w = rect.width();
            let scr_h = rect.height();
            let full_uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

            match &self.config.bg_fill_mode {
                BgFillMode::Stretch => {
                    p.image(bg.id(), rect, full_uv, egui::Color32::WHITE);
                }
                BgFillMode::Fill => {
                    let img_ar = tex_w / tex_h;
                    let scr_ar = scr_w / scr_h;
                    let (u0, v0, u1, v1) = if img_ar > scr_ar {
                        // Wider than screen: crop the left/right sides.
                        let visible_w = tex_h * scr_ar;
                        let crop = (tex_w - visible_w) / 2.0;
                        (crop / tex_w, 0.0, (crop + visible_w) / tex_w, 1.0)
                    } else {
                        // Taller than screen: crop the top/bottom.
                        let visible_h = tex_w / scr_ar;
                        let crop = (tex_h - visible_h) / 2.0;
                        (0.0, crop / tex_h, 1.0, (crop + visible_h) / tex_h)
                    };
                    let uv = egui::Rect::from_min_max(egui::pos2(u0, v0), egui::pos2(u1, v1));
                    p.image(bg.id(), rect, uv, egui::Color32::WHITE);
                }
                BgFillMode::Fit => {
                    p.rect_filled(rect, 0.0, Self::c_bg());
                    let img_ar = tex_w / tex_h;
                    let scr_ar = scr_w / scr_h;
                    let (dest_w, dest_h) = if img_ar > scr_ar {
                        (scr_w, scr_w / img_ar)
                    } else {
                        (scr_h * img_ar, scr_h)
                    };
                    let dest =
                        egui::Rect::from_center_size(rect.center(), egui::vec2(dest_w, dest_h));
                    p.image(bg.id(), dest, full_uv, egui::Color32::WHITE);
                }
                BgFillMode::Center => {
                    p.rect_filled(rect, 0.0, Self::c_bg());
                    let dest_w = tex_w.min(scr_w);
                    let dest_h = tex_h.min(scr_h);
                    let dest =
                        egui::Rect::from_center_size(rect.center(), egui::vec2(dest_w, dest_h));
                    // Crop UV when the image is larger than the screen.
                    let u0 = if tex_w > scr_w {
                        (tex_w - scr_w) / 2.0 / tex_w
                    } else {
                        0.0
                    };
                    let v0 = if tex_h > scr_h {
                        (tex_h - scr_h) / 2.0 / tex_h
                    } else {
                        0.0
                    };
                    let u1 = 1.0 - u0;
                    let v1 = 1.0 - v0;
                    let uv = egui::Rect::from_min_max(egui::pos2(u0, v0), egui::pos2(u1, v1));
                    p.image(bg.id(), dest, uv, egui::Color32::WHITE);
                }
                BgFillMode::Tile => {
                    let mut y = rect.min.y;
                    while y < rect.max.y {
                        let tile_h = tex_h.min(rect.max.y - y);
                        let v1 = tile_h / tex_h;
                        let mut x = rect.min.x;
                        while x < rect.max.x {
                            let tile_w = tex_w.min(rect.max.x - x);
                            let u1 = tile_w / tex_w;
                            let tile_rect = egui::Rect::from_min_size(
                                egui::pos2(x, y),
                                egui::vec2(tile_w, tile_h),
                            );
                            let uv =
                                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(u1, v1));
                            p.image(bg.id(), tile_rect, uv, egui::Color32::WHITE);
                            x += tex_w;
                        }
                        y += tex_h;
                    }
                }
            }
        } else {
            p.rect_filled(rect, 0.0, Self::c_bg());
        }
        // Dark overlay to improve card legibility over background images.
        p.rect_filled(rect, 0.0, Self::c_overlay());
    }

    /// Draws the background thumbnail switcher strip along the bottom edge.
    /// Supports horizontal scroll when thumbnails exceed the available width.
    /// In video mode the strip iterates loaded videos; in image mode, image thumbnails.
    pub(super) fn draw_thumbnail_strip(&mut self, ui: &mut egui::Ui, screen: egui::Rect) {
        let tw = THUMB_W as f32;
        let th = THUMB_H as f32;
        let gap = 7.0;
        let pad_x = 16.0;
        let pad_y = 12.0;
        // Reserves a narrow column at the right for the version label and the
        // toggle pill stacked above it.
        let version_reserve = 112.0;
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        let mode = self.config.bg_mode;
        let n = match mode {
            BgMode::Image => self.bg_thumbs.len(),
            BgMode::Video => self.video_thumbs.len(),
        };
        let active_idx = match mode {
            BgMode::Image => self.current_bg,
            BgMode::Video => self.current_video,
        };

        let strip_x = screen.min.x + pad_x;
        let strip_y = screen.max.y - th - pad_y;
        let strip_w = screen.width() - pad_x - version_reserve - pad_x;
        let total_w = n as f32 * (tw + gap) - gap;
        let max_scroll = (total_w - strip_w).max(0.0);

        let strip_sense = egui::Rect::from_min_max(
            egui::pos2(strip_x, strip_y - 4.0),
            egui::pos2(strip_x + strip_w, strip_y + th + 14.0),
        );
        let in_strip = ui.input(|i| {
            i.pointer
                .latest_pos()
                .is_some_and(|p| strip_sense.contains(p))
        });

        // Mouse wheel scroll
        if in_strip {
            let delta = ui.input(|i| i.smooth_scroll_delta);
            self.thumb_scroll_offset -= delta.x + delta.y;
            self.thumb_scroll_offset = self.thumb_scroll_offset.clamp(0.0, max_scroll);
        }

        // Drag scrolling with momentum
        let (primary_pressed, primary_down, pointer_dx) = ui.input(|i| {
            (
                i.pointer.primary_pressed(),
                i.pointer.primary_down(),
                i.pointer.delta().x,
            )
        });

        if primary_pressed && in_strip {
            self.thumb_drag_active = true;
            self.thumb_velocity = 0.0;
            self.thumb_drag_distance = 0.0;
        }

        if self.thumb_drag_active {
            if primary_down {
                self.thumb_drag_distance += pointer_dx.abs();
                let dx = -pointer_dx;
                self.thumb_scroll_offset += dx;
                self.thumb_scroll_offset = self.thumb_scroll_offset.clamp(0.0, max_scroll);
                self.thumb_velocity = self.thumb_velocity * 0.6 + dx * 0.4;
            } else {
                self.thumb_drag_active = false;
                // Treat short movements as a click, not a drag
                if self.thumb_drag_distance < DRAG_THRESHOLD {
                    self.thumb_velocity = 0.0;
                }
            }
        }

        // Momentum decay
        if !self.thumb_drag_active && self.thumb_velocity.abs() > 0.5 {
            self.thumb_scroll_offset += self.thumb_velocity;
            self.thumb_scroll_offset = self.thumb_scroll_offset.clamp(0.0, max_scroll);
            self.thumb_velocity *= 0.92;
            if self.thumb_velocity.abs() < 0.5 {
                self.thumb_velocity = 0.0;
            }
            ui.ctx().request_repaint();
        }

        let clip_rect = egui::Rect::from_min_max(
            egui::pos2(strip_x - 1.0, strip_y - 4.0),
            egui::pos2(strip_x + strip_w + 1.0, strip_y + th + 14.0),
        );
        let painter = ui.painter().with_clip_rect(clip_rect);

        for i in 0..n {
            let x = strip_x + i as f32 * (tw + gap) - self.thumb_scroll_offset;
            let r = egui::Rect::from_min_size(egui::pos2(x, strip_y), egui::vec2(tw, th));

            if r.max.x <= clip_rect.min.x || r.min.x >= clip_rect.max.x {
                continue;
            }

            // Use the clipped rect for interaction so partially visible thumbnails respond correctly.
            let visible = egui::Rect::from_min_max(
                egui::pos2(r.min.x.max(clip_rect.min.x), r.min.y),
                egui::pos2(r.max.x.min(clip_rect.max.x), r.max.y),
            );
            let resp = ui.allocate_rect(visible, egui::Sense::click());

            let thumb = match mode {
                BgMode::Image => self.bg_thumbs.get(i).and_then(|t| t.as_ref()),
                BgMode::Video => self.video_thumbs.get(i).and_then(|t| t.as_ref()),
            };
            if let Some(thumb) = thumb {
                painter.image(thumb.id(), r, uv, egui::Color32::WHITE);
            } else {
                painter.rect_filled(r, egui::CornerRadius::same(4), Self::c_card());
            }

            let is_active = i == active_idx;
            let border_color = if is_active {
                Self::c_thumb_active()
            } else if resp.hovered() {
                Self::c_thumb_hover()
            } else {
                Self::c_thumb_inactive()
            };
            let border_w = if is_active { 2.0 } else { 1.0 };
            painter.rect_stroke(
                r,
                egui::CornerRadius::same(4),
                egui::Stroke::new(border_w, border_color),
                egui::StrokeKind::Outside,
            );

            if is_active {
                painter.circle_filled(
                    egui::pos2(r.center().x, r.max.y + 5.0),
                    2.0,
                    Self::c_thumb_active(),
                );
            }

            if resp.clicked() && self.thumb_drag_distance < DRAG_THRESHOLD {
                match mode {
                    BgMode::Image => {
                        self.current_bg = i;
                        self.config.bg_index = i;
                    }
                    BgMode::Video => {
                        self.current_video = i;
                        self.config.bg_video_index = i;
                    }
                }
                if let Err(e) = self.config.save() {
                    tracing::warn!("Failed to save bg selection: {}", e);
                }
                // Scroll the selected thumbnail into view.
                let thumb_left = i as f32 * (tw + gap);
                let thumb_right = thumb_left + tw;
                if thumb_left < self.thumb_scroll_offset {
                    self.thumb_scroll_offset = thumb_left;
                } else if thumb_right > self.thumb_scroll_offset + strip_w {
                    self.thumb_scroll_offset = (thumb_right - strip_w).clamp(0.0, max_scroll);
                }
            }
        }

        // Scroll progress bar (only shown when content overflows).
        if max_scroll > 0.0 {
            let bar_y = strip_y + th + 9.0;
            let knob_w = (strip_w / total_w * strip_w).max(20.0);
            let knob_x = strip_x + (self.thumb_scroll_offset / max_scroll) * (strip_w - knob_w);
            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(strip_x, bar_y), egui::vec2(strip_w, 2.0)),
                0.0,
                Self::c_thumb_inactive(),
            );
            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(knob_x, bar_y), egui::vec2(knob_w, 2.0)),
                0.0,
                Self::c_thumb_active(),
            );
        }

        self.draw_bg_mode_pill(ui, screen);

        ui.painter().text(
            screen.max - egui::vec2(12.0, 8.0),
            egui::Align2::RIGHT_BOTTOM,
            concat!("v", env!("CARGO_PKG_VERSION")),
            egui::FontId::proportional(11.0),
            Self::c_text3(),
        );
    }

    /// Draws a small two-segment pill that toggles between image and video
    /// backgrounds. Stacked directly above the version label so the bottom
    /// thumbnail strip can extend further right.
    fn draw_bg_mode_pill(&mut self, ui: &mut egui::Ui, screen: egui::Rect) {
        let pill_w = 96.0;
        let pill_h = 20.0;
        let right_margin = 12.0;
        let version_baseline = 8.0;
        // Approximate height of the 11pt version label, plus a small visual gap.
        let version_band = 13.0;
        let stack_gap = 4.0;

        let pill_max_x = screen.max.x - right_margin;
        let pill_min_x = pill_max_x - pill_w;
        let pill_max_y = screen.max.y - version_baseline - version_band - stack_gap;
        let pill_min_y = pill_max_y - pill_h;
        let pill_rect = egui::Rect::from_min_max(
            egui::pos2(pill_min_x, pill_min_y),
            egui::pos2(pill_max_x, pill_max_y),
        );

        let painter = ui.painter();
        painter.rect_filled(
            pill_rect,
            egui::CornerRadius::same(10),
            Self::c_thumb_inactive(),
        );

        let seg_w = pill_w / 2.0;
        let img_seg = egui::Rect::from_min_size(pill_rect.min, egui::vec2(seg_w, pill_h));
        let vid_seg = egui::Rect::from_min_size(
            egui::pos2(pill_rect.min.x + seg_w, pill_rect.min.y),
            egui::vec2(seg_w, pill_h),
        );

        let active_seg = match self.config.bg_mode {
            BgMode::Image => img_seg,
            BgMode::Video => vid_seg,
        };
        let active_corners = match self.config.bg_mode {
            BgMode::Image => egui::CornerRadius {
                nw: 10,
                sw: 10,
                ne: 0,
                se: 0,
            },
            BgMode::Video => egui::CornerRadius {
                nw: 0,
                sw: 0,
                ne: 10,
                se: 10,
            },
        };
        painter.rect_filled(active_seg, active_corners, Self::c_thumb_active());

        let label_font = egui::FontId::proportional(11.0);
        let img_color = if self.config.bg_mode == BgMode::Image {
            egui::Color32::WHITE
        } else {
            Self::c_text3()
        };
        let vid_color = if self.config.bg_mode == BgMode::Video {
            egui::Color32::WHITE
        } else {
            Self::c_text3()
        };
        painter.text(
            img_seg.center(),
            egui::Align2::CENTER_CENTER,
            self.tr.bg_toggle_image,
            label_font.clone(),
            img_color,
        );
        painter.text(
            vid_seg.center(),
            egui::Align2::CENTER_CENTER,
            self.tr.bg_toggle_video,
            label_font,
            vid_color,
        );

        let img_resp = ui.allocate_rect(img_seg, egui::Sense::click());
        let vid_resp = ui.allocate_rect(vid_seg, egui::Sense::click());

        let prev_mode = self.config.bg_mode;
        if img_resp.clicked() {
            self.config.bg_mode = BgMode::Image;
        }
        if vid_resp.clicked() {
            self.config.bg_mode = BgMode::Video;
        }
        if self.config.bg_mode != prev_mode
            && let Err(e) = self.config.save()
        {
            tracing::warn!("Failed to save bg_mode: {}", e);
        }
    }
}
