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
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::time::Instant;

use super::DnfLoginApp;
pub(super) use super::{THUMB_H, THUMB_W};

mod webp;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_video_dir_ignores_mp4_files() {
        let dir = std::env::temp_dir().join(format!("dnf-login-webp-scan-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let webp = dir.join("loop.webp");
        let mp4 = dir.join("loop.mp4");
        std::fs::write(&webp, b"not a real webp").unwrap();
        std::fs::write(mp4, b"ignored").unwrap();

        let entries = scan_video_dir(dir.to_str().unwrap());

        assert_eq!(entries, vec![webp]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// One animated WebP entry in the switcher strip.
pub(crate) struct VideoEntry {
    pub(super) bytes: Arc<Vec<u8>>,
}

impl VideoEntry {
    fn is_ready(&self) -> bool {
        !self.bytes.is_empty()
    }
}

/// Decoded animation frame ready for GPU upload.
pub(super) struct VideoFrame {
    /// Per-frame delay from the animated WebP source.
    pub(super) fallback_delay: std::time::Duration,
    pub(super) image: egui::ColorImage,
}

pub(crate) struct VideoLoadData {
    pub(super) index: usize,
    pub(super) bytes: Arc<Vec<u8>>,
    pub(super) thumb_image: egui::ColorImage,
}

/// Owns a streaming decode task. Dropping the handle stops the worker.
pub(crate) struct VideoWorkerHandle {
    pub(super) rx: Receiver<VideoFrame>,
    /// When true, the worker stops decoding. The main thread keeps the last
    /// uploaded texture so returning to video mode is instant and non-black.
    pub(super) paused: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
}

impl VideoWorkerHandle {
    pub(super) fn new(
        rx: Receiver<VideoFrame>,
        stop: Arc<AtomicBool>,
        paused: Arc<AtomicBool>,
    ) -> Self {
        Self { rx, stop, paused }
    }
}

impl Drop for VideoWorkerHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

pub(super) fn new_stop_flag() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}

pub(super) fn scan_video_dir(dir: &str) -> Vec<PathBuf> {
    let path = Path::new(dir);
    if !path.is_dir() {
        return Vec::new();
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("webp"))
        })
        .collect();
    entries.sort();
    entries
}

fn load_video_source(path: &Path) -> Option<(Arc<Vec<u8>>, egui::ColorImage)> {
    let bytes = Arc::new(std::fs::read(path).ok()?);
    let thumb = webp::decode_webp_thumbnail(&bytes)?;
    Some((bytes, thumb))
}

pub(super) fn start_video_worker(
    bytes: Arc<Vec<u8>>,
    runtime: &tokio::runtime::Runtime,
    ctx: egui::Context,
) -> VideoWorkerHandle {
    webp::start_webp_worker(bytes, runtime, ctx)
}

impl DnfLoginApp {
    pub(super) fn start_video_loading(&mut self) {
        // Drop any worker still holding bytes from a previous scan.
        self.video_worker = None;
        self.video_worker_idx = None;
        self.video_texture = None;
        self.next_video_frame_at = None;

        let paths = scan_video_dir(&self.config.bg_vid_path);
        let n = paths.len();

        self.videos = (0..n)
            .map(|_| VideoEntry {
                bytes: Arc::new(Vec::new()),
            })
            .collect();
        self.video_thumbs = vec![None; n];

        if n == 0 || self.current_video >= n {
            self.current_video = 0;
        }

        let (tx, rx) = std::sync::mpsc::channel::<Option<VideoLoadData>>();
        self.video_load_rx = rx;
        self.video_pending = n;

        let parallelism = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2);
        let max_jobs = parallelism.saturating_sub(1).max(1);
        let sem = Arc::new(tokio::sync::Semaphore::new(max_jobs));

        for (i, path) in paths.into_iter().enumerate() {
            let tx = tx.clone();
            let sem = sem.clone();
            self.runtime.spawn(async move {
                let _permit = sem.acquire_owned().await.ok();
                let result = tokio::task::spawn_blocking(move || {
                    let (bytes, thumb_image) = load_video_source(&path)?;
                    Some(VideoLoadData {
                        index: i,
                        bytes,
                        thumb_image,
                    })
                })
                .await;
                let _ = tx.send(result.ok().flatten());
            });
        }
    }

    /// Drives the animated-WebP pipeline each frame. Switching videos restarts
    /// the worker; switching to image mode pauses it while keeping the last GPU
    /// texture visible.
    pub(super) fn tick_video(&mut self, ctx: &egui::Context) {
        use crate::config::BgMode;

        if self.config.bg_mode != BgMode::Video {
            if let Some(handle) = self.video_worker.as_ref() {
                handle.paused.store(true, Ordering::Release);
            }
            self.next_video_frame_at = None;
            return;
        }

        let idx = self.current_video;
        let source_ready = idx < self.videos.len() && self.videos[idx].is_ready();
        let needs_restart = self.video_worker_idx != Some(idx) || self.video_worker.is_none();
        if source_ready && needs_restart {
            self.video_worker = None;
            self.video_texture = None;
            self.next_video_frame_at = None;

            let worker =
                start_video_worker(self.videos[idx].bytes.clone(), &self.runtime, ctx.clone());
            self.video_worker = Some(worker);
            self.video_worker_idx = Some(idx);
        } else if let Some(handle) = self.video_worker.as_ref() {
            handle.paused.store(false, Ordering::Release);
        }

        self.tick_video_fallback(ctx);
    }

    fn tick_video_fallback(&mut self, ctx: &egui::Context) {
        let handle = match self.video_worker.as_ref() {
            Some(h) => h,
            None => return,
        };
        let now = Instant::now();
        let due = self.next_video_frame_at.map(|t| t <= now).unwrap_or(true);
        if due {
            if let Ok(frame) = handle.rx.try_recv() {
                self.next_video_frame_at = Some(now + frame.fallback_delay);
                self.upload_frame(ctx, frame);
            }
        } else if let Some(target) = self.next_video_frame_at {
            ctx.request_repaint_after(target.saturating_duration_since(now));
        }
    }

    fn upload_frame(&mut self, ctx: &egui::Context, frame: VideoFrame) {
        match self.video_texture.as_mut() {
            Some(tex) => tex.set(frame.image, egui::TextureOptions::LINEAR),
            None => {
                self.video_texture = Some(ctx.load_texture(
                    "video_current",
                    frame.image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
    }
}
