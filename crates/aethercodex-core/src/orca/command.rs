//! Run a short-lived child process and give up after a deadline.
//!
//! Orca's CLI is a full Electron binary. A query that would normally answer in
//! under a second can hang on a stale lockfile or a half-started app, and a
//! hung probe must never hang AetherCodex, so every call has a deadline.
//!
//! Output is read on its own thread rather than after waiting: `orca-ide
//! project list --json` embeds base64 repository icons and runs to megabytes,
//! which fills the pipe buffer and blocks the child before it can exit.
//! Waiting first would therefore time out on exactly the calls that work.

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Cap on what a single query may return. Comfortably above a project list
/// with icons, far below anything that would strain memory.
const MAX_OUTPUT_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug)]
pub enum RunError {
    Spawn(std::io::Error),
    TimedOut,
    /// The process ran but reported failure. Holds its own diagnostics.
    Failed {
        code: Option<i32>,
        stderr: String,
    },
}

#[derive(Debug)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
}

/// Run `program` with `args`, returning its output or giving up after `timeout`.
pub fn run(program: &str, args: &[&str], timeout: Duration) -> Result<Output, RunError> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(RunError::Spawn)?;

    // One trait object per stream so both are drained by the same loop; a
    // ChildStdout and a ChildStderr are different types but read identically.
    let stdout: Option<Box<dyn Read + Send>> = child
        .stdout
        .take()
        .map(|s| Box::new(s) as Box<dyn Read + Send>);
    let stderr: Option<Box<dyn Read + Send>> = child
        .stderr
        .take()
        .map(|s| Box::new(s) as Box<dyn Read + Send>);
    let (tx, rx) = mpsc::channel();

    for (stream, is_stdout) in [(stdout, true), (stderr, false)] {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut text = String::new();
            if let Some(mut stream) = stream {
                // Lossy so a truncated multi-byte sequence at the cap cannot
                // turn a working query into a read error.
                let mut bytes = Vec::new();
                let _ = stream
                    .as_mut()
                    .take(MAX_OUTPUT_BYTES)
                    .read_to_end(&mut bytes);
                text = String::from_utf8_lossy(&bytes).into_owned();
            }
            let _ = tx.send((is_stdout, text));
        });
    }
    drop(tx);

    let deadline = std::time::Instant::now() + timeout;
    let mut out = String::new();
    let mut err = String::new();
    let mut received = 0;
    while received < 2 {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        match rx.recv_timeout(remaining) {
            Ok((true, text)) => {
                out = text;
                received += 1;
            }
            Ok((false, text)) => {
                err = text;
                received += 1;
            }
            Err(_) => {
                // Killing our own query process is not the same as touching
                // Orca itself: this child is a CLI invocation we spawned, and
                // the running Orca app is never signalled.
                let _ = child.kill();
                let _ = child.wait();
                return Err(RunError::TimedOut);
            }
        }
    }

    let status = match child.wait() {
        Ok(status) => status,
        Err(error) => return Err(RunError::Spawn(error)),
    };
    if !status.success() {
        return Err(RunError::Failed {
            code: status.code(),
            stderr: err,
        });
    }
    Ok(Output {
        stdout: out,
        stderr: err,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_program_is_a_spawn_error_not_a_panic() {
        let error = run("aethercodex-no-such-program", &[], Duration::from_secs(1));
        assert!(matches!(error, Err(RunError::Spawn(_))));
    }

    #[cfg(unix)]
    #[test]
    fn output_larger_than_the_pipe_buffer_still_comes_back() {
        // The regression this guards: waiting on the child before draining its
        // pipe deadlocks once output exceeds the buffer, so a working query
        // would look like a timeout.
        let out = run(
            "/bin/sh",
            &["-c", "yes abcdefghij | head -c 1000000"],
            Duration::from_secs(20),
        )
        .expect("large output should be read, not time out");
        assert_eq!(out.stdout.len(), 1_000_000);
    }

    #[cfg(unix)]
    #[test]
    fn a_hanging_process_is_killed_at_the_deadline() {
        let started = std::time::Instant::now();
        let error = run("/bin/sh", &["-c", "sleep 30"], Duration::from_millis(300));
        assert!(matches!(error, Err(RunError::TimedOut)));
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "should return at the deadline, not wait out the process"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_failing_process_reports_its_code_and_stderr() {
        let error = run(
            "/bin/sh",
            &["-c", "echo trouble >&2; exit 3"],
            Duration::from_secs(10),
        );
        match error {
            Err(RunError::Failed { code, stderr }) => {
                assert_eq!(code, Some(3));
                assert!(stderr.contains("trouble"));
            }
            other => panic!("expected a failure, got {other:?}"),
        }
    }
}
