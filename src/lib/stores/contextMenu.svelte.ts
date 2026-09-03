export interface MenuItem {
  label: string;
  /** Absent on a separator row. */
  action?: () => void;
  /** Section break, same language as the Rust-owned Global Menu model: the
   * app's menus group by rules (Edit | Playback | Container | Import-state),
   * and a hairline says it without a header row. */
  separator?: boolean;
}

/** Shared section break — plain data, one instance reused everywhere is
 *  fine (the renderer keys each-row by index, never by identity). */
export const SEP: MenuItem = { label: "", separator: true };

/** App-wide context menu state — one menu at a time, positioned at the
 * pointer. Rendered by ContextMenu.svelte (mounted once in App.svelte). */
export const contextMenu = $state({
  open: false,
  x: 0,
  y: 0,
  items: [] as MenuItem[],
});

export function openContextMenu(x: number, y: number, items: MenuItem[]) {
  contextMenu.x = x;
  contextMenu.y = y;
  // Flip side when the pointer is too close to an edge (the component also
  // clamps against its measured size after mount).
  contextMenu.items = items;
  contextMenu.open = true;
}

export function closeContextMenu() {
  contextMenu.open = false;
}
