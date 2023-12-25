use crate::output::{self, Output};
use crate::Proto;
use eva_common::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

pub fn run_client(
    path: &str,
    timeout: Duration,
    interval_sec: f64,
    warn: Option<f64>,
    output_kind: output::Kind,
    output_options: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = (path, 0)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| Error::invalid_params("invalid socket addr"))?;
    let interval = Duration::from_secs_f64(interval_sec);
    let mut output = Output::create(
        output_kind,
        output_options,
        addr,
        Proto::Icmp,
        None,
        interval,
        warn.map(Duration::from_secs_f64),
    )?;
    loop {
        let res = ping::ping(addr.ip(), Some(timeout), None, None, None, None);
        output.log_iteration(res.err().map(Into::into))?;
    }
}
