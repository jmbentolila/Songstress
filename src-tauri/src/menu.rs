//! Rust-owned menu model (Step 3): the source of truth for the in-titlebar
//! menu bar and the Plasma Global Menu (dbusmenu). The frontend fetches it
//! via `get_menu`, keeps dynamic bits fresh via `set_menu_state`, and
//! activates items via `menu_activate` — playback items are handled here
//! natively, everything else is re-emitted to the frontend as `menu-action`.

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    /// Some(bool) = checkable; None = plain item.
    pub checked: Option<bool>,
}

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Menu {
    pub id: String,
    pub label: String,
    pub items: Vec<MenuItem>,
}

/// Dynamic bits the frontend owns; pushed up whenever they change so both
/// renderers (titlebar + dbusmenu) re-render from one recomputed model.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MenuState {
    /// Some(true) = playing, Some(false) = paused, None = stopped.
    pub playing: Option<bool>,
    pub has_track: bool,
    pub scanning: bool,
    pub any_staged: bool,
    pub theme_dark: bool,
    /// Shuffle stage: off/album/artist/all (Step 5a).
    #[serde(default)]
    pub shuffle: String,
    /// Repeat stage: off/album/track (Step 5a).
    #[serde(default)]
    pub repeat: String,
    /// Equalizer on/off + preset display name (Step 6); "Custom" when the
    /// gains diverged from every preset.
    #[serde(default)]
    pub eq_enabled: bool,
    #[serde(default = "default_eq_preset")]
    pub eq_preset: String,
}

fn default_eq_preset() -> String {
    "Flat".into()
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            playing: None,
            has_track: false,
            scanning: false,
            any_staged: false,
            theme_dark: true,
            shuffle: "off".into(),
            repeat: "off".into(),
            eq_enabled: false,
            eq_preset: default_eq_preset(),
        }
    }
}

static STATE: Mutex<Option<MenuState>> = Mutex::new(None);

pub fn set_state(state: MenuState) {
    *STATE.lock().unwrap() = Some(state);
}

pub fn state() -> MenuState {
    STATE.lock().unwrap().clone().unwrap_or_default()
}

pub fn build() -> Vec<Menu> {
    let s = state();
    let item = |id: &str, label: &str, enabled: bool, checked: Option<bool>| MenuItem {
        id: id.to_string(),
        label: label.to_string(),
        enabled,
        checked,
    };
    vec![
        Menu {
            id: "playback".into(),
            label: "Playback".into(),
            items: vec![
                item(
                    "playback.play-pause",
                    if s.playing == Some(true) { "Pause" } else { "Play" },
                    s.has_track,
                    None,
                ),
                item("playback.stop", "Stop", s.has_track, None),
                item("playback.previous", "Previous", s.has_track, None),
                item("playback.next", "Next", s.has_track, None),
                item("playback.album-prev", "Previous album", s.has_track, None),
                item("playback.album-next", "Next album", s.has_track, None),
                // Cycling stage items (click advances to the next stage);
                // checkmark = stage active. Frontend owns the cycling.
                item(
                    "playback.shuffle",
                    &format!(
                        "Shuffle: {}",
                        match s.shuffle.as_str() {
                            "album" => "Album",
                            "artist" => "Artist",
                            "all" => "All Artists",
                            _ => "Off",
                        }
                    ),
                    true,
                    Some(s.shuffle != "off"),
                ),
                item(
                    "playback.repeat",
                    &format!(
                        "Repeat: {}",
                        match s.repeat.as_str() {
                            "album" => "Album",
                            "track" => "Track",
                            _ => "Off",
                        }
                    ),
                    true,
                    Some(s.repeat != "off"),
                ),
                // Equalizer (Step 6): enable toggle, cycling preset picker,
                // and an item that opens the playbar popover (frontend).
                item(
                    "playback.eq",
                    &format!("Equalizer: {}", if s.eq_enabled { "On" } else { "Off" }),
                    true,
                    Some(s.eq_enabled),
                ),
                item(
                    "playback.eq-preset",
                    &format!("EQ Preset: {}", s.eq_preset),
                    s.eq_enabled,
                    None,
                ),
                item("playback.eq-customize", "Customize Equalizer…", true, None),
            ],
        },
        Menu {
            id: "library".into(),
            label: "Library".into(),
            items: vec![
                item(
                    "library.rescan",
                    if s.scanning { "Scanning…" } else { "Scan for changes" },
                    !s.scanning,
                    None,
                ),
                item("library.rescan-full", "Re-read all files", !s.scanning, None),
                item("library.add-files", "Import music files…", !s.scanning, None),
                item("library.add-folder", "Import music folder…", !s.scanning, None),
                item(
                    "library.save-imports",
                    "Save imported music",
                    !s.scanning && s.any_staged,
                    None,
                ),
                item("library.choose-folder", "Music folders…", true, None),
            ],
        },
        Menu {
            id: "view".into(),
            label: "View".into(),
            items: vec![item(
                "view.theme",
                "Dark theme",
                true,
                Some(s.theme_dark),
            )],
        },
        Menu {
            id: "help".into(),
            label: "Help".into(),
            items: vec![item("help.about", "About Songstress", true, None)],
        },
    ]
}

