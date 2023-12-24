use crate::output::{self, Output};
use crate::{Proto, MAX_UDP_FRAME_SIZE, MIN_FRAME_SIZE};
use eva_common::Error;
use log::info;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Duration;

pub fn run_server(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("UDP listening at {}", path);
    let sock = UdpSocket::bind(path)?;
    let mut buf = vec![0; MAX_UDP_FRAME_SIZE];
    while let Ok((size, addr)) = sock.recv_from(&mut buf) {
        sock.send_to(&buf[..size], addr)?;
    }
    Ok(())
}

fn run_client_session(
    addr: SocketAddr,
    timeout: Duration,
    req: &[u8],
    output: &mut Output,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = UdpSocket::bind("0.0.0.0:0")?;
    conn.set_read_timeout(Some(timeout))?;
    conn.set_write_timeout(Some(timeout))?;
    let mut response_buf = vec![0_u8; req.len()];
    output.reset();
    loop {
        conn.send_to(req, addr)?;
        let size = conn.recv(&mut response_buf)?;
        if size != req.len() || req != response_buf {
            return Err(Error::invalid_data("invalid packet").into());
        }
        output.log_iteration(None)?;
    }
}

pub fn run_client(
    path: &str,
    timeout: Duration,
    frame_size_bytes: u32,
    interval_sec: f64,
    warn: Option<f64>,
    output_kind: output::Kind,
) -> Result<(), Box<dyn std::error::Error>> {
    let frame_size = usize::try_from(frame_size_bytes)?;
    if frame_size < MIN_FRAME_SIZE {
        return Err(Error::invalid_data(format!("invalid frame size: {}", frame_size)).into());
    }
    let req = crate::create_frame(frame_size);
    let addr: SocketAddr = path
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| Error::invalid_params("invalid socket addr"))?;
    let interval = Duration::from_secs_f64(interval_sec);
    let mut output = Output::new(
        output_kind,
        addr,
        Proto::Udp,
        Some(frame_size),
        interval,
        warn.map(Duration::from_secs_f64),
    );
    loop {
        if let Err(e) = run_client_session(addr, timeout, &req, &mut output) {
            output.log_iteration(Some(e))?;
        }
    }
}
