// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Program class for the dev instance. The dock groups by WM_CLASS *before*
/// app-id (gnome-shell's tracker checks the class first and treats a hit as
/// canonical), and the class defaults to the binary basename — "songstress"
/// for BOTH builds — so without this the dev window reports the prod class
/// and groups under the prod icon no matter what the app-id says. The dev
/// desktop file declares the matching StartupWMClass. Prod keeps the default
/// (its RPM desktop already claims StartupWMClass=songstress, which is how
/// the main app groups).
const DEV_PRGNAME: &str = "songstress-dev";

fn main() {
    // Read the merged config the `tauri dev` CLI injects (TAURI_CONFIG) —
    // the same object the app itself boots from, so there is no second
    // mechanism to keep in sync. The RPM never sees this var (verified: zero
    // TAURI_ entries in its environ), and missing/unparseable means prod.
    // Must run before any GTK init: the class is snapshotted at first use.
    let dev = std::env::var("TAURI_CONFIG")
        .ok()
        .and_then(|cfg| serde_json::from_str::<serde_json::Value>(&cfg).ok())
        .and_then(|v| {
            v.get("identifier")?
                .as_str()
                .map(|id| id != songstress_lib::profile::PROD_ID)
        })
        .unwrap_or(false);
    if dev {
        glib::set_prgname(Some(DEV_PRGNAME));
    }
    songstress_lib::run()
}
