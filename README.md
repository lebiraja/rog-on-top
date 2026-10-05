# rog-on-top

Local RGB control for ASUS ROG laptops: a Rust daemon (`rogd`), a CLI (`rogctl`) and a web UI on `http://127.0.0.1:7777`.

- Full G16 keyboard layout, colour wheel, gradients, white temperature, palettes
- Effects: static, breathe, pulse, rainbow, comet; brightness; per-zone paint
- Presets with favourites and a boot default; last look survives restarts
- Claude Code overlay: open sessions drive the lightbar (working, asking, done, idle)

Verified on one machine only: **ROG Strix G614JU** (`0b05:19b6`, 4-zone keyboard + 6-LED lightbar). Other models are untested.

## Build and install
Requires Rust and Node 20+.

```sh
(cd web && npm ci && npm run build)   # the UI is embedded into rogd
cargo build --release
install -Dm755 target/release/rogd target/release/rogctl -t ~/.local/bin
install -Dm644 packaging/rogd.service ~/.config/systemd/user/rogd.service
sudo install -Dm644 packaging/99-rog-on-top.rules /etc/udev/rules.d/ && sudo udevadm control --reload && sudo udevadm trigger
systemctl --user enable --now rogd
```

asusd also drives the Aura device; stop its aura control or the LEDs will fight.

## Use
`rogctl effect rainbow`, `rogctl brightness 40`, or open http://127.0.0.1:7777.
Claude Code hooks: point them at `rogctl hook <start|working|ask|done|end>`.

Docs: [architecture](docs/architecture.md), [API](docs/API.md), [protocol notes](docs/protocol-g614ju.md), [plan](docs/plans/rog-on-top.md).

## License
Apache-2.0, see [LICENSE](LICENSE) and [NOTICE](NOTICE).
