import type { Album } from "./types";

export type GridRow =
  | { kind: "albums"; items: Album[] }
  | { kind: "expanded"; album: Album };

export function columnCount(width: number, tileSize: number, gap: number): number {
  if (width <= 0 || tileSize <= 0) return 1;
  return Math.max(1, Math.floor((width + gap) / (tileSize + gap)));
}

export function buildRows(
  albums: Album[],
  columns: number,
  expandedId: string | null,
): GridRow[] {
  const rows: GridRow[] = [];
  const expanded = expandedId ? albums.find((a) => a.id === expandedId) : undefined;
  const cols = Math.max(1, columns);

  for (let i = 0; i < albums.length; i += cols) {
    const items = albums.slice(i, i + cols);
    rows.push({ kind: "albums", items });
    if (expanded && items.some((a) => a.id === expanded.id)) {
      rows.push({ kind: "expanded", album: expanded });
    }
  }
  return rows;
}
