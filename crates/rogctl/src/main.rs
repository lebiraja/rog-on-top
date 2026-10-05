mod hook;

use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand, ValueEnum};
use rog_fx::{Command, Effect, Event, Response};
use rog_hid::Rgb;

const IO_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Parser)]
#[command(name = "rogctl", about = "Control ROG laptop lighting through rogd")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Switch the running effect.
    #[command(subcommand)]
    Effect(EffectCmd),
    /// Set brightness, 0 to 100.
    Brightness { level: u8 },
    /// Print the current effect and brightness.
    State,
    /// Called by Claude Code hooks: reads the hook JSON on stdin and forwards the event to rogd.
    Hook { event: HookEvent },
}

#[derive(Clone, Copy, ValueEnum)]
enum HookEvent {
    Start,
    Working,
    Ask,
    Done,
    End,
}

impl From<HookEvent> for Event {
    fn from(e: HookEvent) -> Self {
        match e {
            HookEvent::Start => Event::Start,
            HookEvent::Working => Event::Working,
            HookEvent::Ask => Event::Ask,
            HookEvent::Done => Event::Done,
            HookEvent::End => Event::End,
        }
    }
}

#[derive(Subcommand)]
enum EffectCmd {
    Static { colour: Rgb },
    Breathe {
        colour: Rgb,
        #[arg(long, default_value_t = 4000)]
        period_ms: u64,
    },
    Rainbow {
        #[arg(long, default_value_t = 6000)]
        period_ms: u64,
        /// Hue cycles across each strip.
        #[arg(long, default_value_t = 1.0)]
        spread: f32,
    },
    Comet {
        colour: Rgb,
        #[arg(long, default_value_t = 2400)]
        period_ms: u64,
    },
}

impl From<EffectCmd> for Effect {
    fn from(e: EffectCmd) -> Self {
        match e {
            EffectCmd::Static { colour } => Effect::Static { colour },
            EffectCmd::Breathe { colour, period_ms } => Effect::Breathe { colour, period_ms },
            EffectCmd::Rainbow { period_ms, spread } => Effect::Rainbow { period_ms, spread },
            EffectCmd::Comet { colour, period_ms } => {
                Effect::Comet { colour, period_ms, tail: 0.45, floor: 0.05, tail_colour: None }
            }
        }
    }
}

fn socket_path() -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"));
    dir.join("rogd.sock")
}

fn send(cmd: &Command) -> io::Result<Response> {
    let mut stream = UnixStream::connect(socket_path())
        .map_err(|e| io::Error::new(e.kind(), format!("cannot reach rogd ({e}); is it running?")))?;
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    let mut line = serde_json::to_string(cmd).map_err(io::Error::other)?;
    line.push('\n');
    stream.write_all(line.as_bytes())?;
    let mut reply = String::new();
    BufReader::new(stream).read_line(&mut reply)?;
    serde_json::from_str(reply.trim()).map_err(io::Error::other)
}

fn main() -> ExitCode {
    let cmd = match Cli::parse().command {
        // Hooks must never fail or block Claude Code: swallow every error.
        Cmd::Hook { event } => {
            if let Some(cmd) = hook::command(event.into()) {
                let _ = send(&cmd);
            }
            return ExitCode::SUCCESS;
        }
        Cmd::Effect(e) => Command::SetEffect { effect: e.into() },
        Cmd::Brightness { level } => Command::SetBrightness { level },
        Cmd::State => Command::State,
    };
    match send(&cmd) {
        Ok(Response::Ok) => ExitCode::SUCCESS,
        Ok(Response::State { effect, brightness, .. }) => {
            println!("{}\nbrightness {brightness}", serde_json::to_string(&effect).unwrap_or_default());
            ExitCode::SUCCESS
        }
        Ok(other @ (Response::Presets { .. } | Response::Preset { .. })) => {
            println!("{}", serde_json::to_string(&other).unwrap_or_default());
            ExitCode::SUCCESS
        }
        Ok(Response::Error { message }) => {
            eprintln!("rogd: {message}");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("rogctl: {e}");
            ExitCode::FAILURE
        }
    }
}
