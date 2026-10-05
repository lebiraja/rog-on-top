use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use rog_fx::{Look, Preset};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::engine::{default_look, Engine};

const STATE_FILE: &str = "state.json";
const PRESETS_FILE: &str = "presets.json";

#[derive(Serialize, Deserialize)]
struct StateFile {
    look: Look,
    default_preset: Option<String>,
}

/// JSON files under `~/.config/rog-on-top/`: the last look and the saved presets.
pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn default_dir() -> PathBuf {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("rog-on-top")
    }

    pub fn load(&self) -> Engine {
        let state: Option<StateFile> = read_json(&self.dir.join(STATE_FILE));
        let presets: Vec<Preset> = read_json(&self.dir.join(PRESETS_FILE)).unwrap_or_default();
        let (last, default) = state.map_or((default_look(), None), |s| (s.look, s.default_preset));
        Engine::restore(last, presets, default)
    }

    pub fn save(&self, engine: &Engine) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let state = StateFile { look: engine.look().clone(), default_preset: engine.default_preset().map(str::to_owned) };
        write_json(&self.dir.join(STATE_FILE), &state)?;
        write_json(&self.dir.join(PRESETS_FILE), &engine.presets())
    }
}

/// A missing file is normal; an unreadable one is set aside as `.bad` so it is never silently overwritten.
fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let raw = fs::read_to_string(path).ok()?;
    match serde_json::from_str(&raw) {
        Ok(value) => Some(value),
        Err(e) => {
            let bad = path.with_extension("json.bad");
            eprintln!("{} is invalid ({e}); moved to {}", path.display(), bad.display());
            let _ = fs::rename(path, bad);
            None
        }
    }
}

fn write_json(path: &Path, value: &impl Serialize) -> io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(value).map_err(io::Error::other)?)?;
    fs::rename(tmp, path)
}

#[cfg(test)]
mod tests {
    use std::time::{Instant, SystemTime};

    use rog_fx::{Command, Effect, Response};
    use rog_hid::Rgb;

    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rogd-store-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn apply(engine: &mut Engine, cmd: Command) -> Response {
        engine.apply(cmd, Instant::now(), SystemTime::now())
    }

    #[test]
    fn first_run_starts_from_the_default_look() {
        let store = Store::new(scratch("first"));

        assert_eq!(store.load().look(), &default_look());
    }

    #[test]
    fn look_presets_and_default_survive_a_restart() {
        let store = Store::new(scratch("roundtrip"));
        let mut engine = store.load();
        apply(&mut engine, Command::SetEffect { effect: Effect::Static { colour: Rgb::new(1, 2, 3) } });
        apply(&mut engine, Command::SetBrightness { level: 40 });
        let Response::Preset { preset } = apply(&mut engine, Command::SavePreset { name: "Dim".into() }) else { panic!("preset") };
        apply(&mut engine, Command::SetEffect { effect: Effect::Static { colour: Rgb::new(9, 9, 9) } });
        apply(&mut engine, Command::SetDefaultPreset { id: Some(preset.id.clone()) });
        store.save(&engine).unwrap();

        let reloaded = store.load();

        assert_eq!(reloaded.presets(), engine.presets());
        assert_eq!(reloaded.default_preset(), Some(preset.id.as_str()));
        assert_eq!(reloaded.look(), &preset.look);
    }

    #[test]
    fn a_corrupt_file_is_set_aside_not_overwritten() {
        let dir = scratch("corrupt");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(STATE_FILE), "{ not json").unwrap();
        let store = Store::new(dir.clone());

        let engine = store.load();

        assert_eq!(engine.look(), &default_look());
        assert!(dir.join("state.json.bad").exists());
        assert!(!dir.join(STATE_FILE).exists());
    }
}
