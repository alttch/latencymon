use bmart_derive::EnumStr;
use clap::{Parser, ValueEnum};
use rand::{thread_rng, Rng};
use std::time::Duration;
use syslog::{BasicLogger, Facility, Formatter3164};

mod icmp;
mod output;
mod tcp;
mod udp;

fn create_frame(frame_size: usize) -> Vec<u8> {
    let mut frame = vec![0_u8; frame_size];
    thread_rng().fill(frame.as_mut_slice());
    frame
}

const HELLO: u8 = 0xEE;
const MIN_FRAME_SIZE: usize = 1;
const MAX_UDP_FRAME_SIZE: usize = 10_000_000;

#[derive(ValueEnum, PartialEq, Clone)]
#[clap(rename_all = "lowercase")]
enum Mode {
    Client,
    Server,
}

#[derive(ValueEnum, PartialEq, Clone, EnumStr)]
#[clap(rename_all = "lowercase")]
#[enumstr(rename_all = "UPPERCASE")]
pub enum Proto {
    Tcp,
    Udp,
    Icmp,
}

#[derive(Parser)]
struct Args {
    #[clap()]
    mode: Mode,
    #[clap()]
    proto: Proto,
    #[clap()]
    socket: String,
    #[clap(short = 'T', long = "timeout", default_value = "30")]
    timeout: u16,
    #[clap(short = 'I', long = "interval", default_value = "1.0")]
    interval: f64,
    #[clap(short = 'S', long = "frame-size", default_value = "1500")]
    frame_size: u32,
    #[clap(short = 'W', long = "latency-warn")]
    warn: Option<f64>,
    #[clap(long = "syslog")]
    syslog: bool,
    #[clap(short = 'C', long = "chart")]
    chart: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let level_filter = log::LevelFilter::Info;
    if args.syslog {
        let formatter = Formatter3164 {
            facility: Facility::LOG_USER,
            hostname: None,
            process: "latencymon".into(),
            pid: 0,
        };

        let logger = syslog::unix(formatter)?;
        log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
            .map(|()| log::set_max_level(level_filter))?;
    } else {
        env_logger::Builder::new()
            .target(env_logger::Target::Stdout)
            .filter_level(level_filter)
            .init();
    }
    let timeout = Duration::from_secs(args.timeout.into());
    match args.mode {
        Mode::Server => match args.proto {
            Proto::Tcp => tcp::run_server(&args.socket, timeout)?,
            Proto::Udp => udp::run_server(&args.socket)?,
            Proto::Icmp => unimplemented!(),
        },
        Mode::Client => match args.proto {
            Proto::Tcp => tcp::run_client(
                &args.socket,
                timeout,
                args.frame_size,
                args.interval,
                args.warn,
                args.chart,
            )?,
            Proto::Udp => udp::run_client(
                &args.socket,
                timeout,
                args.frame_size,
                args.interval,
                args.warn,
                args.chart,
            )?,
            Proto::Icmp => {
                icmp::run_client(&args.socket, timeout, args.interval, args.warn, args.chart)?
            }
        },
    }
    Ok(())
}
