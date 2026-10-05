use std::time::Duration;

use rog_fx::Scenes;
use rog_hid::Rgb;

use crate::sessions::State;

/// Brightness of the two LEDs on either side of the seam between two sessions.
const SEAM: f32 = 0.55;

/// Target colour for every bar LED. Each live session owns a contiguous segment of the bar;
/// with no sessions the whole bar shows the `none` scene.
///
/// With exactly two sessions the halves are mirror images (their effects move symmetrically about the
/// centre) and the LEDs beside the seam are dimmed, so two sessions in the same state still read as two.
pub fn bar_targets(scenes: &Scenes, slots: &[State], leds: usize, t: Duration) -> Vec<Rgb> {
    let names: Vec<&str> = if slots.is_empty() { vec!["none"] } else { slots.iter().map(|s| s.scene()).collect() };
    let pair = names.len() == 2;
    let (base, extra) = (leds / names.len(), leds % names.len());
    let mut out = Vec::with_capacity(leds);
    for (k, name) in names.iter().enumerate() {
        let len = base + usize::from(k < extra);
        for j in 0..len {
            let mut pos = if len > 1 { j as f32 / (len - 1) as f32 } else { 0.5 };
            if pair && k == 1 {
                pos = 1.0 - pos;
            }
            let seam = pair && ((k == 0 && j == len - 1) || (k == 1 && j == 0));
            let colour = scenes.get(name).map_or(Rgb::BLACK, |e| e.colour_at(t, pos));
            out.push(if seam { colour.scale(SEAM) } else { colour });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn static_scenes() -> Scenes {
        Scenes::from_toml(
            "[none]\neffect = \"static\"\ncolour = \"#ffffff\"\n\
             [idle]\neffect = \"static\"\ncolour = \"#0000ff\"\n\
             [done]\neffect = \"static\"\ncolour = \"#00ff00\"\n\
             [ask]\neffect = \"static\"\ncolour = \"#ff0000\"\n",
        )
        .expect("test scenes are valid")
    }

    fn orange(t: &[Rgb]) -> usize {
        t.iter().filter(|c| c.r > 200).count()
    }

    #[test]
    fn no_sessions_fills_bar_with_none_scene() {
        let targets = bar_targets(&Scenes::bundled(), &[], 6, Duration::ZERO);

        assert_eq!(targets, vec![Rgb::new(255, 255, 255); 6]);
    }

    #[test]
    fn two_sessions_split_the_bar_in_halves() {
        let targets = bar_targets(&static_scenes(), &[State::Idle, State::Done], 6, Duration::ZERO);

        let blue = Rgb::new(0, 0, 255);
        let green = Rgb::new(0, 255, 0);
        assert_eq!(targets, vec![blue, blue, blue.scale(SEAM), green.scale(SEAM), green, green]);
    }

    #[test]
    fn two_sessions_in_the_same_state_still_show_a_seam() {
        let targets = bar_targets(&static_scenes(), &[State::Idle, State::Idle], 6, Duration::ZERO);

        assert!(targets[2].b < targets[1].b);
        assert_eq!(targets[2], targets[3]);
    }

    #[test]
    fn two_working_sessions_animate_as_mirror_images() {
        let scenes = Scenes::bundled();

        for ms in [0, 300, 700, 1100] {
            let t = bar_targets(&scenes, &[State::Working, State::Working], 6, Duration::from_millis(ms));

            assert_eq!(t[0], t[5], "at {ms} ms");
            assert_eq!(t[1], t[4], "at {ms} ms");
            assert_eq!(t[2], t[3], "at {ms} ms");
        }
    }

    #[test]
    fn a_bar_with_no_leds_renders_nothing() {
        assert!(bar_targets(&static_scenes(), &[State::Idle, State::Ask], 0, Duration::ZERO).is_empty());
    }

    #[test]
    fn single_session_has_no_seam() {
        let targets = bar_targets(&static_scenes(), &[State::Idle], 6, Duration::ZERO);

        assert_eq!(targets, vec![Rgb::new(0, 0, 255); 6]);
    }

    #[test]
    fn four_sessions_share_six_leds_two_two_one_one() {
        let slots = [State::Idle, State::Done, State::Ask, State::Idle];

        let targets = bar_targets(&static_scenes(), &slots, 6, Duration::ZERO);

        assert_eq!(targets[0], targets[1]);
        assert_eq!(targets[2], targets[3]);
        assert_ne!(targets[1], targets[2]);
        assert_eq!(targets.len(), 6);
    }

    #[test]
    fn six_sessions_get_one_led_each() {
        let slots = [State::Idle, State::Done, State::Idle, State::Done, State::Idle, State::Done];

        let targets = bar_targets(&static_scenes(), &slots, 6, Duration::ZERO);

        assert_eq!(targets[0], Rgb::new(0, 0, 255));
        assert_eq!(targets[1], Rgb::new(0, 255, 0));
        assert_eq!(targets[5], Rgb::new(0, 255, 0));
    }

    #[test]
    fn missing_scene_renders_dark_instead_of_panicking() {
        let targets = bar_targets(&Scenes::from_toml("").unwrap(), &[State::Ask], 6, Duration::ZERO);

        assert_eq!(targets, vec![Rgb::BLACK; 6]);
    }

    #[test]
    fn working_comet_lights_the_head_end_first_and_moves() {
        let scenes = Scenes::bundled();

        let start = bar_targets(&scenes, &[State::Working], 6, Duration::ZERO);
        let middle = bar_targets(&scenes, &[State::Working], 6, Duration::from_millis(900));

        assert_eq!(orange(&start), 1);
        assert!(start[0].r > start[5].r);
        assert!(middle[5].r > middle[0].r);
    }
}
