use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rog_fx::look::MAX_NAME_LEN;
use rog_fx::{Command, Effect, Look, Preset, Response, Scenes};
use rog_fx::command::MAX_BRIGHTNESS;
use rog_hid::{Profile, Rgb};

use crate::claude::bar_targets;
use crate::sessions::Sessions;

/// What the lights should show right now. Clients mutate it through `apply`; the render loop reads it
/// through `targets`. Open Claude sessions overlay the lightbar group on top of the look.
pub struct Engine {
    look: Look,
    presets: Vec<Preset>,
    default_preset: Option<String>,
    dirty: bool,
    sessions: Sessions,
    scenes: Scenes,
}

const OVERLAY_GROUP: &str = "bar";

pub fn default_look() -> Look {
    Look { effect: Effect::Rainbow { period_ms: 6000, spread: 1.0 }, brightness: MAX_BRIGHTNESS, paint: BTreeMap::new() }
}

fn valid_name(name: &str) -> Result<String, Response> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_LEN {
        return Err(Response::Error { message: format!("preset name must be 1 to {MAX_NAME_LEN} characters") });
    }
    Ok(name.to_owned())
}

fn not_found(id: &str) -> Response {
    Response::Error { message: format!("no preset {id:?}") }
}

impl Engine {
    pub fn new(effect: Effect) -> Self {
        Self::restore(Look { effect, ..default_look() }, Vec::new(), None)
    }

    /// Start from saved data. A default preset, when it still exists, wins over the last look.
    pub fn restore(last: Look, presets: Vec<Preset>, default_preset: Option<String>) -> Self {
        let default_preset = default_preset.filter(|id| presets.iter().any(|p| &p.id == id));
        let look = default_preset
            .as_ref()
            .and_then(|id| presets.iter().find(|p| &p.id == id))
            .map_or(last, |p| p.look.clone());
        Self { look, presets, default_preset, dirty: false, sessions: Sessions::default(), scenes: Scenes::bundled() }
    }

    pub fn look(&self) -> &Look {
        &self.look
    }

    pub fn presets(&self) -> &[Preset] {
        &self.presets
    }

    pub fn default_preset(&self) -> Option<&str> {
        self.default_preset.as_deref()
    }

    /// True once after any change that should be written to disk.
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    pub fn apply(&mut self, cmd: Command, now: Instant, at: SystemTime) -> Response {
        match cmd {
            Command::SetEffect { effect } => self.look.effect = effect,
            Command::SetBrightness { level } if level <= MAX_BRIGHTNESS => self.look.brightness = level,
            Command::SetBrightness { level } => {
                return Response::Error { message: format!("brightness {level} is above {MAX_BRIGHTNESS}") };
            }
            Command::SetPaint { paint } => self.look.paint = paint,
            Command::Claude(msg) => {
                self.sessions.apply(&msg, now, at);
                return Response::Ok;
            }
            Command::State => {
                return Response::State {
                    effect: self.look.effect.clone(),
                    brightness: self.look.brightness,
                    paint: self.look.paint.clone(),
                };
            }
            Command::ListPresets => {
                return Response::Presets { presets: self.presets.clone(), default: self.default_preset.clone() };
            }
            Command::SavePreset { name } => return self.save_preset(&name, at),
            Command::ApplyPreset { id } => match self.presets.iter().find(|p| p.id == id) {
                Some(p) => self.look = p.look.clone(),
                None => return not_found(&id),
            },
            Command::DeletePreset { id } => {
                let before = self.presets.len();
                self.presets.retain(|p| p.id != id);
                if self.presets.len() == before {
                    return not_found(&id);
                }
                if self.default_preset.as_deref() == Some(id.as_str()) {
                    self.default_preset = None;
                }
            }
            Command::UpdatePreset { id, name, favourite } => {
                let name = match name.as_deref().map(valid_name).transpose() {
                    Ok(name) => name,
                    Err(e) => return e,
                };
                let Some(p) = self.presets.iter_mut().find(|p| p.id == id) else { return not_found(&id) };
                if let Some(name) = name {
                    p.name = name;
                }
                if let Some(favourite) = favourite {
                    p.favourite = favourite;
                }
            }
            Command::SetDefaultPreset { id } => {
                if let Some(id) = &id {
                    if !self.presets.iter().any(|p| &p.id == id) {
                        return not_found(id);
                    }
                }
                self.default_preset = id;
            }
        }
        self.dirty = true;
        Response::Ok
    }

