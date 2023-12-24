use crate::Proto;
use colored::Colorize;
use log::{info, warn};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::thread;
use std::time::{Duration, Instant};
use textplots::{AxisBuilder, Chart, LabelBuilder, LabelFormat, LineStyle, Plot, Shape};

const CAROUSEL_CHARS: &[char] = &['-', '\\', '|', '/'];
const CLREOL: &[u8] = &[0x1b, b'[', b'0', b'G', 0x1b, b'[', b'0', b'K'];

const MAX_POINTS: u16 = 1000;

static DATA: Lazy<Mutex<VecDeque<f32>>> =
    Lazy::new(|| Mutex::new(vec![0.0; usize::from(MAX_POINTS)].into()));

pub fn append(v: f32) {
    let mut data = DATA.lock();
    data.push_back(v);
    data.pop_front();
}

pub fn redraw(title: &str, last: f32) {
    if let Ok((mut width, height)) = termion::terminal_size() {
        let _ = write!(
            io::stdout(),
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        );
        width = width * 2 - 18;
        if width > MAX_POINTS {
            width = MAX_POINTS;
        }
        #[allow(clippy::cast_precision_loss)]
        let points = {
            let data = DATA.lock();
            let points = data
                .iter()
                .skip(data.len() - usize::from(width))
                .enumerate()
                .map(|(i, v)| (i as f32, *v))
                .collect::<Vec<(f32, f32)>>();
            points
        };
        println!("{}: {} ms", title, format!("{:.0}", last).white().bold());
        Chart::new(width.into(), height.into(), 0.0, f32::from(width))
            .x_axis_style(LineStyle::None)
            .y_axis_style(LineStyle::None)
            .x_label_format(LabelFormat::None)
            .y_label_format(LabelFormat::Custom(Box::new(|v| format!("{:.0}", v))))
            .lineplot(&Shape::Lines(&points))
            .display();
    }
}

pub fn clear_line() {
    if atty::is(atty::Stream::Stdout) {
        let _ = io::stdout().write(CLREOL);
    }
}

pub struct Output {
    interval: Duration,
    carousel_enabled: bool,
    carousel_buf: [u8; 5],
    carousel_pos: usize,
    latency_warn: Option<Duration>,
    draw_chart: bool,
    next: Instant,
    op: Instant,
    title: String,
}

impl Output {
    pub fn new(
        addr: SocketAddr,
        proto: Proto,
        frame_size: Option<usize>,
        interval: Duration,
        latency_warn: Option<Duration>,
        draw_chart: bool,
    ) -> Self {
        let now = Instant::now();
        let mut title = format!(
            "{} ({})",
            if addr.port() > 0 {
                addr.to_string()
            } else {
                addr.ip().to_string()
            }
            .green(),
            proto,
        );
        if let Some(f) = frame_size {
            let _ = write!(title, " {} bytes", f.to_string().cyan());
        }
        Self {
            title,
            interval,
            carousel_enabled: atty::is(atty::Stream::Stdout),
            carousel_buf: [0x1b, b'[', b'D', 0x00, 0],
            carousel_pos: 0,
            draw_chart,
            latency_warn,
            next: now + interval,
            op: now,
        }
    }
    pub fn finish_iteration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let elapsed = self.op.elapsed();
        if self.draw_chart {
            let val = elapsed.as_secs_f32() * 1000.0;
            append(val);
            redraw(&self.title, val);
        } else {
            if let Some(w) = self.latency_warn {
                if elapsed >= w {
                    if self.carousel_enabled {
                        io::stdout().write_all(CLREOL)?;
                    }
                    warn!(
                        "latency: {} sec ({} ms)",
                        elapsed.as_secs_f64(),
                        elapsed.as_millis()
                    );
                } else if self.carousel_enabled {
                    self.carousel_buf[4] = CAROUSEL_CHARS[self.carousel_pos] as u8;
                    let mut stdout = io::stdout();
                    stdout.write_all(&self.carousel_buf)?;
                    io::stdout().flush()?;
                }
            } else {
                info!(
                    "latency: {} sec ({} ms)",
                    elapsed.as_secs_f64(),
                    elapsed.as_millis()
                );
            }
            self.carousel_pos += 1;
            if self.carousel_pos >= CAROUSEL_CHARS.len() {
                self.carousel_pos = 0;
            }
        }
        let now = Instant::now();
        if now > self.next {
            if !self.draw_chart {
                warn!("loop timeout");
            }
            self.next = now + self.interval;
        } else {
            thread::sleep(self.next - now);
            self.next += self.interval;
        }
        self.op = Instant::now();
        Ok(())
    }
}
