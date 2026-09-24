//! Plasma Global Menu (Step 3): serves `com.canonical.dbusmenu` at
//! `/org/songstress/Menu` on the session bus as `com.yossi.songstress`,
//! backed by the Rust-owned menu model (`crate::menu`). KWin learns about it
//! via the `org_kde_kwin_appmenu` Wayland protocol (`wayland_appmenu.rs`).
//!
//! Item ids map deterministically to i32s so GetLayout never needs state:
//! root = 0, top-level menu i = i+1, item j of menu i = (i+1)*100 + j.
//! Activation funnels through `menu::activate` — playback natively in Rust,
//! everything else re-emitted to the frontend as `menu-action`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use zbus::object_server::{InterfaceRef, SignalEmitter};
use zbus::zvariant::{OwnedValue, Value};

use crate::menu::{self, MenuItem};
use crate::profile;

pub const OBJECT_PATH: &str = "/org/songstress/Menu";

static REVISION: AtomicU32 = AtomicU32::new(1);
static MENU_REF: OnceLock<InterfaceRef<Dbusmenu>> = OnceLock::new();
/// Millis timestamp of the last client query — the wayland retry loop uses it
/// to detect "a menu consumer successfully found and imported us" and stop
/// re-registering (each re-register flaps the panel widget).
static LAST_QUERY_MS: AtomicU64 = AtomicU64::new(0);

/// Per-subtree revision the consumer last fetched (GetLayout) — lets
/// AboutToShow answer "stale?" exactly like Qt's exporter does. KDE's
/// DBusMenuImporter only re-fetches a submenu on open when AboutToShow
/// returns true (LayoutUpdated refreshes top-level actions only), so a
/// constant false freezes item enabled/checked state at first import.
static LAST_SEEN: OnceLock<Mutex<HashMap<i32, u32>>> = OnceLock::new();

fn seen_map() -> &'static Mutex<HashMap<i32, u32>> {
    LAST_SEEN.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn last_query_ms() -> u64 {
    LAST_QUERY_MS.load(Ordering::SeqCst)
}

fn touch_query() {
    LAST_QUERY_MS.store(now_millis(), Ordering::SeqCst);
}

/// (id, properties, children) — the dbusmenu layout node. Recursive, so it
/// must NOT implement zvariant's Type (the signature would be infinite);
/// the wire types below break the recursion with variants, exactly like
/// Qt's own dbusmenu exporter.
#[derive(serde::Serialize, Clone, Debug)]
pub struct LayoutNode {
    pub id: i32,
    pub props: HashMap<String, OwnedValue>,
    pub children: Vec<LayoutNode>,
}

/// Wire form of a layout node: struct (i32, a{sv}, av) where each child is
/// variant-wrapped — byte-for-byte what Qt's exporter emits and KDE's
/// DBusMenuImporter demarshals (beginArray + QDBusVariant per element).
#[derive(serde::Serialize, zbus::zvariant::Type)]
pub struct LayoutWire {
    pub id: i32,
    pub props: HashMap<String, OwnedValue>,
    pub children: Vec<ChildWire>,
}

/// A child node on the wire: the variant wrapper is the recursion breaker.
#[derive(serde::Serialize, zbus::zvariant::Type)]
pub struct ChildWire(Value<'static>);

impl LayoutNode {
    fn leaf(id: i32, props: HashMap<String, OwnedValue>) -> Self {
        Self {
            id,
            props,
            children: Vec::new(),
        }
    }
}

/// The child's struct value: (i32, a{sv}, av) with each grandchild
/// variant-wrapped, so the dynamic signature terminates at "(ia{sv}av)".
fn struct_value(node: LayoutNode) -> Value<'static> {
    let children: Vec<Value<'static>> = node
        .children
        .into_iter()
        .map(|c| Value::Value(Box::new(struct_value(c))))
        .collect();
    let structure = zbus::zvariant::StructureBuilder::new()
        .add_field(node.id)
        .append_field(Value::from(node.props))
        .append_field(Value::from(children))
        .build()
        .expect("layout structure");
    Value::Structure(structure)
}

fn child_wire(node: LayoutNode) -> ChildWire {
    ChildWire(struct_value(node))
}

fn to_wire(node: LayoutNode) -> LayoutWire {
    LayoutWire {
        id: node.id,
        props: node.props,
        children: node.children.into_iter().map(child_wire).collect(),
    }
}

fn owned<V: Into<Value<'static>>>(v: V) -> OwnedValue {
    OwnedValue::try_from(v.into()).unwrap_or(0u8.into())
}

