use bmart_derive::EnumStr;
use clap::{Parser, ValueEnum};
use rand::{thread_rng, Rng};
use std::time::Duration;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPOSITORY: &str = "https://github.com/alttch/latencymon";

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

#[derive(ValueEnum, PartialEq, Copy, Clone, EnumStr)]
#[clap(rename_all = "lowercase")]
#[enumstr(rename_all = "UPPERCASE")]
pub enum Proto {
    Tcp,
    Udp,
    Icmp,
}

#[derive(Parser)]
#[clap(version = VERSION, about = REPOSITORY)]
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
    #[clap(
        short = 'S',
        long = "frame-size",
        default_value = "1500",
        help = "frame size (TCP/UDP)"
    )]
    frame_size: u32,
    #[clap(short = 'W', long = "latency-warn")]
    warn: Option<f64>,
    #[clap(
        short = 'O',
        long = "output",
        help = "output kind",
        default_value = "regular"
    )]
    output_kind: output::Kind,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    output::init_logger(args.output_kind)?;
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
                args.output_kind,
            )?,
            Proto::Udp => udp::run_client(
                &args.socket,
                timeout,
                args.frame_size,
                args.interval,
                args.warn,
                args.output_kind,
            )?,
            Proto::Icmp => icmp::run_client(
                &args.socket,
                timeout,
                args.interval,
                args.warn,
                args.output_kind,
            )?,
        },
    }
    Ok(())
}
