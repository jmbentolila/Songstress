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
    /// Checkable renders as a radio dot rather than a checkmark (the theme
    /// trio mirrors the Appearance pane's segmented control — three mutually
    /// exclusive states, not three independent toggles).
    #[serde(default)]
    pub radio: bool,
    /// Section break. The sidebar panes group their rows (Import/Scan/Storage,
    /// Sizes/Theme); the Global Menu says the same structure with separators.
    /// Separators are never enabled, never carry a label that renders.
    #[serde(default)]
    pub separator: bool,
    /// freedesktop icon name, rendered by the dbusmenu consumer (KDE's panel
    /// resolves it against the Plasma icon theme — the Breeze media-* set).
    /// The in-window sidebar menu ignores this; its rows are text by house
    /// style. Optional: only the transport/mode rows carry glyphs.
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Menu {
    pub id: String,
    pub label: String,
    pub items: Vec<MenuItem>,
}

impl MenuItem {
    fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }
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
    /// Staged ALBUMS (the sidebar counts them the same way); the manage-imports
    /// row gates on >0 and shows the count. Replaces `anyStaged` 2026-09-03 so
    /// the menu row can say what the sidebar door says.
    #[serde(default)]
    pub staged_count: i32,
    /// The MODE ("light" | "dark" | "system"), not the resolved theme — a
    /// bool flattened the 3-state segmented control into a toggle and made
    /// `system` unreachable from the menu (parked 2026-08-31, this pass).
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub playbar_gradient: bool,
    /// Expanded-panel artwork gradient (Step 9b) — on is the shipped look.
    #[serde(default = "default_on")]
    pub album_gradient: bool,
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

fn default_theme() -> String {
    "system".into()
}

