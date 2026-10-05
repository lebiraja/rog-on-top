# rog-on-top — Plan

Universal RGB control for ROG laptops (keyboard + lightbar + extra zones) with a local web UI on `http://127.0.0.1:7777`.
Feature parity target: Armoury Crate / Aura Sync, including Adaptive (screen colour) and Music (audio reactive).

## Decisions (made with Lebi, 2026-10-05)
| Topic | Decision |
|---|---|
| Hardware | ROG laptops only (built-in keyboard, lightbar, logo/lid where present). No USB peripherals in v1. |
| Daemon | New Rust daemon `rogd` (not asusd, not Python). Our own HID driver, not a D-Bus wrapper. |
| Features v1 | Core effects + colour picker, advanced effects, per-key editor, automation, Adaptive (screen), Music (audio). |
| Deploy | Native systemd service (overrides the Docker-everywhere default: hidraw + screen/audio capture need the host session). |
| UI | Embedded static web app served by `rogd` on `127.0.0.1:7777`. Brutalist/dark, not a card grid. |

## What we reuse from claude-glow (`reference/`)
| Asset | Use |
|---|---|
| `aura-hid` crate (profile, frame, device, colour, effects) | Becomes `rog-hid` core: TOML profiles, hidraw discovery by VID:PID, 64-byte frame packing, `Rgb` lerp/scale, `colour_at(t,pos)` effect model |
| `docs/protocol-g614ju.md` | Verified protocol: `5d bc` init, zoned direct frame `5d bc 01 01 04`, stream every 50 ms, firmware reverts when frames stop, restore via `5d b3/b5/b4` |
| G614JU profile (`g614ju.toml`) | First device profile. 4 keyboard zones + 6 lightbar LEDs |
| Daemon loop (50 ms, crossfade, device reconnect w/ backoff, single instance, restore on exit) | Same render loop, generalised to a layer compositor |
| udev rule + systemd unit | Per-model udev generated from profiles |
| `data/aura_support.ron` (asusd, 140 laptop models) | Seed for the per-model capability DB (modes, zones, layout, per-key vs zoned) |

Not reused: Claude session state machine and hooks (kept as an optional later integration: a `claude` source layer).

## Hard facts and risks (read before building)
1. **Only G614JU is verified.** "Universal" = a profile DB + a probe tool + capability-driven UI. Other models ship as `experimental` until someone confirms them on hardware.
2. **Per-key addressing is unmapped** on G614JU (it is a 4-zone keyboard; per-key packets lit everything). The per-key editor is enabled only for models whose profile says `layout = per_key`. Protocol for those comes from asusd's `rog-aura` crate (`/mnt/DataSSD/.../armoury-crate-linux/rog-aura`), to be studied in Phase 2.
3. **Coexistence with asusd.** Both write the same hidraw. `rogd` must detect/stop conflicting asusd streaming, or document "disable asusd aura". Brightness/power-state keys go through `asus::kbd_backlight` sysfs, which we can keep using for brightness.
4. **Adaptive on Wayland** (this machine is Wayland): needs `xdg-desktop-portal` ScreenCast + PipeWire, which prompts for permission once and lives in the user session. So the capture worker runs as a *user* service; hardware daemon can be a system or user service (user + `uaccess` udev, as now).
5. **Music**: capture the default sink monitor via PipeWire/PulseAudio, FFT (rustfft) → bass/mid/treble bands + beat detection.
6. **Protocol safety**: never send `0x5a` feature reports (firmware territory). Frame rate capped; restore packet always sent on clean exit.

