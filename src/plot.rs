use std::io::{BufWriter, Write};
use std::process::{Command, Stdio};

use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use rayon::prelude::*;

pub struct LandscapeCache {
    background_buffer: Vec<u8>,
    width: u32,
    height: u32,
    scale_x: f32,
    scale_y: f32,
    offset_x: f32,
    offset_y: f32,
}

impl LandscapeCache {
    fn new<F>(
        func: F,
        x_bounds: [f32; 2],
        y_bounds: [f32; 2],
        width: u32,
        height: u32,
        resolution: u32,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        F: Fn(&[f32]) -> f32,
    {
        let mut buffer = vec![0u8; (width * height * 3) as usize];
        let [x_min, x_max] = x_bounds;
        let [y_min, y_max] = y_bounds;
        let step_x = (x_max - x_min) / resolution as f32;
        let step_y = (y_max - y_min) / resolution as f32;
        let mut z_values = Vec::with_capacity((resolution * resolution) as usize);
        let mut true_min = f32::MAX;
        let mut true_max = f32::MIN;
        for x_i in 0..resolution {
            for y_i in 0..resolution {
                let x = x_min + x_i as f32 * step_x + (step_x / 2.0);
                let y = y_min + y_i as f32 * step_y + (step_y / 2.0);
                let z = func(&[x, y]);
                if z < true_min {
                    true_min = z;
                }
                if z > true_max {
                    true_max = z;
                }
                z_values.push(z);
            }
        }
        if (true_max - true_min).abs() < f32::EPSILON {
            true_max += 1.0;
        }
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
            root.fill(&WHITE)?;
            let mut chart =
                ChartBuilder::on(&root).build_cartesian_2d(x_min..x_max, y_min..y_max)?;
            chart.configure_mesh().disable_mesh().draw()?;
            chart.draw_series(
                (0..resolution)
                    .flat_map(|x| (0..resolution).map(move |y| (x, y)))
                    .map(|(x_i, y_i)| {
                        let x0 = x_min + x_i as f32 * step_x;
                        let y0 = y_min + y_i as f32 * step_y;
                        let z = z_values[(x_i * resolution + y_i) as usize];
                        let t = (z - true_min) / (true_max - true_min);
                        let color = HSLColor((1.0 - t) as f64 * 0.66, 1.0, 0.5);
                        Rectangle::new(
                            [(x0, y0), (x0 + step_x * 1.02, y0 + step_y * 1.02)],
                            color.filled(),
                        )
                    }),
            )?;
            root.present()?;
        }
        let scale_x = width as f32 / (x_max - x_min);
        let scale_y = height as f32 / (y_max - y_min);
        Ok(Self {
            background_buffer: buffer,
            width,
            height,
            scale_x,
            scale_y,
            offset_x: x_min,
            offset_y: y_min,
        })
    }

    #[inline(always)]
    fn map_coords(&self, x: f32, y: f32) -> (i32, i32) {
        let px = ((x - self.offset_x) * self.scale_x) as i32;
        let py = self.height as i32 - 1 - ((y - self.offset_y) * self.scale_y) as i32;
        (px, py)
    }

    fn draw_fast_circle(&self, buf: &mut [u8], cx: i32, cy: i32, r: i32, color: [u8; 3]) {
        let r_sq = r * r;
        let w = self.width as i32;
        let h = self.height as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r_sq {
                    let px = cx + dx;
                    let py = cy + dy;
                    if px >= 0 && px < w && py >= 0 && py < h {
                        let idx = ((py * w + px) * 3) as usize;
                        buf[idx..idx + 3].copy_from_slice(&color);
                    }
                }
            }
        }
    }

    fn render_frame(
        &self,
        points: &[[f32; 2]],
        best_f_x: f32,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut frame_buffer = self.background_buffer.clone();
        for point in points {
            let (px, py) = self.map_coords(point[0], point[1]);
            self.draw_fast_circle(&mut frame_buffer, px, py, 4, [255, 255, 255]);
            self.draw_fast_circle(&mut frame_buffer, px, py, 3, [0, 0, 0]);
        }
        {
            let root = BitMapBackend::with_buffer(&mut frame_buffer, (self.width, self.height))
                .into_drawing_area();
            let text_content = format!("{:.2}", best_f_x);
            let mut text_style = TextStyle::from(("sans-serif", 30.0).into_font()).color(&WHITE);
            text_style.pos = Pos::new(HPos::Right, VPos::Top);
            let (text_w, text_h) = root.estimate_text_size(&text_content, &text_style)?;
            let text_anchor_x = self.width as i32 - 15;
            let text_anchor_y = 15;
            root.draw(&Rectangle::new(
                [
                    (text_anchor_x - text_w as i32 - 8, text_anchor_y - 8),
                    (text_anchor_x + 8, text_anchor_y + text_h as i32 + 8),
                ],
                BLACK.mix(0.7).filled(),
            ))?;
            root.draw(&Text::new(
                text_content,
                (text_anchor_x, text_anchor_y),
                text_style,
            ))?;
            root.present()?;
        }
        Ok(frame_buffer)
    }
}

pub fn save_video_h264<F>(
    func: F,
    x_history: &[Vec<[f32; 2]>],
    best_f_x_history: &[f32],
    filename: &str, // Ensure this ends in .mp4
    x_bounds: [f32; 2],
    y_bounds: [f32; 2],
    width: u32,
    height: u32,
    resolution: u32,
    fps: u32,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(&[f32]) -> f32 + Sync,
{
    let landscape = LandscapeCache::new(func, x_bounds, y_bounds, width, height, resolution)?;
    let mut child = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb24",
            "-video_size",
            &format!("{}x{}", width, height),
            "-framerate",
            &fps.to_string(),
            "-i",
            "-",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "28",
            "-threads",
            "0",
            "-pix_fmt",
            "yuv420p",
            filename,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {}", e))?;
    let stdin = child.stdin.take().ok_or("Failed to open ffmpeg stdin")?;
    let mut writer = BufWriter::with_capacity((width * height * 3 * 2) as usize, stdin);
    let batch_size: usize = 48;
    let total = x_history.len();
    for batch_start in (0..total).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(total);
        let frames: Vec<Vec<u8>> = (batch_start..batch_end)
            .into_par_iter()
            .map(|i| {
                landscape
                    .render_frame(&x_history[i], best_f_x_history[i])
                    .expect("Render failed")
            })
            .collect();
        for pixels in frames {
            writer.write_all(&pixels)?;
        }
    }
    drop(writer);
    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("FFmpeg error: {:?}", status.code()).into())
    }
}
