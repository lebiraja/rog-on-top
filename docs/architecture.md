# Architecture

## Overview
`rogd` is a native user-level Rust daemon that is the single writer to a ROG laptop's Aura HID device. Clients (`rogctl`, Claude Code hooks, later a web UI) send JSON-line commands over a unix socket; the daemon renders effects at 20 fps and overlays Claude Code session state on the lightbar. Actors: the user, Claude Code hooks, asusd (coexisting, must not drive Aura).

## Tech Stack
- Rust 2021 workspace (std threads, no async yet), clap, serde/serde_json, toml, ctrlc
- Device I/O: raw `/dev/hidraw*` (64-byte reports), udev `uaccess`
- Runtime: systemd user service (native, not Docker: needs hidraw and the user session)
- HTTP: axum 0.8 on a tokio current-thread runtime in its own thread; UI embedded with rust-embed
- UI: Vite + React 19 + TypeScript strict + Tailwind 4 + Zod (`web/`)
- Planned: PipeWire/rustfft, xdg-desktop-portal ScreenCast

## Services & Responsibilities
- `rog-hid`: profile TOML, LED→packet offsets, `Frame`, `HidrawDevice`/`HidSink`, `Rgb`
- `rog-fx`: pure effect model (`Effect::colour_at`), `Scenes`, wire types `Command`/`Response`
- `rogd`: engine (effect+brightness+Claude overlay), render loop, socket server
- `rogd::http`: REST API + embedded UI on 127.0.0.1:7777, Origin/Host guard, shares the Engine with the socket
- `web/`: control UI (effect list, colour, cycle, brightness, live preview)
- `rogctl`: CLI client; `rogctl hook <event>` is the Claude Code hook entrypoint

## Full Application Flow
```text
 Claude Code hook ──► rogctl hook ─┐
 rogctl effect/...  ───────────────┤ JSON line
 Browser UI ─ HTTP :7777 ──────────┤ (same Engine)
                                   ↓
                     $XDG_RUNTIME_DIR/rogd.sock
                                   ↓
              ┌───────────────── rogd ─────────────────┐
              │ socket::serve → Engine::apply          │
              │ render loop (50 ms):                   │
              │   Engine::targets                      │
              │     keys+bar ← base Effect × brightness│
              │     bar      ← Claude sessions overlay │
              │   smoothing → Frame::packet            │
              └───────────────────┬────────────────────┘
                                  ↓
                     /dev/hidraw0 (0b05:19b6)
                                  ↓
                  keyboard zones + lightbar LEDs
```

## Backend Architecture
```text
 main (loop, device, restore on exit)
   ├─ socket  (bind 0600, JSON-line serve)
   ├─ engine  (Command → state; targets per LED)
   │    ├─ sessions (Claude slots/states, sweep)
   │    ├─ claude   (bar_targets: per-session segments)
   │    └─ rog-fx   (Effect, Scenes)
   ├─ procfs  (claude pid alive, transcript interrupt)
   └─ rog-hid (Profile → Frame → HidrawDevice)
```

## Data Models
`Profile` (vid/pid, packet spec, restore packets, LEDs with group+offset), `Effect` (static, breathe, pulse, rainbow, comet), `Command`/`Response`, `ClaudeMessage{event,session,pid,transcript}`, `Sessions` (slot, state, since).

## External Integrations
Claude Code hooks (`~/.claude/settings.json` → `rogctl hook`), asusd (conflict warning only), kernel hidraw.

## Environment & Config
`XDG_RUNTIME_DIR` (socket dir). No secrets. Profile: `data/profiles/g614ju.toml`; scenes: `data/scenes_default.toml`. Unknown `[?]`: user config dir (`~/.config/rog-on-top/`) not used yet.
