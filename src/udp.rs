use crate::output::Output;
use crate::{ClientOptions, MAX_UDP_FRAME_SIZE};
use eva_common::Error;
use log::info;
use std::net::{SocketAddr, UdpSocket};
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
    opts: &ClientOptions,
    timeout: Duration,
    output: &mut Output,
) -> Result<(), Box<dyn std::error::Error>> {
    let req = opts.req.as_ref().unwrap();
    loop {
        if let Err(e) = run_client_session(opts.addr, timeout, req, output) {
            output.log_iteration(Some(e))?;
        }
    }
}
