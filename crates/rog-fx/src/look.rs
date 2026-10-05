use std::collections::BTreeMap;

use rog_hid::Rgb;
use serde::{Deserialize, Serialize};

use crate::effects::Effect;

pub const MAX_BRIGHTNESS: u8 = 100;
pub const MAX_NAME_LEN: usize = 40;

/// Everything that decides what the lights show, apart from live overlays such as Claude sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Look {
    pub effect: Effect,
    pub brightness: u8,
    /// Fixed colours by LED name (`key1`..`key4`, `bar1`..`bar6`); they replace the effect on those LEDs.
    #[serde(default)]
    pub paint: BTreeMap<String, Rgb>,
}

/// A saved look.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub favourite: bool,
    pub look: Look,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_is_optional_when_reading_a_look() {
        let look: Look =
            serde_json::from_str(r##"{"effect":{"effect":"static","colour":"#ffffff"},"brightness":40}"##).unwrap();

        assert!(look.paint.is_empty());
    }

    #[test]
    fn a_preset_round_trips_with_paint() {
        let mut paint = BTreeMap::new();
        paint.insert("key1".to_owned(), Rgb::new(255, 0, 0));
        let preset = Preset {
            id: "p1".into(),
            name: "Red left".into(),
            favourite: true,
            look: Look { effect: Effect::Static { colour: Rgb::new(255, 255, 255) }, brightness: 50, paint },
        };

        let json = serde_json::to_string(&preset).unwrap();

        assert_eq!(serde_json::from_str::<Preset>(&json).unwrap(), preset);
    }
}
