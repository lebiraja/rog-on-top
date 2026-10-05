# HANDOVER

## What we're building
**rog-on-top**: universal RGB control for ASUS ROG laptops (keyboard, lightbar, logo/lid where present) via a new Rust daemon `rogd`, a `rogctl` CLI, and a local web UI on `http://127.0.0.1:7777`. Feature parity with Armoury Crate / Aura Sync, including Adaptive (screen colour) and Music (audio reactive).

## The journey / why
Started from `/home/lebi/Projects/responce-claude` (claude-glow): a Rust Aura HID driver + daemon that drives the G614JU lightbar from Claude Code hooks. Lebi wants that hardware work generalised. Reused: the `aura-hid` crate, verified G614JU protocol, udev/systemd files. Also copied asusd's `aura_support.ron` (140 models) as the capability-DB seed. Dead ends already known (G614JU): per-key packets light the whole keyboard, `asusctl --zone lightbar-*` is rejected, commit packets after frames snap back to the saved mode.

## Current state
- DONE: repo initialised, `reference/` populated, plan written and committed (commit 3fb21c6, plus this handover's move of the plan).
- NOT STARTED: any code. No Cargo workspace yet.
- Machine: ROG Strix G614JU, Arch/CachyOS, Wayland, `0b05:19b6` on `/dev/hidraw0`, asusd installed, cargo 1.98.

## Locked decisions (don't revisit unless Lebi asks)
- Hardware: ROG laptops only in v1 (no USB peripherals).
- New Rust daemon `rogd` (not Python, not asusd D-Bus wrapper); own HID driver.
- v1 features: core effects + colour picker, advanced effects, per-key editor (per-key models only), automation, Adaptive screen, Music.
- Native systemd service (overrides the Docker-everywhere default).
- UI on :7777, bound to 127.0.0.1, Origin-checked; dark/brutalist, no card grids.

## Plan location
`docs/plans/rog-on-top.md` (architecture, crate layout, effect catalogue, API draft, phases 0–8, risks).

## Exact next steps
1. Ask Lebi (one line) to confirm names `rogd` / `rogctl`; he hasn't answered yet.
2. Phase 0: create Cargo workspace with `crates/{rog-hid,rog-fx,rogd,rogctl}`; port `reference/claude-glow/crates/aura-hid/src/*` into `rog-hid` (rename, keep tests), move effects into `rog-fx`; copy `profiles/g614ju.toml` to `data/profiles/`.
3. Run `cargo test` until green, then Phase 1 (render loop, compositor, `rogctl effect`).
4. Before Phase 1 hardware tests: `systemctl --user stop claude-glow` and check asusd isn't streaming (both write `/dev/hidraw0`).
5. Docs per CLAUDE.md: create `docs/architecture.md`, `docker.md` (note native decision), `API.md`, `FIXES.md`; append STATUS.md after each milestone.

## Cold-start context
Read `docs/plans/rog-on-top.md`, then `docs/protocol-g614ju.md` and `reference/claude-glow/` (the working code to port). Protocol essentials: send `5d bc` once, then stream 64-byte frames with header `5d bc 01 01 04` every ~50 ms; keyboard zone RGB at offsets 9/12/15/18, lightbar LEDs at 27..42 step 3; firmware reverts to saved mode when frames stop; restore with the `5d b3/b5/b4` packets; never send `0x5a` reports. Follow Lebi's CLAUDE.md: reply starts with "Lebi,", tests via cargo, smallest diffs, STATUS.md prepended newest-first.
