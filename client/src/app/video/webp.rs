use eframe::egui;
use image::AnimationDecoder;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::sync_channel;
use std::time::Duration;

use super::{THUMB_H, THUMB_W, VideoFrame, VideoWorkerHandle, new_stop_flag};

/// Returns the first frame of an animated WebP, scaled into a thumbnail tile.
/// Two-stage downscale via `thumbnail` then Lanczos3 mirrors the image
/// pipeline used for static backgrounds.
pub(super) fn decode_webp_thumbnail(bytes: &[u8]) -> Option<egui::ColorImage> {
    let decoder = image::codecs::webp::WebPDecoder::new(Cursor::new(bytes)).ok()?;
    let first = decoder.into_frames().next()?.ok()?;
    let frame_buf = first.into_buffer();
    let dynamic = image::DynamicImage::ImageRgba8(frame_buf);
    let pre = dynamic.thumbnail(THUMB_W * 4, THUMB_H * 4);
    let pixels = pre
        .resize_to_fill(THUMB_W, THUMB_H, image::imageops::FilterType::Lanczos3)
        .to_rgba8()
        .into_raw();

    Some(egui::ColorImage::from_rgba_unmultiplied(
        [THUMB_W as usize, THUMB_H as usize],
        &pixels,
    ))
}

/// Spawns a worker that streams animated-WebP frames from `bytes` to a bounded
/// channel. The decoder is rebuilt at iterator end for seamless looping; channel
/// capacity 4 provides back-pressure so decoding pauses when the consumer lags.
pub(super) fn start_webp_worker(
    bytes: Arc<Vec<u8>>,
    runtime: &tokio::runtime::Runtime,
    ctx: egui::Context,
) -> VideoWorkerHandle {
    let (tx, rx) = sync_channel::<VideoFrame>(4);
    let stop = new_stop_flag();
    let stop_worker = stop.clone();
    let paused = Arc::new(AtomicBool::new(false));
    let paused_for_worker = paused.clone();

    runtime.spawn_blocking(move || {
        loop {
            if stop_worker.load(Ordering::Relaxed) {
                return;
            }
            // Pause check before opening the decoder. `if`+`continue` rebuilds
            // the decoder from the start of bytes on resume, rather than
            // slipping into the middle of a previous iteration.
            if paused_for_worker.load(Ordering::Acquire) {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            let cursor = Cursor::new(bytes.as_slice());
            let decoder = match image::codecs::webp::WebPDecoder::new(cursor) {
                Ok(d) => d,
                Err(e) => {
                    tracing::warn!("Failed to open animated WebP: {}", e);
                    return;
                }
            };
            for frame_result in decoder.into_frames() {
                if stop_worker.load(Ordering::Relaxed) {
                    return;
                }
                // Pause check inside the decode loop. `while` holds the
                // current `frame_result` until resume, so a long animation
                // can pause mid-iteration without rebuilding the decoder.
                while paused_for_worker.load(Ordering::Acquire) {
                    std::thread::sleep(Duration::from_millis(50));
                    if stop_worker.load(Ordering::Relaxed) {
                        return;
                    }
                }
                let Ok(frame) = frame_result else { continue };

                let mut delay: Duration = frame.delay().into();
                if delay.is_zero() {
                    delay = Duration::from_millis(33);
                }

                let buf = frame.buffer();
                let (w, h) = (buf.width() as usize, buf.height() as usize);
                let image = egui::ColorImage::from_rgba_unmultiplied([w, h], buf.as_raw());

                if tx
                    .send(VideoFrame {
                        fallback_delay: delay,
                        image,
                    })
                    .is_err()
                {
                    return;
                }
                ctx.request_repaint();
            }
        }
    });

    VideoWorkerHandle::new(rx, stop, paused)
}
