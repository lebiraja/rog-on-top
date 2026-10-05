# Docker
Not used. `rogd` runs natively as a systemd user service (`packaging/rogd.service`): it needs `/dev/hidraw*` (udev `uaccess`) and, later, the user's PipeWire/portal session. Tests run with `cargo test`.
