use crate::output::Output;
use crate::ClientOptions;
use std::time::Duration;

pub fn run_client(
    opts: &ClientOptions,
    timeout: Duration,
    output: &mut Output,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let res = ping::ping(opts.addr.ip(), Some(timeout), None, None, None, None);
        output.log_iteration(res.err().map(Into::into))?;
    }
}
