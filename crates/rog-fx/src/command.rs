use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use rog_hid::Rgb;

use crate::effects::Effect;
use crate::look::Preset;

pub use crate::look::MAX_BRIGHTNESS;

/// One request from a client (`rogctl`, later the web API) to `rogd`, sent as a single JSON line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Command {
    SetEffect { effect: Effect },
    SetBrightness { level: u8 },
    /// Replace the whole paint map (LED name to fixed colour); an empty map clears it.
    SetPaint { paint: BTreeMap<String, Rgb> },
    State,
    ListPresets,
    /// Save the current look under a name.
    SavePreset { name: String },
    ApplyPreset { id: String },
    DeletePreset { id: String },
    UpdatePreset { id: String, name: Option<String>, favourite: Option<bool> },
    /// The preset to load when rogd starts; `None` resumes the last look.
    SetDefaultPreset { id: Option<String> },
    /// A Claude Code hook event; sessions drive the lightbar overlay.
    Claude(ClaudeMessage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Event {
    Start,
    Working,
    Ask,
    Done,
    End,
}

/// One event from a Claude Code hook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaudeMessage {
    pub event: Event,
    pub session: String,
    /// The `claude` process that owns the session, used to drop sessions that die without a `SessionEnd`.
    pub pid: Option<u32>,
    /// The session transcript, used to notice interrupts (which fire no hook).
    pub transcript: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Response {
    Ok,
    State { effect: Effect, brightness: u8, paint: BTreeMap<String, Rgb> },
    Presets { presets: Vec<Preset>, default: Option<String> },
    Preset { preset: Preset },
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use rog_hid::colour::Rgb;

    #[test]
    fn set_effect_round_trips_through_json() {
        let cmd = Command::SetEffect { effect: Effect::Breathe { colour: Rgb::new(255, 0, 0), period_ms: 2000 } };

        let line = serde_json::to_string(&cmd).unwrap();

        assert_eq!(serde_json::from_str::<Command>(&line).unwrap(), cmd);
    }

    #[test]
    fn parses_the_hand_written_form() {
        let cmd: Command = serde_json::from_str(r#"{"cmd":"set_effect","effect":{"effect":"rainbow","period_ms":4000}}"#).unwrap();

        assert_eq!(cmd, Command::SetEffect { effect: Effect::Rainbow { period_ms: 4000, spread: 1.0 } });
    }

    #[test]
    fn claude_hook_message_parses_with_optional_fields_absent() {
        let cmd: Command = serde_json::from_str(r#"{"cmd":"claude","event":"done","session":"s"}"#).unwrap();

        assert_eq!(cmd, Command::Claude(ClaudeMessage { event: Event::Done, session: "s".into(), pid: None, transcript: None }));
    }

    #[test]
    fn rejects_unknown_commands() {
        assert!(serde_json::from_str::<Command>(r#"{"cmd":"explode"}"#).is_err());
    }
}