    fn save_preset(&mut self, name: &str, at: SystemTime) -> Response {
        let name = match valid_name(name) {
            Ok(name) => name,
            Err(e) => return e,
        };
        let mut stamp = at.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
        while self.presets.iter().any(|p| p.id == format!("p{stamp}")) {
            stamp += 1;
        }
        let preset = Preset { id: format!("p{stamp}"), name, favourite: false, look: self.look.clone() };
        self.presets.push(preset.clone());
        self.dirty = true;
        Response::Preset { preset }
    }

    /// Drop dead or interrupted Claude sessions.
    pub fn sweep(&mut self, now: Instant, at: SystemTime, alive: impl Fn(u32) -> bool, interrupted: impl Fn(&Path, SystemTime) -> bool) {
        self.sessions.sweep(now, at, alive, interrupted);
    }

    /// Target colour per LED, in profile order. Each LED group is laid out as its own strip, 0.0 to 1.0.
    /// Paint replaces the effect on named LEDs; open Claude sessions then replace the lightbar.
    pub fn targets(&self, profile: &Profile, t: Duration, now: Instant) -> Vec<Rgb> {
        let mut out = vec![Rgb::BLACK; profile.leds.len()];
        let level = f32::from(self.look.brightness) / f32::from(MAX_BRIGHTNESS);
        for group in groups(profile) {
            let members: Vec<usize> = profile.group_indices(group).collect();
            for (j, &led) in members.iter().enumerate() {
                let pos = if members.len() > 1 { j as f32 / (members.len() - 1) as f32 } else { 0.5 };
                out[led] = self.look.effect.colour_at(t, pos).scale(level);
            }
        }
        for (name, colour) in &self.look.paint {
            if let Some(led) = profile.led_index(name) {
                out[led] = colour.scale(level);
            }
        }
        let bar: Vec<usize> = profile.group_indices(OVERLAY_GROUP).collect();
        let slots = self.sessions.slots(now, bar.len());
        if !slots.is_empty() {
            for (led, colour) in bar.iter().zip(bar_targets(&self.scenes, &slots, bar.len(), t)) {
                out[*led] = colour.scale(level);
            }
        }
        out
    }
}

