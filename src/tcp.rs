use crate::output::Output;
use crate::{ClientOptions, HELLO, MIN_FRAME_SIZE};
use eva_common::Error;
use log::info;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

fn handle_server_stream(
    mut conn: TcpStream,
    addr: SocketAddr,
    timeout: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    conn.set_nodelay(true)?;
    conn.set_read_timeout(Some(timeout))?;
    conn.set_write_timeout(Some(timeout))?;
    conn.write_all(&[HELLO])?;
    let mut buf = vec![0_u8; 5];
    conn.read_exact(&mut buf)?;
    if buf[0] != HELLO {
        return Err(Error::invalid_data("invalid hello").into());
    }
    let frame_size = usize::try_from(u32::from_le_bytes(buf[1..].try_into().unwrap()))?;
    info!("{} frame size: {} bytes", addr, frame_size);
    if frame_size < MIN_FRAME_SIZE {
        return Err(Error::invalid_data(format!("invalid frame size: {}", frame_size)).into());
    }
    let mut buf = vec![0_u8; frame_size];
    loop {
        if let Err(e) = conn.read_exact(&mut buf) {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                return Ok(());
            }
            return Err(e.into());
        }
        conn.write_all(&buf)?;
    }
}

pub fn run_server(path: &str, timeout: Duration) -> Result<(), Box<dyn std::error::Error>> {
    info!("TCP listening at {}, timeout: {:?}", path, timeout);
    let srv = TcpListener::bind(path)?;
    while let Ok((conn, addr)) = srv.accept() {
        info!("{}: connected", addr);
        thread::spawn(move || {
            if let Err(e) = handle_server_stream(conn, addr, timeout) {
                log::error!("{}: {}", addr, e);
            } else {
                info!("{}: disconnected", addr);
            }
        });
    }
    Ok(())
}

pub fn run_client_session(
    addr: SocketAddr,
    timeout: Duration,
    req: &[u8],
    output: &mut Output,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = TcpStream::connect_timeout(&addr, timeout)?;
    conn.set_nodelay(true)?;
    conn.set_read_timeout(Some(timeout))?;
    conn.set_write_timeout(Some(timeout))?;
    let mut buf = [0u8];
    conn.read_exact(&mut buf)?;
    if buf[0] != HELLO {
        return Err(Error::invalid_data("invalid hello").into());
    }
    let mut buf = vec![HELLO];
    buf.extend(u32::try_from(req.len())?.to_le_bytes());
    conn.write_all(&buf)?;
    info!("connected");
    let mut response_buf = vec![0_u8; req.len()];
    output.reset();
    loop {
        conn.write_all(req)?;
        conn.read_exact(&mut response_buf)?;
        if req != response_buf {
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
