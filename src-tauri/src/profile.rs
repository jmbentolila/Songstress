//! Dev/prod instance separation.
//!
//! `tauri dev` runs with the `tauri.dev.conf.json` overlay, whose identifier
//! is `com.yossi.songstress.dev` (the RPM keeps `com.yossi.songstress`).
//! Tauri derives the data/config/cache dirs from the identifier, so the
//! library DB, window size, thumbnails and import staging isolate with zero
//! code. What is NOT derived — the mpv socket dir under `$XDG_RUNTIME_DIR`,
//! the MPRIS bus name/identity, and the Global Menu service name — is
//! namespaced here, from the one predicate. Everything takes the runtime
//! identifier (`app.config().identifier`); there is deliberately no second
//! env-var mechanism to keep in sync.

/// Identifier the RPM ships (and plain `tauri dev` without the overlay).
pub const PROD_ID: &str = "com.yossi.songstress";
/// Identifier the dev overlay uses.
pub const DEV_ID: &str = "com.yossi.songstress.dev";

/// Anything that is not the prod identifier runs as an isolated instance.
/// (Deliberately not `== DEV_ID`: an unknown future flavor must isolate,
/// never collide with the library you listen to.)
pub(crate) fn is_dev(identifier: &str) -> bool {
    identifier != PROD_ID
}

/// `$XDG_RUNTIME_DIR/<this>/mpv.sock` (+ the mpv pidfile beside it).
pub(crate) fn socket_dir_name(identifier: &str) -> &'static str {
    if is_dev(identifier) {
        "songstress-dev"
    } else {
        "songstress"
    }
}

/// MPRIS bus name: prod keeps the bare name so existing clients and applets
/// keep working; dev claims a sibling so both can hold one at once.
pub(crate) fn mpris_bus_name(identifier: &str) -> &'static str {
    if is_dev(identifier) {
        "org.mpris.MediaPlayer2.songstress.dev"
    } else {
        "org.mpris.MediaPlayer2.songstress"
    }
}

/// What MPRIS controllers (playerctl, media applets) display.
pub(crate) fn mpris_identity(identifier: &str) -> &'static str {
    if is_dev(identifier) {
        "Songstress (dev)"
    } else {
        "Songstress"
    }
}

/// Basename (no `.desktop` suffix) of the desktop file MPRIS clients use for
/// the icon lookup. Matches the file installed for each instance: the RPM's
/// `com.yossi.songstress.desktop`, the dev's user-local
/// `com.yossi.songstress.dev.desktop` (mono icon). Prod keeps its historical
/// value — still deferred, see the note on `desktop_entry`.
pub(crate) fn mpris_desktop_entry(identifier: &str) -> &'static str {
    if is_dev(identifier) {
        "com.yossi.songstress.dev"
    } else {
        "songstress"
    }
}

/// Global Menu service name over D-Bus (the KWin Wayland registration reuses
/// it, so it must differ too or dev steals prod's menu slot on Plasma).
///
/// Dev does NOT reuse the identifier here: with `enableGTKAppId` the GTK
/// application itself owns `com.yossi.songstress.dev` on the bus (tao
/// registers it), so claiming it a second time for the menu would fail and
/// silently kill dev's Global Menu. The `.menu` sibling is arbitrary — KWin
/// is handed (service, path) explicitly — it just has to be unique.
pub(crate) fn appmenu_service_name(identifier: &str) -> &'static str {
    if is_dev(identifier) {
        "com.yossi.songstress.dev.menu"
    } else {
        PROD_ID
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prod_keeps_legacy_names() {
        assert!(!is_dev(PROD_ID));
        assert_eq!(socket_dir_name(PROD_ID), "songstress");
        assert_eq!(mpris_bus_name(PROD_ID), "org.mpris.MediaPlayer2.songstress");
        assert_eq!(mpris_identity(PROD_ID), "Songstress");
        assert_eq!(mpris_desktop_entry(PROD_ID), "songstress");
        assert_eq!(appmenu_service_name(PROD_ID), PROD_ID);
    }

    #[test]
    fn dev_namespaces_everything() {
        assert!(is_dev(DEV_ID));
        assert_eq!(socket_dir_name(DEV_ID), "songstress-dev");
        assert_eq!(
            mpris_bus_name(DEV_ID),
            "org.mpris.MediaPlayer2.songstress.dev"
        );
        assert_eq!(mpris_identity(DEV_ID), "Songstress (dev)");
        assert_eq!(mpris_desktop_entry(DEV_ID), DEV_ID);
        assert_eq!(appmenu_service_name(DEV_ID), "com.yossi.songstress.dev.menu");
    }

    #[test]
    fn unknown_flavor_isolates() {
        assert!(is_dev("com.example.something-else"));
        assert_eq!(socket_dir_name("com.example.something-else"), "songstress-dev");
    }
}
