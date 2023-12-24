use crate::Proto;
use clap::ValueEnum;
use colored::Colorize;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::fmt;
use std::fmt::Write as _;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::thread;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use syslog::{BasicLogger, Facility, Formatter3164};
use textplots::{AxisBuilder, Chart, LabelBuilder, LabelFormat, LineStyle, Plot, Shape};

const CAROUSEL_CHARS: &[char] = &['-', '\\', '|', '/'];
const CLREOL: &[u8] = &[0x1b, b'[', b'0', b'G', 0x1b, b'[', b'0', b'K'];

const MAX_POINTS: u16 = 1000;

static DATA: Lazy<Mutex<VecDeque<f32>>> =
    Lazy::new(|| Mutex::new(vec![0.0; usize::from(MAX_POINTS)].into()));

#[derive(ValueEnum, PartialEq, Copy, Clone, Default)]
#[clap(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Regular,
    Syslog,
    Chart,
    Ndjson,
}

pub fn init_logger(kind: Kind) -> Result<(), Box<dyn std::error::Error>> {
    match kind {
        Kind::Regular | Kind::Chart => {
            env_logger::Builder::new()
                .target(env_logger::Target::Stdout)
                .filter_level(log::LevelFilter::Info)
                .init();
        }
        Kind::Syslog => {
            let formatter = Formatter3164 {
                facility: Facility::LOG_USER,
                hostname: None,
                process: "latencymon".into(),
                pid: 0,
            };

            let logger = syslog::unix(formatter)?;
            log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
                .map(|()| log::set_max_level(log::LevelFilter::Info))?;
        }
        Kind::Ndjson => {}
    }
    Ok(())
}

fn timestamp() -> f64 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

pub fn append_data(v: f32) {
    let mut data = DATA.lock();
    data.push_back(v);
    data.pop_front();
}

pub fn redraw_chart(title: &str, last: f32) {
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
    kind: Kind,
    next: Instant,
    op: Instant,
    title: String,
}

impl Output {
    pub fn new(
        kind: Kind,
        addr: SocketAddr,
        proto: Proto,
        frame_size: Option<usize>,
        interval: Duration,
        latency_warn: Option<Duration>,
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
            kind,
            latency_warn,
            next: now + interval,
            op: now,
        }
    }
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.op = now;
        self.next = now + self.interval;
    }
    pub fn log_iteration(
        &mut self,
        err: Option<Box<dyn std::error::Error>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(e) = err {
            self.print_error(e);
        } else {
            let elapsed = self.op.elapsed();
            if self.kind == Kind::Chart {
                let val = elapsed.as_secs_f32() * 1000.0;
                append_data(val);
                redraw_chart(&self.title, val);
            } else {
                if let Some(w) = self.latency_warn {
                    if elapsed >= w {
                        if self.carousel_enabled {
                            io::stdout().write_all(CLREOL)?;
                        }
                        self.print_latency(elapsed.as_secs_f64(), log::Level::Warn);
                    } else if self.carousel_enabled {
                        self.carousel_buf[4] = CAROUSEL_CHARS[self.carousel_pos] as u8;
                        let mut stdout = io::stdout();
                        stdout.write_all(&self.carousel_buf)?;
                        io::stdout().flush()?;
                    }
                } else {
                    self.print_latency(elapsed.as_secs_f64(), log::Level::Info);
                }
                self.carousel_pos += 1;
                if self.carousel_pos >= CAROUSEL_CHARS.len() {
                    self.carousel_pos = 0;
                }
            }
        }
        let now = Instant::now();
        if now > self.next {
            if self.kind != Kind::Chart {
                self.print_warning("loop timeout");
            }
            self.next = now + self.interval;
        } else {
            thread::sleep(self.next - now);
            self.next += self.interval;
        }
        self.op = Instant::now();
        Ok(())
    }
    fn print_warning(&self, msg: impl fmt::Display) {
        match self.kind {
            Kind::Regular | Kind::Syslog | Kind::Chart => {
                clear_line();
                log::warn!("{}", msg);
            }
            Kind::Ndjson => {}
        }
    }
    fn print_error(&self, msg: impl fmt::Display) {
        match self.kind {
            Kind::Regular | Kind::Syslog | Kind::Chart => {
                clear_line();
                log::error!("{}", msg);
            }
            Kind::Ndjson => self.print_latency(-1.0, log::Level::Error),
        }
    }
    fn print_latency(&self, latency: f64, level: log::Level) {
        match self.kind {
            Kind::Regular | Kind::Syslog => {
                log::log!(
                    level,
                    "latency: {} sec ({:.0} ms)",
                    latency,
                    latency * 1000.0
                );
            }
            Kind::Ndjson => {
                println!(r#"{{"t":{},"v":{}}}"#, timestamp(), latency);
            }
            Kind::Chart => {}
        }
    }
}
