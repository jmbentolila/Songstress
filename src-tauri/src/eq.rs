//! Equalizer (PLAN.md Step 6): a 10-band graphic EQ + preamp, applied as an
//! mpv lavfi filter chain on the `af` property (`volume` + chained peaking
//! `equalizer` filters). Setting `af` live is gapless-safe and composes with
//! ReplayGain. Pure builder fns live here; mpv.rs owns the property push.

use serde::{Deserialize, Serialize};

/// Center frequency of each band (Hz), index = slider position left→right.
pub const BANDS: [f64; 10] =
    [31.0, 62.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0];

pub const MAX_GAIN_DB: f64 = 12.0;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Eq {
    pub enabled: bool,
    #[serde(default)]
    pub preamp_db: f64,
    pub gains: [f64; 10],
    /// UI hint only (which preset the gains came from, null = Custom);
    /// Rust never branches on it.
    #[serde(default)]
    pub preset: Option<String>,
}

impl Default for Eq {
    fn default() -> Self {
        Self {
            enabled: false,
            preamp_db: 0.0,
            gains: [0.0; 10],
            preset: Some("Flat".into()),
        }
    }
}

/// Clamp a dB value into the ±12 dB slider range.
pub fn clamp_db(v: f64) -> f64 {
    v.clamp(-MAX_GAIN_DB, MAX_GAIN_DB)
}

/// Below a tenth of a dB the ear hears nothing — treat as flat so tiny
/// float residue from slider math never keeps a dead filter alive.
fn significant(v: f64) -> bool {
    v.abs() >= 0.1
}

pub fn is_flat(eq: &Eq) -> bool {
    !significant(eq.preamp_db) && eq.gains.iter().all(|g| !significant(*g))
}

/// The lavfi chain to set on mpv's `af`, or None when disabled or flat
/// (clearing `af` entirely = zero per-sample cost, exactly the old sound).
/// Only non-flat bands get a filter; octave-width peaking (`width_type=o`
/// `w=1`) per PLAN.md.
pub fn af_chain(eq: &Eq) -> Option<String> {
    if !eq.enabled || is_flat(eq) {
        return None;
    }
    let mut parts: Vec<String> = Vec::new();
    let preamp = clamp_db(eq.preamp_db);
    if significant(preamp) {
        parts.push(format!("volume={preamp:.1}dB"));
    }
    for (band, gain) in BANDS.iter().zip(eq.gains.iter()) {
        let g = clamp_db(*gain);
        if significant(g) {
            parts.push(format!("equalizer=f={band}:width_type=o:w=1:g={g:.1}"));
        }
    }
    if parts.is_empty() {
        return None;
    }
    Some(format!("lavfi=[{}]", parts.join(",")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eq(enabled: bool, preamp: f64, gains: [f64; 10]) -> Eq {
        Eq { enabled, preamp_db: preamp, gains, preset: None }
    }

    #[test]
    fn disabled_or_flat_clears_af() {
        assert_eq!(af_chain(&eq(false, 3.0, [5.0; 10])), None, "off = no chain");
        assert_eq!(af_chain(&eq(true, 0.0, [0.0; 10])), None, "flat = no chain");
        // Residue under the significance threshold still counts as flat.
        let mut e = eq(true, 0.0, [0.0; 10]);
        e.gains[3] = 0.05;
        e.preamp_db = -0.09;
        assert_eq!(af_chain(&e), None);
    }

    #[test]
    fn band_mapping_is_frequency_ordered_peaking_filters() {
        let mut gains = [0.0; 10];
        gains[0] = 4.0; // 31 Hz
        gains[9] = -6.5; // 16 kHz
        let out = af_chain(&eq(true, 0.0, gains)).expect("chain");
        assert_eq!(
            out,
            "lavfi=[equalizer=f=31:width_type=o:w=1:g=4.0,equalizer=f=16000:width_type=o:w=1:g=-6.5]"
                .to_string()
        );
    }

    #[test]
    fn preamp_leads_the_chain_only_when_significant() {
        let mut gains = [0.0; 10];
        gains[2] = 2.0;
        let with = af_chain(&eq(true, -3.0, gains)).expect("chain");
        assert!(with.starts_with("lavfi=[volume=-3.0dB,"), "got {with}");
        let without = af_chain(&eq(true, 0.0, gains)).expect("chain");
        assert!(without.starts_with("lavfi=[equalizer="), "got {without}");
    }

    #[test]
    fn everything_clamps_to_plus_minus_twelve() {
        let mut e = eq(true, 99.0, [99.0; 10]);
        e.gains[0] = -99.0;
        let out = af_chain(&e).expect("chain");
        assert!(out.contains("volume=12.0dB"), "got {out}");
        assert!(out.contains("g=-12.0"), "got {out}");
        assert!(out.contains("g=12.0"), "got {out}");
        assert_eq!(clamp_db(15.0), 12.0);
        assert_eq!(clamp_db(-15.0), -12.0);
        assert_eq!(clamp_db(3.3), 3.3);
    }

    #[test]
    fn enabled_preamp_only_still_builds_a_chain() {
        let out = af_chain(&eq(true, -6.0, [0.0; 10])).expect("chain");
        assert_eq!(out, "lavfi=[volume=-6.0dB]");
    }

    #[test]
    fn serde_roundtrips_camel_case_with_defaults() {
        let json = r#"{"enabled":true,"preampDb":2,"gains":[1,2,3,4,5,6,7,8,9,10]}"#;
        let parsed: Eq = serde_json::from_str(json).expect("parse");
        assert_eq!(parsed.preset, None, "preset defaults when absent");
        assert_eq!(parsed.gains[9], 10.0);
        let back = serde_json::to_string(&parsed).expect("serialize");
        assert!(back.contains("preampDb"), "camelCase on the wire too");
        assert!(!back.contains("preamp_db"));
    }
}
