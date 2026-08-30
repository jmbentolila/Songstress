import type { Album } from "./types";

export type GridRow =
  | { kind: "albums"; items: Album[] }
  | { kind: "expanded"; album: Album };

export function columnCount(width: number, tileSize: number, gap: number): number {
  if (width <= 0 || tileSize <= 0) return 1;
  return Math.max(1, Math.floor((width + gap) / (tileSize + gap)));
}

/**
 * `hostId` = the album whose row the live panel sits in. The expanded row
 * is inserted directly after the row containing the album. An id not
 * present in the list is skipped.
 */
export function buildRows(albums: Album[], columns: number, hostId: string | null): GridRow[] {
  const rows: GridRow[] = [];
  const entries: string[] = [];
  if (hostId && albums.some((a) => a.id === hostId)) entries.push(hostId);
  const cols = Math.max(1, columns);

  for (let i = 0; i < albums.length; i += cols) {
    const items = albums.slice(i, i + cols);
    rows.push({ kind: "albums", items });
    for (const id of entries) {
      const album = items.find((a) => a.id === id);
      if (album) rows.push({ kind: "expanded", album });
    }
  }
  return rows;
}
