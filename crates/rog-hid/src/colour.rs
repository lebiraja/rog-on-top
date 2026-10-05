use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb { r: 0, g: 0, b: 0 };

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn scale(self, k: f32) -> Self {
        let f = |c: u8| (c as f32 * k.clamp(0.0, 1.0)).round() as u8;
        Self::new(f(self.r), f(self.g), f(self.b))
    }

    /// Fully saturated-capable colour from hue (any real, wraps at 1.0), saturation and value in 0..=1.
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let (s, v) = (s.clamp(0.0, 1.0), v.clamp(0.0, 1.0));
        let h6 = h.rem_euclid(1.0) * 6.0;
        let f = |n: f32| {
            let k = (n + h6) % 6.0;
            ((v - v * s * k.min(4.0 - k).clamp(0.0, 1.0)) * 255.0).round() as u8
        };
        Self::new(f(5.0), f(3.0), f(1.0))
    }

    pub fn lerp(self, to: Rgb, k: f32) -> Self {
        let k = k.clamp(0.0, 1.0);
        let f = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * k).round() as u8;
        Self::new(f(self.r, to.r), f(self.g, to.g), f(self.b, to.b))
    }
}

impl FromStr for Rgb {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.strip_prefix('#').unwrap_or(s);
        if hex.len() != 6 {
            return Err(format!("colour must be 6 hex digits, got {s:?}"));
        }
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| format!("{s:?}: {e}"));
        Ok(Self::new(byte(0)?, byte(2)?, byte(4)?))
    }
}

impl<'de> Deserialize<'de> for Rgb {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(de::Error::custom)
    }
}

impl Serialize for Rgb {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_with_and_without_hash() {
        assert_eq!("#00ffff".parse(), Ok(Rgb::new(0, 255, 255)));
        assert_eq!("ff00ff".parse(), Ok(Rgb::new(255, 0, 255)));
    }

    #[test]
    fn rejects_bad_length_and_digits() {
        assert!("#fff".parse::<Rgb>().is_err());
        assert!("#zzzzzz".parse::<Rgb>().is_err());
    }

    #[test]
    fn hsv_hits_the_primaries_and_wraps() {
        assert_eq!(Rgb::from_hsv(0.0, 1.0, 1.0), Rgb::new(255, 0, 0));
        assert_eq!(Rgb::from_hsv(1.0 / 3.0, 1.0, 1.0), Rgb::new(0, 255, 0));
        assert_eq!(Rgb::from_hsv(2.0 / 3.0, 1.0, 1.0), Rgb::new(0, 0, 255));
        assert_eq!(Rgb::from_hsv(1.0, 1.0, 1.0), Rgb::new(255, 0, 0));
        assert_eq!(Rgb::from_hsv(0.5, 0.0, 0.5), Rgb::new(128, 128, 128));
    }

    #[test]
    fn serialises_as_hex_and_round_trips() {
        let json = serde_json::to_string(&Rgb::new(0, 255, 16)).unwrap();

        assert_eq!(json, "\"#00ff10\"");
        assert_eq!(serde_json::from_str::<Rgb>(&json).unwrap(), Rgb::new(0, 255, 16));
    }
}