## Architecture
```text
 Browser :7777            rogd (Rust, one user systemd service)
┌────────────┐  HTTP/WS  ┌──────────────────────────────────────────────┐
│ Web UI     │ ◄───────► │ api (axum): REST + WebSocket live preview    │
└────────────┘           │        │ commands                            │
                         │        ↓                                     │
 rogctl (CLI) ─ unix ──► │ engine: layers → compositor → 50 ms render   │
                         │   layers (priority, blend):                  │
                         │    base effect ← profile/automation          │
                         │    adaptive  ← screen worker (portal+PW)     │
                         │    music     ← audio worker (PW + FFT)       │
                         │    overlay   ← claude/notifications (later)  │
                         │        │ Frame (per-LED RGB)                 │
                         │        ↓                                     │
                         │ rog-hid: profile → packet → /dev/hidrawN     │
                         └──────────────────────┬───────────────────────┘
                                                ↓
                              keyboard zones/keys + lightbar (+ logo, lid)
```
Workspace layout:
```text
crates/rog-hid     profile, packet, hidraw, Rgb (from aura-hid)
crates/rog-fx      effect model + library, pure functions, no IO
crates/rog-probe   DMI detect, hidraw enumerate, per-LED probe CLI, profile generator
crates/rog-capture screen sampler (portal/PipeWire) and audio analyser (PipeWire/FFT)
crates/rogd        engine, compositor, automation, API server, embedded UI
crates/rogctl      thin CLI over the unix socket
web/               UI source (Vite + React + TS + Tailwind + shadcn), built into rogd via rust-embed
data/              profiles/*.toml, models.ron (from aura_support.ron), presets
packaging/         udev rules, systemd user unit, install script
```

## Effect catalogue (parity with Armoury Crate)
- **Basic** (also writable to firmware where supported): Static, Breathing, Colour Cycle, Rainbow (wave), Strobing/Flash.
- **Advanced** (software-rendered via streaming): Comet (have), Rain, Ripple, Starry Night, Reactive (key press), Laser, Fire, Gradient, Wave with direction/speed.
- **Adaptive**: screen → average/dominant colour per zone (left/right/top mapping to keyboard zones and bar halves), smoothing, saturation boost, update ≤ 30 fps at 64×36 downsample.
- **Music**: spectrum-bars across zones, beat flash, colour-by-pitch, bass pulse, sensitivity + smoothing.
- Controls: colour picker (HSV wheel, hex, palettes, 2–4 colour gradients), brightness, speed, direction, per-zone targeting, sync-all toggle.

## API (v1 draft, versioned `/api/v1`)
```text
GET  /devices                   detected devices, profile, capabilities, status
GET  /state                     current layers, effect, brightness
PUT  /effect                    {zones, effect, params}
PUT  /brightness                {level}
PUT  /keys                      {key_id: rgb}   (per_key models only)
PUT  /adaptive | /music         {enabled, settings}
GET/PUT/DELETE /profiles/{id}   saved looks; POST /profiles/{id}/apply
GET/PUT /automation             rules (AC/battery, power profile, app, time, boot/sleep)
WS   /live                      frame stream for the UI preview + state events
```
Bind `127.0.0.1` only; strict CORS/Origin check (a web page on the internet must not be able to drive the LEDs); config in `~/.config/rog-on-top/`.

## Phases
| # | Deliverable | Done when |
|---|---|---|
| 0 | Repo scaffold, workspace, port `aura-hid` → `rog-hid`, CI-less `cargo test` green | tests from claude-glow pass in new crates |
| 1 | `rogd` core: render loop, layer compositor, Static/Breathe/Rainbow/Comet on G614JU, restore on exit, systemd unit, udev | `rogctl effect rainbow` works on hardware |
| 2 | Capability DB from `aura_support.ron`, DMI auto-detect, profile loader, `rog-probe` (probe + generate a profile); study asusd `rog-aura` for per-key + other PIDs | G614JU auto-detected, unknown model gives a probe walkthrough |
| 3 | API + WebSocket on :7777, UI v1: device header, effect grid, colour picker, brightness/speed, live preview | full control from browser |
| 4 | Advanced effect library + per-zone targeting + saved profiles | all effects selectable in UI |
| 5 | Music mode | bars react to system audio |
| 6 | Adaptive screen mode (Wayland portal; X11 fallback) | zones follow on-screen colours |
| 7 | Automation (AC/battery, power profile, boot/sleep/shutdown, app rules), Claude layer | rules fire; resume after suspend |
| 8 | Per-key editor (per_key models), import/export, docs, packaging | ships with install script |

## Testing
`cargo test` for every pure crate (effects, packet packing, profile validation, FFT bands, colour averaging) with a `HidSink` mock; API tests with `axum` test client; `vitest` for UI logic; `playwright` E2E against `rogd` with a fake device; hardware smoke script on G614JU only.

## Open items for Lebi
- Name check: crate/service names `rogd` / `rogctl` OK?
- Is G614JU the only machine we can test on, or can friends run `rog-probe` for other models?
