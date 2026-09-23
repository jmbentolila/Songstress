//! KWin Global Menu registration (Step 3): binds the `org_kde_kwin_appmenu`
//! Wayland protocol to OUR window's wl_surface and points it at the
//! dbusmenu object served by `menu_dbus.rs`. This is how the Plasma Global
//! Menu widget knows which menu belongs to the focused window.
//!
//! The wl_display/wl_surface come from Tauri's raw-window-handle; they belong
//! to WebKitGTK, so we adopt them in "guest mode"
//! (`Backend::from_foreign_display`) and never touch their lifecycle.

use std::sync::atomic::{AtomicU8, Ordering};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_backend::sys::client::ObjectId;
use wayland_protocols_plasma::appmenu::client::org_kde_kwin_appmenu::OrgKdeKwinAppmenu;
use wayland_protocols_plasma::appmenu::client::org_kde_kwin_appmenu_manager::OrgKdeKwinAppmenuManager;

#[derive(Default)]
struct RegistryState {
    appmenu_manager: Option<OrgKdeKwinAppmenuManager>,
}

impl Dispatch<WlRegistry, ()> for RegistryState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_registry::Event;
        if let Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == OrgKdeKwinAppmenuManager::interface().name {
                let max = OrgKdeKwinAppmenuManager::interface().version;
                let ver = version.min(max);
                state.appmenu_manager =
                    Some(registry.bind(name, ver, qh, ()));
            }
        }
    }
}

