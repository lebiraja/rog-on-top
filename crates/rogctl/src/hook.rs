use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;

use rog_fx::{ClaudeMessage, Command, Event};
use serde::Deserialize;

const CLAUDE_COMM: &str = "claude";
const MAX_ANCESTORS: usize = 8;

#[derive(Deserialize, Default)]
struct HookInput {
    session_id: Option<String>,
    transcript_path: Option<PathBuf>,
    notification_type: Option<String>,
}

/// Notifications that mean Claude is waiting on the user; others (idle reminders, auth) are not questions.
fn is_question(notification_type: Option<&str>) -> bool {
    notification_type.is_none_or(|t| matches!(t, "permission_prompt" | "elicitation_dialog"))
}

fn comm(pid: u32) -> Option<String> {
    fs::read_to_string(format!("/proc/{pid}/comm")).ok().map(|c| c.trim().to_owned())
}

fn parent_of(pid: u32) -> Option<u32> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    stat.rsplit_once(')')?.1.split_whitespace().nth(1)?.parse().ok()
}

/// The `claude` process that (transitively) spawned this hook, if any.
fn claude_ancestor() -> Option<u32> {
    let mut pid = std::os::unix::process::parent_id();
    for _ in 0..MAX_ANCESTORS {
        if pid <= 1 {
            return None;
        }
        if comm(pid)? == CLAUDE_COMM {
            return Some(pid);
        }
        pid = parent_of(pid)?;
    }
    None
}

/// Build the daemon command for a Claude Code hook: reads the hook JSON on stdin. `None` means ignore.
pub fn command(event: Event) -> Option<Command> {
    let mut raw = String::new();
    if !io::stdin().is_terminal() {
        let _ = io::stdin().read_to_string(&mut raw);
    }
    let input: HookInput = serde_json::from_str(&raw).unwrap_or_default();
    if event == Event::Ask && !is_question(input.notification_type.as_deref()) {
        return None;
    }
    Some(Command::Claude(ClaudeMessage {
        event,
        session: input.session_id.unwrap_or_else(|| "unknown".to_owned()),
        pid: claude_ancestor(),
        transcript: input.transcript_path,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_permission_and_elicitation_notifications_are_questions() {
        assert!(is_question(None));
        assert!(is_question(Some("permission_prompt")));
        assert!(is_question(Some("elicitation_dialog")));
        assert!(!is_question(Some("idle_prompt")));
        assert!(!is_question(Some("auth_success")));
    }
}
