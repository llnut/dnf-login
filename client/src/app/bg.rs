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
use std::sync::mpsc::channel;

use super::{BgImageData, DnfLoginApp, THUMB_H, THUMB_W};

impl DnfLoginApp {
    /// Decodes the embedded ICO file and registers it as an egui texture.
    /// Returns `None` if the ICO data cannot be decoded.
    pub(super) fn load_app_icon(ctx: &egui::Context) -> Option<egui::TextureHandle> {
        const ICON_BYTES: &[u8] = include_bytes!("../../resources/DNF.ico");
        let dir = ico::IconDir::read(std::io::Cursor::new(ICON_BYTES)).ok()?;
        let entry = dir
            .entries()
            .iter()
            .filter(|e| e.width() >= 32)
            .max_by_key(|e| e.width())
            .or_else(|| dir.entries().first())?;
        let image = entry.decode().ok()?;
        let w = image.width() as usize;
        let h = image.height() as usize;
        let rgba = image.rgba_data().to_vec();
        let color_image = egui::ColorImage::from_rgba_unmultiplied([w, h], &rgba);
        Some(ctx.load_texture("app_icon", color_image, egui::TextureOptions::LINEAR))
    }

    /// Decodes one background JPEG and produces both the full-size and thumbnail variants,
    /// sharing the single decompressed image to avoid redundant decode work.
    ///
    /// Thumbnails use a two-stage resize: `thumbnail()` (O(output) box filter) pre-reduces to
    /// 4× the target size, then Lanczos3 runs on the small intermediate. This limits the kernel
    /// support width that Lanczos3 would otherwise expand at large reduction ratios.
    pub(super) fn decode_bg_pair(
        bytes: &[u8],
        max_w: u32,
        max_h: u32,
        thumb_w: u32,
        thumb_h: u32,
    ) -> Option<(egui::ColorImage, egui::ColorImage)> {
        let img = image::load_from_memory(bytes).ok()?;

        // Preserve the original aspect ratio; the fill mode handles layout at render time.
        let scaled = img.thumbnail(max_w, max_h);
        let (sw, sh) = (scaled.width(), scaled.height());
        let bg_pixels = scaled.to_rgba8().into_raw();
        let full_image =
            egui::ColorImage::from_rgba_unmultiplied([sw as usize, sh as usize], &bg_pixels);

        let pre = img.thumbnail(thumb_w * 4, thumb_h * 4);
        let thumb_pixels = pre
            .resize_to_fill(thumb_w, thumb_h, image::imageops::FilterType::Lanczos3)
            .to_rgba8()
            .into_raw();
        let thumb_image = egui::ColorImage::from_rgba_unmultiplied(
            [thumb_w as usize, thumb_h as usize],
            &thumb_pixels,
        );

        Some((full_image, thumb_image))
    }

    /// Scans `dir` for JPG/JPEG files and returns their paths sorted by filename.
    /// Returns an empty list if the directory does not exist or cannot be read.
    pub(super) fn scan_image_dir(dir: &str) -> Vec<std::path::PathBuf> {
        let path = std::path::Path::new(dir);
        if !path.is_dir() {
            return Vec::new();
        }
        let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(path)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.extension().is_some_and(|ext| {
                        ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg")
                    })
            })
            .collect();
        entries.sort();
        entries
    }

    /// Spawns background decode tasks for every JPG file found in
    /// `config.bg_pic_path`. Replacing `img_rx` disconnects the old channel so
    /// any stale in-flight tasks discard their results silently.
    ///
    /// A `Semaphore` limits concurrent CPU-bound tasks to `max(1, cpu_count − 1)`,
    /// leaving at least one core available for the render thread during loading.
    pub(super) fn start_bg_loading(&mut self) {
        let paths = Self::scan_image_dir(&self.config.bg_pic_path);
        let n = paths.len();

        self.bgs = vec![None; n];
        self.bg_thumbs = vec![None; n];
        // Fall back to the first image when the saved index is out of range.
        // config.bg_index is intentionally left unchanged.
        if n == 0 || self.current_bg >= n {
            self.current_bg = 0;
        }

        let (img_tx, img_rx) = channel::<Option<BgImageData>>();
        self.img_rx = img_rx;
        self.bg_pending = n;

        let bg_w = 960u32;
        let bg_h = 540u32;
        let thumb_w = THUMB_W;
        let thumb_h = THUMB_H;

        let parallelism = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2);
        let max_jobs = parallelism.saturating_sub(1).max(1);
        let sem = std::sync::Arc::new(tokio::sync::Semaphore::new(max_jobs));

        for (i, path) in paths.into_iter().enumerate() {
            let tx = img_tx.clone();
            let sem = sem.clone();
            self.runtime.spawn(async move {
                let _permit = sem.acquire_owned().await.ok();
                let result = tokio::task::spawn_blocking(move || {
                    let bytes = std::fs::read(&path).ok()?;
                    Self::decode_bg_pair(&bytes, bg_w, bg_h, thumb_w, thumb_h).map(
                        |(full_image, thumb_image)| BgImageData {
                            index: i,
                            full_image,
                            thumb_image,
                        },
                    )
                })
                .await;
                let _ = tx.send(result.ok().flatten());
            });
        }
    }

    /// Loads CJK fallback fonts from Windows system fonts for Chinese, Japanese, and Korean.
    pub(super) fn try_load_cjk_fonts(ctx: &egui::Context) {
        let font_groups: &[(&str, &[&str])] = &[
            (
                "cjk_zh",
                &[
                    r"C:\Windows\Fonts\msyh.ttc",
                    r"C:\Windows\Fonts\msjh.ttc",
                    r"C:\Windows\Fonts\simsun.ttc",
                ],
            ),
            (
                "cjk_ja",
                &[
                    r"C:\Windows\Fonts\meiryo.ttc",
                    r"C:\Windows\Fonts\YuGothR.ttc",
                    r"C:\Windows\Fonts\msgothic.ttc",
                ],
            ),
            (
                "cjk_ko",
                &[
                    r"C:\Windows\Fonts\malgun.ttf",
                    r"C:\Windows\Fonts\gulim.ttc",
                    r"C:\Windows\Fonts\batang.ttc",
                ],
            ),
        ];

        let mut fonts = egui::FontDefinitions::default();
        let mut loaded = false;

        for &(name, candidates) in font_groups {
            let mut group_loaded = false;
            for path in candidates {
                if let Ok(data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        name.to_owned(),
                        std::sync::Arc::new(egui::FontData::from_owned(data)),
                    );
                    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                        family.push(name.to_owned());
                    }
                    tracing::info!("CJK font loaded: {} from {}", name, path);
                    loaded = true;
                    group_loaded = true;
                    break;
                }
            }
            if !group_loaded {
                tracing::warn!("No font found for group: {}", name);
            }
        }

        if loaded {
            ctx.set_fonts(fonts);
        }
    }
}