fn groups(profile: &Profile) -> Vec<&str> {
    let mut seen: Vec<&str> = Vec::new();
    for led in &profile.leds {
        if !seen.contains(&led.group.as_str()) {
            seen.push(&led.group);
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use super::*;
    use rog_fx::{ClaudeMessage, Event};

    const RED: Rgb = Rgb::new(255, 0, 0);

    fn apply(engine: &mut Engine, cmd: Command) -> Response {
        engine.apply(cmd, Instant::now(), SystemTime::now())
    }

    fn solid(colour: Rgb) -> Engine {
        Engine::new(Effect::Static { colour })
    }

    #[test]
    fn static_effect_fills_every_led() {
        let p = Profile::g614ju();

        let targets = solid(RED).targets(&p, Duration::ZERO, Instant::now());

        assert_eq!(targets, vec![RED; 10]);
    }

    #[test]
    fn brightness_scales_the_output() {
        let p = Profile::g614ju();
        let mut engine = solid(RED);

        apply(&mut engine, Command::SetBrightness { level: 50 });

        assert_eq!(engine.targets(&p, Duration::ZERO, Instant::now()), vec![RED.scale(0.5); 10]);
    }

    #[test]
    fn brightness_above_max_is_rejected_and_ignored() {
        let mut engine = solid(RED);

        let reply = apply(&mut engine, Command::SetBrightness { level: 101 });

        assert!(matches!(reply, Response::Error { .. }));
        assert!(matches!(apply(&mut engine, Command::State), Response::State { brightness: 100, .. }));
    }

    #[test]
    fn each_group_spans_its_own_strip() {
        let p = Profile::g614ju();
        let engine = Engine::new(Effect::Rainbow { period_ms: 3000, spread: 0.5 });

        let t = engine.targets(&p, Duration::ZERO, Instant::now());

        assert_eq!(t[p.led_index("key1").unwrap()], t[p.led_index("bar1").unwrap()]);
        assert_ne!(t[p.led_index("key1").unwrap()], t[p.led_index("key4").unwrap()]);
        assert_eq!(t[p.led_index("key4").unwrap()], t[p.led_index("bar6").unwrap()]);
    }

    #[test]
    fn set_effect_replaces_the_effect_and_state_reports_it() {
        let mut engine = solid(RED);
        let blue = Effect::Static { colour: Rgb::new(0, 0, 255) };

        let reply = apply(&mut engine, Command::SetEffect { effect: blue.clone() });

        assert_eq!(reply, Response::Ok);
        assert_eq!(apply(&mut engine, Command::State), Response::State { effect: blue, brightness: 100, paint: BTreeMap::new() });
    }

    fn claude(event: Event) -> Command {
        Command::Claude(ClaudeMessage { event, session: "s".into(), pid: None, transcript: None })
    }

    #[test]
    fn an_open_session_overrides_only_the_lightbar() {
        let p = Profile::g614ju();
        let mut engine = solid(RED);

        apply(&mut engine, claude(Event::Done));
        let t = engine.targets(&p, Duration::ZERO, Instant::now());

        assert!(p.group_indices("keys").all(|i| t[i] == RED));
        assert!(p.group_indices("bar").all(|i| t[i] == Rgb::new(0, 255, 0)));
    }

    #[test]
    fn ending_the_last_session_returns_the_bar_to_the_base_effect() {
        let p = Profile::g614ju();
        let mut engine = solid(RED);
        apply(&mut engine, claude(Event::Working));

        apply(&mut engine, claude(Event::End));

        assert_eq!(engine.targets(&p, Duration::ZERO, Instant::now()), vec![RED; 10]);
    }

    #[test]
    fn brightness_also_dims_the_claude_overlay() {
        let p = Profile::g614ju();
        let mut engine = solid(RED);
        apply(&mut engine, claude(Event::Done));
        apply(&mut engine, Command::SetBrightness { level: 50 });

        let t = engine.targets(&p, Duration::ZERO, Instant::now());

        assert_eq!(t[p.led_index("bar1").unwrap()], Rgb::new(0, 255, 0).scale(0.5));
    }

    fn save(engine: &mut Engine, name: &str) -> Preset {
        match apply(engine, Command::SavePreset { name: name.into() }) {
            Response::Preset { preset } => preset,
            other => panic!("expected a preset, got {other:?}"),
        }
    }

    fn paint(entries: &[(&str, Rgb)]) -> BTreeMap<String, Rgb> {
        entries.iter().map(|(n, c)| ((*n).to_owned(), *c)).collect()
    }

    #[test]
    fn paint_replaces_the_effect_on_named_leds_only() {
        let p = Profile::g614ju();
        let blue = Rgb::new(0, 0, 255);
        let mut engine = solid(RED);

        apply(&mut engine, Command::SetPaint { paint: paint(&[("key2", blue), ("bar5", blue)]) });
        let t = engine.targets(&p, Duration::ZERO, Instant::now());

        assert_eq!(t[p.led_index("key2").unwrap()], blue);
        assert_eq!(t[p.led_index("bar5").unwrap()], blue);
        assert_eq!(t[p.led_index("key1").unwrap()], RED);
    }

    #[test]
    fn unknown_paint_names_are_ignored_and_brightness_dims_paint() {
        let p = Profile::g614ju();
        let mut engine = solid(RED);
        apply(&mut engine, Command::SetPaint { paint: paint(&[("key9", RED), ("key1", RED)]) });
        apply(&mut engine, Command::SetBrightness { level: 50 });

        let t = engine.targets(&p, Duration::ZERO, Instant::now());

        assert_eq!(t[p.led_index("key1").unwrap()], RED.scale(0.5));
    }

    #[test]
    fn a_saved_preset_restores_the_look_it_captured() {
        let blue = Rgb::new(0, 0, 255);
        let mut engine = solid(RED);
        apply(&mut engine, Command::SetPaint { paint: paint(&[("key1", blue)]) });
        apply(&mut engine, Command::SetBrightness { level: 30 });
        let preset = save(&mut engine, "  Night  ");
        apply(&mut engine, Command::SetEffect { effect: Effect::Static { colour: blue } });
        apply(&mut engine, Command::SetPaint { paint: BTreeMap::new() });
        apply(&mut engine, Command::SetBrightness { level: 100 });

        let reply = apply(&mut engine, Command::ApplyPreset { id: preset.id.clone() });

        assert_eq!(reply, Response::Ok);
        assert_eq!(preset.name, "Night");
        assert_eq!(engine.look(), &preset.look);
        assert_eq!(engine.look().brightness, 30);
    }

    #[test]
    fn preset_ids_stay_unique_when_saved_in_the_same_millisecond() {
        let mut engine = solid(RED);

        let a = save(&mut engine, "A");
        let b = save(&mut engine, "B");

        assert_ne!(a.id, b.id);
    }

    #[test]
    fn bad_preset_names_and_unknown_ids_are_errors() {
        let mut engine = solid(RED);

        assert!(matches!(apply(&mut engine, Command::SavePreset { name: "   ".into() }), Response::Error { .. }));
        assert!(matches!(apply(&mut engine, Command::SavePreset { name: "x".repeat(41) }), Response::Error { .. }));
        assert!(matches!(apply(&mut engine, Command::ApplyPreset { id: "nope".into() }), Response::Error { .. }));
        assert!(matches!(apply(&mut engine, Command::DeletePreset { id: "nope".into() }), Response::Error { .. }));
        assert!(matches!(apply(&mut engine, Command::SetDefaultPreset { id: Some("nope".into()) }), Response::Error { .. }));
    }

    #[test]
    fn update_renames_and_favourites_and_delete_clears_the_default() {
        let mut engine = solid(RED);
        let preset = save(&mut engine, "A");
        apply(&mut engine, Command::SetDefaultPreset { id: Some(preset.id.clone()) });

        apply(&mut engine, Command::UpdatePreset { id: preset.id.clone(), name: Some("Renamed".into()), favourite: Some(true) });
        let Response::Presets { presets, default } = apply(&mut engine, Command::ListPresets) else { panic!("presets") };
        assert_eq!((presets[0].name.as_str(), presets[0].favourite), ("Renamed", true));
        assert_eq!(default.as_deref(), Some(preset.id.as_str()));

        apply(&mut engine, Command::DeletePreset { id: preset.id });
        assert_eq!(engine.default_preset(), None);
        assert!(engine.presets().is_empty());
    }

    #[test]
    fn restore_prefers_a_surviving_default_preset_over_the_last_look() {
        let mut source = solid(RED);
        let preset = save(&mut source, "Home");
        let last = Look { effect: Effect::Static { colour: Rgb::new(0, 255, 0) }, ..default_look() };

        let with_default = Engine::restore(last.clone(), vec![preset.clone()], Some(preset.id.clone()));
        let stale_default = Engine::restore(last.clone(), vec![preset], Some("gone".into()));

        assert_eq!(with_default.look().effect, Effect::Static { colour: RED });
        assert_eq!(stale_default.look(), &last);
        assert_eq!(stale_default.default_preset(), None);
    }

    #[test]
    fn changes_mark_the_engine_dirty_once_and_reads_do_not() {
        let mut engine = solid(RED);
        apply(&mut engine, Command::State);
        assert!(!engine.take_dirty());

        apply(&mut engine, Command::SetBrightness { level: 10 });

        assert!(engine.take_dirty());
        assert!(!engine.take_dirty());
    }
}