/// The appmenu object has no events, but the proxy machinery still wants a
/// (no-op) Dispatch impl.
impl Dispatch<OrgKdeKwinAppmenu, ()> for RegistryState {
    fn event(
        _: &mut Self,
        _: &OrgKdeKwinAppmenu,
        _: <OrgKdeKwinAppmenu as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

/// Same for the manager global we bind from the registry.
impl Dispatch<OrgKdeKwinAppmenuManager, ()> for RegistryState {
    fn event(
        _: &mut Self,
        _: &OrgKdeKwinAppmenuManager,
        _: <OrgKdeKwinAppmenuManager as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

/// 0 = not started, 1 = ok, 2 = failed (for a one-line status log).
static STATE: AtomicU8 = AtomicU8::new(0);

/// 0 = registration still trying, 1 = ok, 2 = failed. The frontend starts
/// assuming the Global Menu (no titlebar flash) and shows the in-titlebar
/// menu bar only on a definitive failure (2).
pub fn state() -> u8 {
    STATE.load(Ordering::SeqCst)
}

/// Spawn the registration on its own thread: it must outlive this call (the
/// adopted objects keep the protocol binding alive) and never block setup.
/// GNOME-derived sessions never implement the manager protocol — skip the
/// broadcast there instead of roundtripping a registry that cannot answer.
pub fn register(app: &tauri::AppHandle, service: &str, object_path: &str) {
    use tauri::Manager;
    if crate::classify_de(&crate::desktop_env()) == "gnome" {
        eprintln!("[appmenu-wayland] GNOME session: no Global Menu, skipping");
        STATE.store(2, Ordering::SeqCst);
        return;
    }
    let Some(win) = app.get_webview_window("main") else {
        eprintln!("[appmenu-wayland] no main window");
        STATE.store(2, Ordering::SeqCst);
        return;
    };
    let raw_display = match win.display_handle() {
        Ok(h) => h.as_raw(),
        Err(e) => {
            eprintln!("[appmenu-wayland] no display handle: {e}");
            STATE.store(2, Ordering::SeqCst);
            return;
        }
    };
    let RawDisplayHandle::Wayland(dh) = raw_display else {
        eprintln!("[appmenu-wayland] not a Wayland display");
        STATE.store(2, Ordering::SeqCst);
        return;
    };
    let display = dh.display.as_ptr() as usize;
    let raw_window = match win.window_handle() {
        Ok(h) => h.as_raw(),
        Err(e) => {
            eprintln!("[appmenu-wayland] no window handle: {e}");
            STATE.store(2, Ordering::SeqCst);
            return;
        }
    };
    let RawWindowHandle::Wayland(h) = raw_window else {
        eprintln!("[appmenu-wayland] not a Wayland window");
        STATE.store(2, Ordering::SeqCst);
        return;
    };
    let surface = h.surface.as_ptr() as usize;
    let service = service.to_string();
    let object_path = object_path.to_string();
    let app_handle = app.clone();

    std::thread::Builder::new()
        .name("appmenu-wayland".into())
        .spawn(move || unsafe {
            match register_foreign(display, surface, &service, &object_path) {
                Ok(()) => {
                    STATE.store(1, Ordering::SeqCst);
                    eprintln!("[appmenu-wayland] registered {service}{object_path}");
                    // Tell the frontend so it can hide the in-titlebar menu
                    // bar (the Global Menu now owns those menus).
                    use tauri::Emitter;
                    let _ = app_handle.emit("appmenu-registered", true);
                    // The connection + appmenu object must stay alive for the
                    // binding to persist — park forever (app exit reaps us).
                    loop {
                        std::thread::park();
                    }
                }
                Err(e) => {
                    STATE.store(2, Ordering::SeqCst);
                    eprintln!("[appmenu-wayland] registration failed: {e}");
                }
            }
        })
        .expect("spawn appmenu-wayland thread");
}

/// # Safety
/// `display` and `surface` must be live `*mut wl_display` / `*mut wl_surface`
/// pointers of the compositor connection our window was created on, and must
/// outlive the returned binding (we never drop them — the thread parks).
unsafe fn register_foreign(
    display: usize,
    surface: usize,
    service: &str,
    object_path: &str,
) -> Result<(), String> {
    use wayland_protocols_plasma::appmenu::client::org_kde_kwin_appmenu::OrgKdeKwinAppmenu;
    use wayland_backend::sys::client::Backend as SysBackend;

    let backend = SysBackend::from_foreign_display(display as *mut _);
    let conn = Connection::from_backend(backend);
    let mut queue = conn.new_event_queue::<RegistryState>();
    let qh = queue.handle();

    let _registry = conn.display().get_registry(&qh, ());
    let mut state = RegistryState::default();
    queue
        .roundtrip(&mut state)
        .map_err(|e| format!("registry roundtrip: {e}"))?;
    let Some(manager) = state.appmenu_manager.clone() else {
        return Err("compositor has no org_kde_kwin_appmenu_manager".into());
    };

    let surface_id = ObjectId::from_ptr(WlSurface::interface(), surface as *mut _)
        .map_err(|e| format!("adopt wl_surface: {e}"))?;
    let surface = WlSurface::from_id(&conn, surface_id)
        .map_err(|e| format!("resolve wl_surface: {e}"))?;

    // KWin links an appmenu object to its window ONLY at create time and only
    // if the window is already MAPPED (findWindow searches mapped windows).
    // At launch we race the first buffer attach, so create → release → retry
    // a few times; the attempt that lands after the map sticks. Qt apps dodge
    // this by registering long after mapping.
    //
    // Each re-register makes KWin emit transient application_menu states
    // (including ("","")) which make the panel widget flap — so STOP as soon
    // as a dbusmenu consumer actually queries us (menu_dbus touches
    // LAST_QUERY_MS on every GetLayout/AboutToShow): that's the proof the
    // current binding is live and imported.
    let mut appmenu: Option<OrgKdeKwinAppmenu> = None;
    for attempt in 0..10 {
        if let Some(old) = appmenu.take() {
            old.release();
        }
        std::thread::sleep(std::time::Duration::from_millis(1200));
        let linked_before = crate::menu_dbus::last_query_ms();
        let created = manager.create(&surface, &qh, ());
        created.set_address(service.to_string(), object_path.to_string());
        queue
            .roundtrip(&mut state)
            .map_err(|e| format!("roundtrip (attempt {attempt}): {e}"))?;
        appmenu = Some(created);
        // Give a consumer a moment to notice the (re)bind and query us.
        std::thread::sleep(std::time::Duration::from_millis(1200));
        if crate::menu_dbus::last_query_ms() > linked_before {
            eprintln!("[appmenu-wayland] linked + imported after attempt {attempt}");
            break;
        }
    }

    // Leak the protocol objects so the binding survives this closure; the
    // parked thread keeps the connection alive anyway.
    std::mem::forget(manager);
    std::mem::forget(surface);
    Ok(())
}
