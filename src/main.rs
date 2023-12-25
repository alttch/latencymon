use bmart_derive::EnumStr;
use clap::{Parser, ValueEnum};
use eva_common::Error;
use rand::{thread_rng, Rng};
use std::net::{SocketAddr, ToSocketAddrs};
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
    #[clap(help = "IP(ICMP)/IP:PORT")]
    path: String,
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
    #[clap(long = "output-options", help = "output options")]
    output_options: Option<String>,
}

impl Args {
    fn to_client_options(&self) -> Result<ClientOptions, Box<dyn std::error::Error>> {
        let opts = if self.proto == Proto::Icmp {
            ClientOptions {
                addr: (self.path.as_str(), 0)
                    .to_socket_addrs()?
                    .next()
                    .ok_or_else(|| Error::invalid_params("invalid ip/host"))?,
                req: None,
            }
        } else {
            let frame_size = usize::try_from(self.frame_size)?;
            if frame_size < MIN_FRAME_SIZE {
                return Err(
                    Error::invalid_data(format!("invalid frame size: {}", frame_size)).into(),
                );
            }
            ClientOptions {
                addr: self
                    .path
                    .to_socket_addrs()?
                    .next()
                    .ok_or_else(|| Error::invalid_params("invalid socket addr"))?,
                req: Some(create_frame(frame_size)),
            }
        };
        Ok(opts)
    }
}

pub struct ClientOptions {
    addr: SocketAddr,
    req: Option<Vec<u8>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    output::init_logger(args.output_kind)?;
    let timeout = Duration::from_secs(args.timeout.into());
    match args.mode {
        Mode::Server => match args.proto {
            Proto::Tcp => tcp::run_server(&args.path, timeout)?,
            Proto::Udp => udp::run_server(&args.path)?,
            Proto::Icmp => unimplemented!(),
        },
        Mode::Client => {
            let interval = Duration::from_secs_f64(args.interval);
            let client_options = args.to_client_options()?;
            let mut output = output::Output::create(
                args.output_kind,
                args.output_options.as_deref(),
                client_options.addr,
                args.proto,
                client_options.req.as_ref().map(Vec::len),
                interval,
                args.warn.map(Duration::from_secs_f64),
            )?;
            match args.proto {
                Proto::Tcp => tcp::run_client(&client_options, timeout, &mut output)?,
                Proto::Udp => udp::run_client(&client_options, timeout, &mut output)?,
                Proto::Icmp => icmp::run_client(&client_options, timeout, &mut output)?,
            }
        }
    }
    Ok(())
}