fn default_on() -> bool {
    true
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            playing: None,
            has_track: false,
            scanning: false,
            staged_count: 0,
            theme: default_theme(),
            playbar_gradient: false,
            album_gradient: true,
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
        radio: false,
        separator: false,
        icon: None,
    };
    // Same as `item`, but wearing a glyph in the Global Menu (owner request
    // 2026-09-04: transport rows speak with the desktop's own media icons).
    let radio = |id: &str, label: &str, checked: bool| MenuItem {
        id: id.to_string(),
        label: label.to_string(),
        enabled: true,
        checked: Some(checked),
        radio: true,
        separator: false,
        icon: None,
    };
    let sep = || MenuItem {
        id: String::new(),
        label: String::new(),
        enabled: false,
        checked: None,
        radio: false,
        separator: true,
        icon: None,
    };
    // Glyphs that track state, not just the verb (owner asked for the play
    // CONTROLS: the row already says Play/Pause — the icon must not lie).
    // Checkable rows wear glyphs too (owner's second look 2026-09-04): the
    // appmenu's two-column layout makes an intermittently-empty icon column
    // read as a hole; a continuous stripe reads as rhythm. The checkmark
    // stays the state signal — the glyph names the concept, day-one rule.
    let pp_icon = if s.playing == Some(true) {
        "media-playback-pause"
    } else {
        "media-playback-start"
    };
    let repeat_icon = match s.repeat.as_str() {
        "album" => "media-playlist-repeat",
        "track" => "media-playlist-repeat-song",
        _ => "media-repeat-none",
    };
    let shuffle_icon = if s.shuffle == "off" {
        "media-playlist-no-shuffle"
    } else {
        "media-playlist-shuffle"
    };
    vec![
        Menu {
            id: "playback".into(),
            label: "Playback".into(),
            items: vec![
                // Transport: the Global Menu's unique value here — the
                // sidebar pane dropped these rows because the PlayBar owns
                // them in-window; out-of-window this menu IS the transport.
                item(
                    "playback.play-pause",
                    if s.playing == Some(true) { "Pause" } else { "Play" },
                    s.has_track,
                    None,
                )
                .with_icon(pp_icon),
                item("playback.stop", "Stop", s.has_track, None).with_icon("media-playback-stop"),
                item("playback.previous", "Previous", s.has_track, None).with_icon("media-skip-backward"),
                item("playback.next", "Next", s.has_track, None).with_icon("media-skip-forward"),
                item("playback.album-prev", "Previous album", s.has_track, None).with_icon("go-previous-skip"),
                item("playback.album-next", "Next album", s.has_track, None).with_icon("go-next-skip"),
                sep(),
                // Cycling stage items (click advances to the next stage);
                // checkmark = stage active. Frontend owns the cycling. Order
                // mirrors the sidebar pane (Repeat, Shuffle, Equalizer).
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
                )
                .with_icon(repeat_icon),
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
                )
                .with_icon(shuffle_icon),
                sep(),
                // Equalizer: enable toggle, cycling preset picker, and an
                // item that opens the playbar popover (frontend) — the
                // preamp/band sliders have no menu form and live only in
                // the popover/pane (user decision 2026-09-03).
                item(
                    "playback.eq",
                    &format!("Equalizer: {}", if s.eq_enabled { "On" } else { "Off" }),
                    true,
                    Some(s.eq_enabled),
                )
                .with_icon("view-media-equalizer"),
                item(
                    "playback.eq-preset",
                    &format!("EQ Preset: {}", s.eq_preset),
                    s.eq_enabled,
                    None,
                ),
                item("playback.eq-customize", "Customize Equalizer…", true, None),
                sep(),
                // Footer actions mirror the sidebar panes' footers.
                item("playback.reset", "Reset playback", true, None),
            ],
        },
        Menu {
            id: "library".into(),
            label: "Library".into(),
            items: vec![
                // Sections mirror the sidebar's Library pane: Import | Scan | Storage.
                item("library.add-files", "Import music files…", !s.scanning, None),
                item("library.add-folder", "Import music folder…", !s.scanning, None),
                item(
                    "library.manage-imports",
                    "Manage imported music…",
                    !s.scanning && s.staged_count > 0,
                    None,
                ),
                sep(),
                item(
                    "library.rescan",
                    if s.scanning { "Scanning…" } else { "Scan for changes" },
                    !s.scanning,
                    None,
                ),
                item("library.rescan-full", "Re-read all files", !s.scanning, None),
                sep(),
                item("library.choose-folder", "Music folders…", true, None),
            ],
        },
        Menu {
            // Renamed from "View": the sidebar calls this pane Appearance
            // and the two surfaces say the same words now. The size sliders
            // stay in-window only; "More appearance settings…" walks the
            // user to them (user decision 2026-09-03).
            id: "appearance".into(),
            label: "Appearance".into(),
            items: vec![
                radio("appearance.theme-system", "Theme: System", s.theme == "system"),
                radio("appearance.theme-light", "Theme: Light", s.theme == "light"),
                radio("appearance.theme-dark", "Theme: Dark", s.theme == "dark"),
                sep(),
                item(
                    "appearance.playbar-gradient",
                    "Playbar artwork gradient",
                    true,
                    Some(s.playbar_gradient),
                ),
                item(
                    "appearance.album-gradient",
                    "Album artwork gradient",
                    true,
                    Some(s.album_gradient),
                ),
                item("appearance.accent", "Accent color…", true, None),
                sep(),
                item("appearance.more", "More appearance settings…", true, None),
                item("appearance.reset", "Reset appearance", true, None),
            ],
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

    fn find<'a>(menus: &'a [Menu], menu_id: &str, item_id: &str) -> &'a MenuItem {
        menus
            .iter()
            .find(|m| m.id == menu_id)
            .unwrap_or_else(|| panic!("no menu {menu_id}"))
            .items
            .iter()
            .find(|i| i.id == item_id)
            .unwrap_or_else(|| panic!("no item {item_id}"))
    }

    fn n_seps(menus: &[Menu], menu_id: &str) -> usize {
        menus
            .iter()
            .find(|m| m.id == menu_id)
            .unwrap()
            .items
            .iter()
            .filter(|i| i.separator)
            .count()
    }

    #[test]
    fn play_pause_label_follows_state() {
        set_state(MenuState::default());
        let menus = build();
        let play = find(&menus, "playback", "playback.play-pause");
        assert_eq!(play.label, "Play");
        assert!(!play.enabled);

        set_state(MenuState {
            playing: Some(true),
            has_track: true,
            ..MenuState::default()
        });
        let menus = build();
        assert_eq!(find(&menus, "playback", "playback.play-pause").label, "Pause");
        assert!(find(&menus, "playback", "playback.play-pause").enabled);
        assert!(find(&menus, "playback", "playback.stop").enabled);
    }

    #[test]
    fn library_items_gate_on_scan_and_staging() {
        set_state(MenuState {
            scanning: true,
            staged_count: 2,
            ..MenuState::default()
        });
        let menus = build();
        assert_eq!(find(&menus, "library", "library.rescan").label, "Scanning…");
        assert!(!find(&menus, "library", "library.rescan").enabled);
        assert!(!find(&menus, "library", "library.rescan-full").enabled);
        assert!(!find(&menus, "library", "library.add-files").enabled);
        // Manage-imports follows the sidebar door: gated on the pile AND
        // dead while a scan runs (the sidebar disables it on scanner.running).
        assert!(!find(&menus, "library", "library.manage-imports").enabled);

        set_state(MenuState {
            scanning: false,
            staged_count: 3,
            ..MenuState::default()
        });
        let menus = build();
        let manage = find(&menus, "library", "library.manage-imports");
        assert!(manage.enabled);
        assert_eq!(manage.label, "Manage imported music…", "no count in the label (owner, 2026-09-03)");

        set_state(MenuState::default());
        let menus = build();
        let manage = find(&menus, "library", "library.manage-imports");
        assert!(!manage.enabled, "nothing staged → dead row");
        assert_eq!(manage.label, "Manage imported music…");
    }

    #[test]
    fn theme_is_three_radios_following_the_mode_not_the_resolution() {
        // Default is `system` (the app's default mode) — the old bool could
        // not express this state at all.
        let menus = build();
        assert_eq!(
            find(&menus, "appearance", "appearance.theme-system").checked,
            Some(true)
        );
        assert_eq!(
            find(&menus, "appearance", "appearance.theme-dark").checked,
            Some(false)
        );
        assert!(find(&menus, "appearance", "appearance.theme-system").radio);
        // Exactly one checked in every mode.
        for mode in ["light", "dark", "system"] {
            set_state(MenuState {
                theme: mode.into(),
                ..MenuState::default()
            });
            let menus = build();
            let appearance = menus.iter().find(|m| m.id == "appearance").unwrap();
            let on = appearance
                .items
                .iter()
                .filter(|i| i.radio && i.checked == Some(true))
                .count();
            assert_eq!(on, 1, "radio group reports exactly one mode: {mode}");
        }
    }

    #[test]
    fn sections_mirror_the_sidebar_panes() {
        // Playback: transport | modes | equalizer | reset → 3 breaks.
        // Library: import | scan | storage → 2 breaks.
        // Appearance: theme | playbar+album+accent | more+reset → 2 breaks.
        set_state(MenuState::default());
        let menus = build();
        assert_eq!(n_seps(&menus, "playback"), 3);
        assert_eq!(n_seps(&menus, "library"), 2);
        assert_eq!(n_seps(&menus, "appearance"), 2);
        assert_eq!(n_seps(&menus, "help"), 0);
        // Separators are never activatable, whatever a consumer sends them.
        let menus = build();
        let s = menus
            .iter()
            .find(|m| m.id == "library")
            .unwrap()
            .items
            .iter()
            .find(|i| i.separator)
            .unwrap();
        assert!(!s.enabled);
        assert_eq!(s.checked, None);
    }

    #[test]
    fn playbar_gradient_toggle_follows_state() {
        set_state(MenuState {
            playbar_gradient: true,
            ..MenuState::default()
        });
        let menus = build();
        assert_eq!(
            find(&menus, "appearance", "appearance.playbar-gradient").checked,
            Some(true)
        );
    }

    #[test]
    fn album_gradient_toggle_follows_state() {
        // On is the default (the shipped look); the toggle reports state.
        let menus = build();
        assert_eq!(
            find(&menus, "appearance", "appearance.album-gradient").checked,
            Some(true)
        );
        set_state(MenuState {
            album_gradient: false,
            ..MenuState::default()
        });
        let menus = build();
        assert_eq!(
            find(&menus, "appearance", "appearance.album-gradient").checked,
            Some(false)
        );
    }

    #[test]
    fn eq_items_follow_state() {
        set_state(MenuState::default());
        let menus = build();
        assert_eq!(find(&menus, "playback", "playback.eq").label, "Equalizer: Off");
        assert_eq!(find(&menus, "playback", "playback.eq").checked, Some(false));
        assert_eq!(find(&menus, "playback", "playback.eq-preset").label, "EQ Preset: Flat");
        assert!(!find(&menus, "playback", "playback.eq-preset").enabled, "preset picker gated on enabled");
        assert_eq!(
            find(&menus, "playback", "playback.eq-customize").label,
            "Customize Equalizer…"
        );
        assert!(find(&menus, "playback", "playback.eq-customize").enabled, "customize always reachable");

        set_state(MenuState {
            eq_enabled: true,
            eq_preset: "Rock".into(),
            ..MenuState::default()
        });
        let menus = build();
        assert_eq!(find(&menus, "playback", "playback.eq").label, "Equalizer: On");
        assert_eq!(find(&menus, "playback", "playback.eq").checked, Some(true));
        assert_eq!(find(&menus, "playback", "playback.eq-preset").label, "EQ Preset: Rock");
        assert!(find(&menus, "playback", "playback.eq-preset").enabled);
    }

    #[test]
    fn transport_rows_carry_glyphs_that_track_state() {
        // The Global Menu's transport wears the desktop's own media icons
        // (owner request 2026-09-04) — and they must not lie about what a
        // click does: Play shows ▶, Pause shows ⏸, repeat/shuffle glyphs
        // follow the stage.
        set_state(MenuState::default());
        let menus = build();
        assert_eq!(
            find(&menus, "playback", "playback.play-pause").icon.as_deref(),
            Some("media-playback-start")
        );
        assert_eq!(
            find(&menus, "playback", "playback.stop").icon.as_deref(),
            Some("media-playback-stop")
        );
        assert_eq!(
            find(&menus, "playback", "playback.previous").icon.as_deref(),
            Some("media-skip-backward")
        );
        assert_eq!(
            find(&menus, "playback", "playback.album-next").icon.as_deref(),
            Some("go-next-skip")
        );
        set_state(MenuState {
            playing: Some(true),
            has_track: true,
            ..Default::default()
        });
        let menus = build();
        assert_eq!(
            find(&menus, "playback", "playback.play-pause").icon.as_deref(),
            Some("media-playback-pause")
        );
        // Checkable rows carry glyphs alongside the checkmark (owner's
        // second look 2026-09-04): the icon column is a continuous stripe
        // or it is a hole — the applet's layout leaves no third choice.
        set_state(MenuState {
            repeat: "track".into(),
            shuffle: "all".into(),
            eq_enabled: true,
            ..Default::default()
        });
        let menus = build();
        assert_eq!(
            find(&menus, "playback", "playback.repeat").icon.as_deref(),
            Some("media-playlist-repeat-song")
        );
        assert_eq!(
            find(&menus, "playback", "playback.shuffle").icon.as_deref(),
            Some("media-playlist-shuffle")
        );
        assert_eq!(
            find(&menus, "playback", "playback.eq").icon.as_deref(),
            Some("view-media-equalizer")
        );
    }
}
