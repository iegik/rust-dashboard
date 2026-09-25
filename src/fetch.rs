use std::io::Read;
use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;

pub fn run_command(cmd: &str, timeout: Duration) -> Result<String, String> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn: {e}"))?;

    match child
        .wait_timeout(timeout)
        .map_err(|e| format!("wait: {e}"))?
    {
        None => {
            let _ = child.kill();
            let _ = child.wait();
            Err(format!("timed out after {}ms", timeout.as_millis()))
        }
        Some(status) => {
            let mut stdout = String::new();
            let mut stderr = String::new();
            if let Some(mut out) = child.stdout.take() {
                let _ = out.read_to_string(&mut stdout);
            }
            if let Some(mut err) = child.stderr.take() {
                let _ = err.read_to_string(&mut stderr);
            }
            if status.success() {
                Ok(stdout)
            } else {
                let code = status.code().unwrap_or(-1);
                let err = stderr.trim();
                if err.is_empty() {
                    Err(format!("exit {code}"))
                } else {
                    Err(format!("exit {code}: {err}"))
                }
            }
        }
    }
}
