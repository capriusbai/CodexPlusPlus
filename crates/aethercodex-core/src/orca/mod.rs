//! Read-only client for a locally installed Orca IDE.
//!
//! P0 of the AetherCodex × Orca integration: detect Orca, ask it questions,
//! and degrade silently when it is absent. See
//! `docs/design/aethercodex-orca-integration.md`.
//!
//! Two constraints from §0 of that design are structural here rather than left
//! to callers:
//!
//! - **Read-only.** [`Query`] is a closed set of `--json` query subcommands, so
//!   a write command cannot be sent by mistake. Write paths arrive in later
//!   phases, behind an explicit user action.
//! - **Never address GNOME Orca.** Candidate binaries are vetted in
//!   [`probe`]; a screen reader is refused before any subcommand is sent.
//!
//! When Orca is not installed, [`OrcaClient::detect`] returns `None` and every
//! capability built on it hides itself. That is the ordinary case, not an error.

pub mod command;
pub mod probe;

use std::time::Duration;

use serde_json::{Value, json};

pub use probe::{OrcaCli, VALIDATED_VERSION};

/// Orca's CLI starts an Electron process, so a query is slow before it is
/// broken. Generous enough for a cold start, short enough not to stall a UI.
const QUERY_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, thiserror::Error)]
pub enum OrcaError {
    #[error("Orca IDE is not installed, or its CLI could not be found")]
    NotInstalled,
    #[error("Cannot run the Orca CLI")]
    Spawn(#[source] std::io::Error),
    #[error("The Orca CLI did not answer within {0:?}")]
    TimedOut(Duration),
    #[error("The Orca CLI exited with {code}: {stderr}")]
    Failed { code: String, stderr: String },
    #[error("Cannot parse the Orca CLI's JSON output")]
    Parse(#[source] serde_json::Error),
    /// Orca answered, and its answer says the request did not succeed.
    #[error("Orca reported failure: {0}")]
    Reported(String),
}

/// The read-only queries P0 may send. A closed set by design: see the module note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Query {
    /// Which AI accounts Orca has, and which Codex home they resolve to.
    AccountList,
    /// The projects Orca manages.
    ProjectList,
    /// Orchestration runs, including any instance-level coordinator.
    OrchestrationRunList,
}

impl Query {
    /// The subcommand words, without `--json`.
    pub fn args(self) -> &'static [&'static str] {
        match self {
            Self::AccountList => &["account", "list"],
            Self::ProjectList => &["project", "list"],
            Self::OrchestrationRunList => &["orchestration", "run-list"],
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrcaClient {
    cli: OrcaCli,
    timeout: Duration,
}

impl OrcaClient {
    /// Find a local Orca, or `None` when there is none to talk to.
    pub fn detect() -> Option<Self> {
        probe::probe().map(|cli| Self {
            cli,
            timeout: QUERY_TIMEOUT,
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn cli(&self) -> &OrcaCli {
        &self.cli
    }

    /// Send one read-only query and return its `result`.
    pub fn query(&self, query: Query) -> Result<Value, OrcaError> {
        let mut args: Vec<&str> = query.args().to_vec();
        args.push("--json");
        let output =
            command::run(&self.cli.program, &args, self.timeout).map_err(|error| match error {
                command::RunError::Spawn(error) => OrcaError::Spawn(error),
                command::RunError::TimedOut => OrcaError::TimedOut(self.timeout),
                command::RunError::Failed { code, stderr } => OrcaError::Failed {
                    code: code
                        .map(|code| code.to_string())
                        .unwrap_or_else(|| "signal".into()),
                    stderr: first_lines(&stderr, 3),
                },
            })?;
        unwrap_envelope(&output.stdout)
    }
}

/// Unwrap Orca's `{ id, ok, result }` response envelope.
///
/// `ok: false` is a reported failure rather than a transport error, so it is
/// surfaced with whatever Orca said about it instead of an empty result.
fn unwrap_envelope(stdout: &str) -> Result<Value, OrcaError> {
    let parsed: Value = serde_json::from_str(stdout.trim()).map_err(OrcaError::Parse)?;
    if parsed.get("ok").and_then(Value::as_bool) == Some(false) {
        let reason = parsed
            .get("error")
            .map(render_reason)
            .unwrap_or_else(|| "no reason given".to_string());
        return Err(OrcaError::Reported(reason));
    }
    Ok(parsed
        .get("result")
        .cloned()
        // A response without `result` is still a successful answer; hand back
        // the whole document rather than inventing an error.
        .unwrap_or(parsed))
}

fn render_reason(value: &Value) -> String {
    value
        .get("message")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn first_lines(text: &str, count: usize) -> String {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .take(count)
        .collect::<Vec<_>>()
        .join("; ")
}

/// What the UI needs to decide whether to show anything Orca-related.
///
/// Shaped like `zed_remote::zed_remote_status` so the manager treats both
/// external tools the same way.
pub fn orca_status() -> Value {
    match OrcaClient::detect() {
        None => json!({
            "status": "ok",
            "installed": false,
            "cliPath": "",
            "version": "",
            "olderThanValidated": false,
            "validatedVersion": VALIDATED_VERSION,
        }),
        Some(client) => {
            let cli = client.cli();
            let version = cli.version.clone().unwrap_or_default();
            json!({
                "status": "ok",
                "installed": true,
                "cliPath": cli.program,
                "version": version,
                "olderThanValidated": probe::older_than_validated(&version),
                "validatedVersion": VALIDATED_VERSION,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_read_only_queries_can_be_sent() {
        // The guarantee: no Query spells a command that changes Orca's state.
        // Anything mutating would have to be added to the enum, which is the
        // review point this test creates.
        let mutating = [
            "create",
            "delete",
            "remove",
            "set",
            "write",
            "add",
            "update",
            "dispatch",
            "start",
            "stop",
            "kill",
            "run-create",
            "login",
            "logout",
        ];
        for query in [
            Query::AccountList,
            Query::ProjectList,
            Query::OrchestrationRunList,
        ] {
            for word in query.args() {
                assert!(
                    !mutating.contains(word),
                    "{word:?} mutates Orca state and does not belong in a P0 query"
                );
            }
        }
    }

    #[test]
    fn a_successful_envelope_is_unwrapped_to_its_result() {
        let stdout = r#"{"id":"abc","ok":true,"result":{"projects":[{"displayName":"x"}]}}"#;
        let result = unwrap_envelope(stdout).unwrap();
        assert_eq!(result["projects"][0]["displayName"], "x");
    }

    #[test]
    fn a_reported_failure_carries_orcas_own_reason() {
        let stdout = r#"{"id":"abc","ok":false,"error":{"message":"no active account"}}"#;
        match unwrap_envelope(stdout) {
            Err(OrcaError::Reported(reason)) => assert_eq!(reason, "no active account"),
            other => panic!("expected a reported failure, got {other:?}"),
        }
    }

    #[test]
    fn a_failure_without_a_message_still_reports_something() {
        match unwrap_envelope(r#"{"ok":false}"#) {
            Err(OrcaError::Reported(reason)) => assert!(!reason.is_empty()),
            other => panic!("expected a reported failure, got {other:?}"),
        }
    }

    #[test]
    fn an_answer_without_a_result_key_is_not_an_error() {
        let result = unwrap_envelope(r#"{"ok":true,"runs":[]}"#).unwrap();
        assert!(result.get("runs").is_some());
    }

    #[test]
    fn non_json_output_is_a_parse_error() {
        assert!(matches!(
            unwrap_envelope("command not found"),
            Err(OrcaError::Parse(_))
        ));
    }

    #[test]
    fn status_is_reported_without_orca_installed() {
        // Never an error and never a panic: the absent case is ordinary.
        let status = orca_status();
        assert_eq!(status["status"], "ok");
        assert!(status["installed"].is_boolean());
        assert_eq!(status["validatedVersion"], VALIDATED_VERSION);
    }

    #[test]
    fn stderr_is_trimmed_to_the_first_few_lines() {
        let noisy = "\n\nfirst\nsecond\nthird\nfourth\n";
        assert_eq!(first_lines(noisy, 3), "first; second; third");
    }
}
