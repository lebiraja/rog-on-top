## [2026-10-05 10:15] — Web UI on :7777
**What:** rogd/src/http.rs (axum: /api/v1 state, effect, brightness, frame; Origin+Host guard; rust-embed SPA) and web/ (Vite+React+TS+Tailwind+Zod UI: live preview, effect list, colour, cycle, spread, brightness, Dim white). Installed and verified in browser; 49 Rust + 4 vitest tests green.
**Why:** Phase 3 of the plan; Lebi asked for a UI to control it.
**State:** DONE
**Next:** Persist last effect/brightness across rogd restarts (currently resets to rainbow); shadcn components and WS preview deferred.

## [2026-10-05 10:00] — Claude overlay absorbed into rogd; claude-glow retired
**What:** Ported sessions/procfs/bar scenes from claude-glow into rogd. Open Claude sessions override the lightbar group; keyboard keeps the rogd effect; no sessions falls back to the base effect. `rogctl hook <event>` replaces `claude-glow hook`; 7 hooks in ~/.claude/settings.json repointed (backup: settings.json.bak-pre-rogd). Installed rogd/rogctl to ~/.local/bin, user unit enabled, claude-glow service disabled. docs/ created. 69 tests green.
**Why:** Two daemons wrote hidraw0 at once; Lebi wants Claude state to always win on the bar, seamlessly.
**State:** DONE
**Next:** Visual check of working/ask/done on the bar; asusd still running (aura contention warning).

## [2026-10-05 02:00] — Phase 1 rogd core built
**What:** rog-fx: Rainbow effect + Command/Response wire types; rog-hid: Rgb::from_hsv + hex Serialize; rogd: Engine (effect + brightness -> per-LED targets), 50 ms render loop with smoothing/reconnect/restore-on-exit, JSON-line unix socket ($XDG_RUNTIME_DIR/rogd.sock), asusd warning; rogctl: effect/brightness/state; packaging/ udev rule + user unit. 33 tests green. Smoke run: rogd connected to the real device ~1s, rogctl commands round-tripped, clean exit removed the socket.
**Why:** Phase 1 of docs/plans/rog-on-top.md.
**State:** IN PROGRESS
**Next:** Visual hardware check of `rogctl effect rainbow` (stop claude-glow; asusd is running), then docs/ (architecture.md, API.md, docker.md, FIXES.md).

## [2026-10-05 01:00] — Phase 0 workspace scaffolded
**What:** Cargo workspace with crates rog-hid (colour/device/frame/profile), rog-fx (effects/scene), stub rogd/rogctl; data/profiles/g614ju.toml and data/scenes_default.toml; cargo test green (all ported tests pass).
**Why:** Phase 0 of docs/plans/rog-on-top.md; names rogd/rogctl assumed (Lebi hadn't confirmed).
**State:** DONE
**Next:** Phase 1: render loop, compositor, `rogctl effect` (stop claude-glow service and check asusd first).

## [2026-10-05 00:30] — Handover written
**What:** HANDOVER.md written; plan moved to docs/plans/rog-on-top.md.
**Why:** Lebi continues in a fresh session from this directory.
**State:** DONE
**Next:** Confirm names rogd/rogctl, then Phase 0 scaffold.

## [2026-10-05 00:00] — Project scaffolded and planned
**What:** Created repo, copied claude-glow `aura-hid` crate, G614JU profile and protocol notes, udev/systemd files and asusd's `aura_support.ron` (140 models) into `reference/`; wrote docs/plans/rog-on-top.md.
**Why:** Lebi wants a universal ROG RGB controller (Rust daemon `rogd`, web UI on :7777, Adaptive + Music + per-key + automation).
**State:** DECIDED
**Next:** Lebi approves the plan, then Phase 0 (workspace scaffold, port aura-hid to rog-hid).
