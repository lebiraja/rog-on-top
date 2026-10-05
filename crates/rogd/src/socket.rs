use std::fs::{self, Permissions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

use rog_fx::{Command, Response};

use crate::engine::Engine;

const CLIENT_READ_TIMEOUT: Duration = Duration::from_secs(2);
const CLIENT_MAX_BYTES: u64 = 64 * 1024;

pub fn path() -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"));
    dir.join("rogd.sock")
}

pub fn bind(path: &Path) -> io::Result<UnixListener> {
    let _ = fs::remove_file(path);
    let listener = UnixListener::bind(path)?;
    fs::set_permissions(path, Permissions::from_mode(0o600))?;
    Ok(listener)
}

/// Answer each JSON-line command on the stream with one JSON-line response.
pub fn serve(mut stream: UnixStream, engine: &Mutex<Engine>) {
    if stream.set_read_timeout(Some(CLIENT_READ_TIMEOUT)).is_err() {
        return;
    }
    let Ok(reader) = stream.try_clone() else { return };
    for line in BufReader::new(reader.take(CLIENT_MAX_BYTES)).lines().map_while(Result::ok) {
        let reply = respond(&line, engine);
        let Ok(mut out) = serde_json::to_string(&reply) else { return };
        out.push('\n');
        if stream.write_all(out.as_bytes()).is_err() {
            return;
        }
    }
}

fn respond(line: &str, engine: &Mutex<Engine>) -> Response {
    match serde_json::from_str::<Command>(line.trim()) {
        Ok(cmd) => engine.lock().unwrap_or_else(|e| e.into_inner()).apply(cmd, Instant::now(), SystemTime::now()),
        Err(e) => Response::Error { message: format!("bad command: {e}") },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rog_fx::Effect;
    use rog_hid::Rgb;

    fn engine() -> Mutex<Engine> {
        Mutex::new(Engine::new(Effect::Static { colour: Rgb::BLACK }))
    }

    #[test]
    fn a_valid_command_is_applied() {
        let e = engine();

        let reply = respond(r#"{"cmd":"set_brightness","level":30}"#, &e);

        assert_eq!(reply, Response::Ok);
        assert!(matches!(respond(r#"{"cmd":"state"}"#, &e), Response::State { brightness: 30, .. }));
    }

    #[test]
    fn garbage_gets_an_error_reply_not_a_panic() {
        let reply = respond("not json", &engine());

        assert!(matches!(reply, Response::Error { .. }));
    }
}
