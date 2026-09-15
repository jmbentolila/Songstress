//! Screen color picker via the xdg-desktop-portal `Screenshot.PickColor`
//! method (Step 9b: the dropper behind the panel-gradient editor).
//!
//! No screenshot files, no GTK dialogs: the portal raises the compositor's
//! own crosshair (KWin on Plasma) and reports the clicked pixel. Cancel (or
//! any portal-side failure code) is a quiet `None` so the UI stays silent;
//! only transport-level failures are errors. Uses the `zbus` already in the
//! tree — no new D-Bus crate for one method call and one signal.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::StreamExt;
use zbus::proxy;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Structure, Value};

#[proxy(
    interface = "org.freedesktop.portal.Screenshot",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait ScreenshotPortal {
    /// Opens the picker; the returned path is the Request to wait on.
    fn pick_color(
        &self,
        parent_window: &str,
        options: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.portal.Request",
    default_service = "org.freedesktop.portal.Desktop",
)]
trait PortalRequest {
    /// (response code, results). 0 = picked (`results["color"]` is
    /// (r, g, b) doubles in 0..=1); 1 = cancelled; 2 = failed.
    #[zbus(signal)]
    fn response(
        &self,
        response: u32,
        results: HashMap<String, OwnedValue>,
    ) -> zbus::Result<()>;
}

static NEXT_TOKEN: AtomicU64 = AtomicU64::new(0);

/// Pick a screen color. `Ok(None)` = the user cancelled (Esc / right-click).
pub async fn pick_color() -> Result<Option<String>, String> {
    let conn = zbus::Connection::session()
        .await
        .map_err(|e| format!("could not reach the session bus: {e}"))?;
    let portal = ScreenshotPortalProxy::new(&conn)
        .await
        .map_err(|e| format!("color picker unavailable: {e}"))?;
    // Request tokens must be unique per call (alphanumeric + underscore).
    let token = format!(
        "songstress_{}_{}",
        std::process::id(),
        NEXT_TOKEN.fetch_add(1, Ordering::Relaxed)
    );
    let mut options = HashMap::new();
    options.insert("handle_token", Value::from(token.as_str()));
    // No parent window: the picker is a fullscreen crosshair, not a dialog
    // transient to us — an empty parent is legal and keeps this working on
    // both Wayland and X11 sessions.
    let request = portal
        .pick_color("", options)
        .await
        .map_err(|e| format!("color picker failed to open: {e}"))?;
    // The signal cannot precede us here: it fires on the user's click,
    // whole human seconds after this subscription lands.
    let req = PortalRequestProxy::builder(&conn)
        .path(request.clone())
        .map_err(|e| format!("bad picker request path: {e}"))?
        .build()
        .await
        .map_err(|e| format!("color picker failed: {e}"))?;
    let mut stream = req
        .receive_response()
        .await
        .map_err(|e| format!("color picker failed: {e}"))?;
    let signal = stream
        .next()
        .await
        .ok_or_else(|| "color picker closed without answering".to_string())?;
    let args = signal
        .args()
        .map_err(|e| format!("unreadable picker answer: {e}"))?;
    if args.response != 0 {
        return Ok(None);
    }
    let color = args
        .results
        .get("color")
        .ok_or_else(|| "picker answered without a color".to_string())?;
    let structure =
        Structure::try_from(color.clone()).map_err(|e| format!("bad picker color: {e}"))?;
    let (r, g, b): (f64, f64, f64) =
        structure.try_into().map_err(|e| format!("bad picker color: {e}"))?;
    let byte = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Ok(Some(format!(
        "{:02x}{:02x}{:02x}",
        byte(r),
        byte(g),
        byte(b)
    )))
}