/// Deterministic i32 for a menu item (menu index, item index); None for root.
fn item_id(menu_idx: usize, item_idx: usize) -> i32 {
    ((menu_idx + 1) * 100 + item_idx) as i32
}

fn item_props(item: &MenuItem) -> HashMap<String, OwnedValue> {
    let mut props = HashMap::new();
    if item.separator {
        // dbusmenu's section break: type=separator, no label. Kept
        // visible but disabled so Event() ignores anything a consumer
        // might send it (find_item → enabled==false → early return).
        props.insert("type".into(), owned("separator".to_string()));
        props.insert("enabled".into(), owned(false));
        props.insert("visible".into(), owned(true));
        return props;
    }
    props.insert("label".into(), owned(item.label.clone()));
    if let Some(icon) = &item.icon {
        // dbusmenu's spec key is "icon" — but Plasma 6.7's Global Menu
        // applet imports "icon-name"/"icon-data" (verified in its own
        // binary: _dbusmenu_icon_name + QIcon::fromTheme). Send both;
        // spec-strict consumers and KDE each find what they look for.
        props.insert("icon".into(), owned(icon.clone()));
        props.insert("icon-name".into(), owned(icon.clone()));
    }
    props.insert("enabled".into(), owned(item.enabled));
    props.insert("visible".into(), owned(true));
    props.insert("type".into(), owned("standard".to_string()));
    if let Some(checked) = item.checked {
        // "radio" for the theme trio (mutually exclusive modes, like the
        // Appearance pane's segmented control); "checkmark" elsewhere.
        // KDE's importer treats an ungrouped radio as a checkable item, so
        // this degrades to a checkmark rather than breaking.
        props.insert(
            "toggle-type".into(),
            owned(if item.radio { "radio" } else { "checkmark" }.to_string()),
        );
        props.insert("toggle-state".into(), owned(if checked { 1 } else { 0 }));
    }
    props
}

fn menu_props() -> HashMap<String, OwnedValue> {
    let mut props = HashMap::new();
    props.insert("children-display".into(), owned("submenu".to_string()));
    props
}

/// The whole tree, freshly built from the live model on every call.
fn layout() -> LayoutNode {
    let menus = menu::build();
    let children = menus
        .iter()
        .enumerate()
        .map(|(mi, m)| LayoutNode {
            id: (mi + 1) as i32,
            props: {
                let mut p = menu_props();
                p.insert("label".into(), owned(m.label.clone()));
                p
            },
            children: m
                .items
                .iter()
                .enumerate()
                .map(|(ii, item)| LayoutNode::leaf(item_id(mi, ii), item_props(item)))
                .collect(),
        })
        .collect();
    LayoutNode {
        id: 0,
        props: menu_props(),
        children,
    }
}

/// Reverse lookup: i32 → model item (clone), for Event/GetProperty.
fn find_item(id: i32) -> Option<MenuItem> {
    let menus = menu::build();
    for (mi, m) in menus.iter().enumerate() {
        for (ii, item) in m.items.iter().enumerate() {
            if item_id(mi, ii) == id {
                return Some(item.clone());
            }
        }
    }
    None
}

fn subtree(id: i32) -> Option<LayoutNode> {
    if id == 0 {
        return Some(layout());
    }
    layout().children.into_iter().find(|n| n.id == id)
}

pub struct Dbusmenu {
    engine: Arc<crate::mpv::Mpv>,
    app: tauri::AppHandle,
}

#[zbus::interface(name = "com.canonical.dbusmenu")]
impl Dbusmenu {
    #[zbus(property)]
    fn version(&self) -> u32 {
        3
    }

    #[zbus(property)]
    fn text_direction(&self) -> String {
        "ltr".into()
    }

    #[zbus(property)]
    fn status(&self) -> String {
        "normal".into()
    }

    // "icon-theme-path" and "item-icons-are-real-size" omitted — no icons.