/// Playback ids are handled natively here; everything else goes to the
/// frontend as `menu-action`. Returns true when handled natively.
pub async fn activate(id: &str, engine: &crate::mpv::Mpv) -> bool {
    match id {
        "playback.play-pause" => {
            let paused = {
                let st = engine.state.lock().unwrap();
                if st.current().is_none() {
                    return true; // nothing loaded — nothing to do
                }
                st.paused
            };
            let _ = engine.set_paused(!paused).await;
            true
        }
        "playback.stop" => {
            let _ = engine.stop().await;
            true
        }
        "playback.previous" => {
            let _ = engine.jump(-1).await;
            true
        }
        "playback.next" => {
            let _ = engine.jump(1).await;
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_pause_label_follows_state() {
        set_state(MenuState::default());
        let menus = build();
        let playback = menus.iter().find(|m| m.id == "playback").unwrap();
        assert_eq!(playback.items[0].label, "Play");
        assert!(!playback.items[0].enabled);

        set_state(MenuState {
            playing: Some(true),
            has_track: true,
            ..MenuState::default()
        });
        let playback = build().into_iter().find(|m| m.id == "playback").unwrap();
        assert_eq!(playback.items[0].label, "Pause");
        assert!(playback.items[0].enabled);
        assert!(playback.items[1].enabled); // Stop
    }

    #[test]
    fn library_items_gate_on_scan_and_staging() {
        set_state(MenuState {
            scanning: true,
            any_staged: true,
            ..MenuState::default()
        });
        let library = build().into_iter().find(|m| m.id == "library").unwrap();
        assert_eq!(library.items[0].label, "Scanning…");
        assert!(!library.items[0].enabled);
        assert!(!library.items[1].enabled, "full rescan disabled while scanning");
        assert!(!library.items[4].enabled); // save-imports disabled while scanning

        set_state(MenuState {
            scanning: false,
            any_staged: true,
            ..MenuState::default()
        });
        let library = build().into_iter().find(|m| m.id == "library").unwrap();
        assert!(library.items[4].enabled);

        set_state(MenuState::default());
        let library = build().into_iter().find(|m| m.id == "library").unwrap();
        assert!(!library.items[4].enabled); // nothing staged
    }

    #[test]
    fn theme_item_is_checkable() {
        set_state(MenuState {
            theme_dark: false,
            ..MenuState::default()
        });
        let view = build().into_iter().find(|m| m.id == "view").unwrap();
        assert_eq!(view.items[0].checked, Some(false));
    }

    #[test]
    fn eq_items_follow_state() {
        set_state(MenuState::default());
        let playback = build().into_iter().find(|m| m.id == "playback").unwrap();
        // Last three items = the EQ group.
        let n = playback.items.len();
        assert!(n >= 3);
        let eq = &playback.items[n - 3];
        let preset = &playback.items[n - 2];
        let customize = &playback.items[n - 1];
        assert_eq!(eq.label, "Equalizer: Off");
        assert_eq!(eq.checked, Some(false));
        assert_eq!(preset.label, "EQ Preset: Flat");
        assert!(!preset.enabled, "preset picker gated on enabled");
        assert_eq!(customize.label, "Customize Equalizer…");
        assert!(customize.enabled, "customize always reachable");

        set_state(MenuState {
            eq_enabled: true,
            eq_preset: "Rock".into(),
            ..MenuState::default()
        });
        let playback = build().into_iter().find(|m| m.id == "playback").unwrap();
        let eq = &playback.items[playback.items.len() - 3];
        let preset = &playback.items[playback.items.len() - 2];
        assert_eq!(eq.label, "Equalizer: On");
        assert_eq!(eq.checked, Some(true));
        assert_eq!(preset.label, "EQ Preset: Rock");
        assert!(preset.enabled);
    }
}
