export interface MenuItem {
  label: string;
  action: () => void;
}

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