    async fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        _property_names: Vec<String>,
    ) -> (u32, LayoutWire) {
        touch_query();
        // dbusmenu depth semantics: 0 = item only, N = item + N levels of
        // children, negative = unlimited. Qt's DBusMenuImporter fetches with
        // depth 1 and expects the first level of children to be present —
        // treating 1 as "no children" makes the import come back empty and
        // the Global Menu widget declares us a "naughty app" and hides.
        let node = match subtree(parent_id) {
            Some(node) if recursion_depth != 0 => {
                fn prune(node: LayoutNode, depth: i32) -> LayoutNode {
                    if depth == 0 {
                        return LayoutNode {
                            id: node.id,
                            props: node.props,
                            children: Vec::new(),
                        };
                    }
                    LayoutNode {
                        id: node.id,
                        props: node.props,
                        children: node
                            .children
                            .into_iter()
                            .map(|c| prune(c, depth - 1))
                            .collect(),
                    }
                }
                prune(node, recursion_depth)
            }
            Some(_) | None => LayoutNode {
                id: parent_id,
                props: HashMap::new(),
                children: Vec::new(),
            },
        };
        let revision = REVISION.load(Ordering::SeqCst);
        seen_map().lock().unwrap().insert(parent_id, revision);
        (revision, to_wire(node))
    }

    async fn get_group_properties(
        &self,
        ids: Vec<i32>,
        _property_names: Vec<String>,
    ) -> Vec<(i32, HashMap<String, OwnedValue>)> {
        let mut out = Vec::new();
        for id in ids {
            if id == 0 {
                out.push((0, menu_props()));
            } else if let Some(item) = find_item(id) {
                out.push((id, item_props(&item)));
            }
        }
        out
    }

    async fn about_to_show(&self, id: i32) -> bool {
        touch_query();
        // Stale-subtree semantics (Qt exporter behavior): true when the
        // consumer hasn't fetched this id since the last model change —
        // KDE's importer then re-fetches the submenu on open.
        let revision = REVISION.load(Ordering::SeqCst);
        let mut seen = seen_map().lock().unwrap();
        if seen.get(&id) == Some(&revision) {
            false
        } else {
            seen.insert(id, revision);
            true
        }
    }

    async fn get_property(&self, id: i32, name: String) -> OwnedValue {
        if id == 0 {
            if name == "children-display" {
                return owned("submenu".to_string());
            }
            return owned(false);
        }
        match find_item(id) {
            Some(item) => item_props(&item).get(&name).cloned().unwrap_or(owned(false)),
            None => owned(false),
        }
    }

    async fn event(&self, id: i32, event_id: String, _data: Value<'_>, _timestamp: u32) {
        if event_id != "clicked" {
            return;
        }
        let Some(item) = find_item(id) else { return };
        if !item.enabled {
            return;
        }
        if !menu::activate(&item.id, &self.engine).await {
            use tauri::Emitter;
            let _ = self.app.emit("menu-action", &item.id);
        }
    }

    #[zbus(signal)]
    pub async fn layout_updated(
        emitter: &SignalEmitter<'_>,
        revision: u32,
        parent: i32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn items_properties_updated(
        emitter: &SignalEmitter<'_>,
        changed: Vec<(i32, HashMap<String, OwnedValue>)>,
        removed: Vec<(i32, Vec<String>)>,
    ) -> zbus::Result<()>;
}

/// Called by `set_menu_state` after every model change: bump the revision and
/// tell the panel to re-fetch the layout.
pub fn notify_changed() {
    let Some(iref) = MENU_REF.get() else { return };
    let revision = REVISION.fetch_add(1, Ordering::SeqCst) + 1;
    let emitter = iref.signal_emitter().clone();
    tauri::async_runtime::spawn(async move {
        let _ = Dbusmenu::layout_updated(&emitter, revision, 0).await;
    });
}

/// Serve the dbusmenu interface, then register it with KWin over Wayland.
/// Failures are logged and swallowed: the Global Menu is best-effort and the
/// titlebar menu bar always works.
pub fn serve(engine: Arc<crate::mpv::Mpv>, app: tauri::AppHandle) {
    let service = profile::appmenu_service_name(&app.config().identifier);
    tauri::async_runtime::spawn(async move {
        match serve_inner(engine, app).await {
            Ok(()) => eprintln!("[appmenu] serving {service}{OBJECT_PATH}"),
            Err(e) => eprintln!("[appmenu] unavailable: {e}"),
        }
    });
}

async fn serve_inner(engine: Arc<crate::mpv::Mpv>, app: tauri::AppHandle) -> zbus::Result<()> {
    let service = profile::appmenu_service_name(&app.config().identifier);
    let conn = zbus::connection::Builder::session()?
        .name(service)?
        .serve_at(
            OBJECT_PATH,
            Dbusmenu {
                engine,
                app: app.clone(),
            },
        )?
        .build()
        .await?;

    let iref = conn
        .object_server()
        .interface::<_, Dbusmenu>(OBJECT_PATH)
        .await?;
    let _ = MENU_REF.set(iref.clone());

    // The bus object exists now — KWin may be told about it.
    crate::wayland_appmenu::register(&app, service, OBJECT_PATH);
    Ok(())
}
