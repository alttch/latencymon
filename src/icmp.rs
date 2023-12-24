use crate::output::{clear_line, Output};
use crate::Proto;
use eva_common::Error;
use log::error;
use std::net::{SocketAddr, ToSocketAddrs};
use std::thread;
use std::time::Duration;

pub fn run_client_session(
    addr: SocketAddr,
    timeout: Duration,
    interval: Duration,
    warn: Option<f64>,
    chart: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = Output::new(
        addr,
        Proto::Icmp,
        None,
        interval,
        warn.map(Duration::from_secs_f64),
        chart,
    );
    loop {
        ping::ping(addr.ip(), Some(timeout), None, None, None, None)?;
        output.finish_iteration()?;
    }
}

pub fn run_client(
    path: &str,
    timeout: Duration,
    interval_sec: f64,
    warn: Option<f64>,
    chart: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = (path, 0)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| Error::invalid_params("invalid socket addr"))?;
    let interval = Duration::from_secs_f64(interval_sec);
    loop {
        if let Err(e) = run_client_session(addr, timeout, interval, warn, chart) {
            clear_line();
            error!("{}", e);
        }
        thread::sleep(interval);
    }
}
