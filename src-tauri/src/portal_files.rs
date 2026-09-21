//! Native file/folder pickers via the xdg-desktop-portal `FileChooser`
//! interface — the compositor's OWN dialog (GTK on GNOME, Qt on Plasma),
//! so no toolkit-specific picker binary is ever required.
//!
//! Every picker here is portal-FIRST with a kdialog fallback owned by the
//! caller (`lib.rs`): a missing/broken portal must degrade to the old
//! behavior, never to a dead button. `None` = the user cancelled, which the
//! frontend treats as silence, not an error. Uses the `zbus` already in the
//! tree — same request/signal pattern as `pick_color.rs`.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::StreamExt;
use zbus::proxy;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

#[proxy(
    interface = "org.freedesktop.portal.FileChooser",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait FileChooserPortal {
    /// Opens the dialog; the returned path is the Request to wait on.
    fn open_file(
        &self,
        parent_window: &str,
        title: &str,
        options: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.portal.Request",
    default_service = "org.freedesktop.portal.Desktop",
)]
trait PortalRequest {
    /// (response code, results). 0 = picked (`results["uris"]` is the
    /// chosen `file://` URIs); 1 = cancelled; 2 = failed.
    #[zbus(signal)]
    fn response(
        &self,
        response: u32,
        results: HashMap<String, OwnedValue>,
    ) -> zbus::Result<()>;
}

static NEXT_TOKEN: AtomicU64 = AtomicU64::new(0);

/// `file:///home/u/M%C3%BAsica/a.mp3` → `/home/u/Música/a.mp3`.
/// Non-file URIs and undecodable escapes are `None` (caller skips them).
pub fn uri_to_path(uri: &str) -> Option<String> {
    let rest = uri.strip_prefix("file://")?;
    // Strip a `file://host/` authority when present; the portal only ever
    // hands us local files, so `localhost` or empty both mean this machine.
    let path = rest.find('/').map(|i| &rest[i..]).unwrap_or(rest);
    decode_pct(path)
}

fn decode_pct(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 3 <= bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?;
                out.push(u8::from_str_radix(hex, 16).ok()?);
                i += 3;
            }
            b'%' => return None,
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

fn request_token() -> String {
    format!(
        "songstress_{}_{}",
        std::process::id(),
        NEXT_TOKEN.fetch_add(1, Ordering::Relaxed)
    )
}

async fn open_portal(
    title: &str,
    start: &Path,
    filters: &[(&str, &[&str])],
    multiple: bool,
    directory: bool,
) -> Result<Option<Vec<String>>, String> {
    let conn = zbus::Connection::session()
        .await
        .map_err(|e| format!("could not reach the session bus: {e}"))?;
    let portal = FileChooserPortalProxy::new(&conn)
        .await
        .map_err(|e| format!("file picker unavailable: {e}"))?;
    let mut options: HashMap<&str, Value<'_>> = HashMap::new();
    options.insert("handle_token", Value::from(request_token()));
    options.insert("multiple", Value::from(multiple));
    options.insert("directory", Value::from(directory));
    if !filters.is_empty() {
        let encoded: Vec<(String, Vec<(u32, String)>)> = filters
            .iter()
            .map(|(name, pats)| {
                (
                    name.to_string(),
                    pats.iter().map(|p| (0u32, p.to_string())).collect(),
                )
            })
            .collect();
        options.insert("filters", Value::from(encoded));
    }
    // Seed the dialog location; the portal ignores a missing folder.
    if !start.as_os_str().is_empty() {
        let mut folder = start.to_string_lossy().into_owned().into_bytes();
        folder.push(0);
        options.insert("current_folder", Value::from(folder));
    }
    // No parent window (same reasoning as pick_color): an empty parent is
    // legal and keeps this working on Wayland and X11 alike.
    let request = portal
        .open_file("", title, options)
        .await
        .map_err(|e| format!("file picker failed to open: {e}"))?;
    // The signal cannot precede us here: it fires on the user's choice,
    // whole human seconds after this subscription lands.
    let req = PortalRequestProxy::builder(&conn)
        .path(request.clone())
        .map_err(|e| format!("bad picker request path: {e}"))?
        .build()
        .await
        .map_err(|e| format!("file picker failed: {e}"))?;
    let mut stream = req
        .receive_response()
        .await
        .map_err(|e| format!("file picker failed: {e}"))?;
    let signal = stream
        .next()
        .await
        .ok_or_else(|| "file picker closed without answering".to_string())?;
    let args = signal
        .args()
        .map_err(|e| format!("unreadable picker answer: {e}"))?;
    if args.response != 0 {
        return Ok(None);
    }
    let uris = args
        .results
        .get("uris")
        .ok_or_else(|| "picker answered without any files".to_string())?;
    let uris =
        Vec::<String>::try_from(uris.clone()).map_err(|e| format!("bad picker uris: {e}"))?;
    let paths: Vec<String> = uris.iter().filter_map(|u| uri_to_path(u)).collect();
    Ok(if paths.is_empty() { None } else { Some(paths) })
}

/// Open-file dialog: portal-native, `None` on cancel.
/// `filters` are `(label, globs)` pairs; `multiple` allows multi-select.
pub async fn open_files(
    title: &str,
    start: &Path,
    filters: &[(&str, &[&str])],
    multiple: bool,
) -> Result<Option<Vec<String>>, String> {
    open_portal(title, start, filters, multiple, false).await
}

/// Folder dialog: portal-native, `None` on cancel.
pub async fn open_directory(title: &str, start: &Path) -> Result<Option<String>, String> {
    Ok(open_portal(title, start, &[], false, true)
        .await?
        .and_then(|mut v| v.pop()))
}

#[cfg(test)]
mod tests {
    use super::uri_to_path;

    #[test]
    fn plain_file_uri_round_trips() {
        assert_eq!(
            uri_to_path("file:///home/yossi/Music/a.mp3"),
            Some("/home/yossi/Music/a.mp3".into())
        );
    }

    #[test]
    fn percent_escapes_decode() {
        assert_eq!(
            uri_to_path("file:///home/yossi/M%C3%BAsica/my%20song.flac"),
            Some("/home/yossi/Música/my song.flac".into())
        );
    }

    #[test]
    fn non_file_uris_are_skipped() {
        assert_eq!(uri_to_path("https://example.com/a.mp3"), None);
        assert_eq!(uri_to_path("file:///bad%2"), None);
        assert_eq!(uri_to_path("file:///bad%zz.mp3"), None);
    }
}
