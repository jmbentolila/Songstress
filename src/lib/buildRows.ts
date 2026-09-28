import type { Album } from "./types";

export type GridRow =
  | { kind: "albums"; items: Album[] }
  | { kind: "expanded"; album: Album }
  | { kind: "ghost"; id: string };

export const GRID_GAP = 20;
export const TILE_SIZE_MIN = 120;
export const TILE_SIZE_MAX = 320;

export function columnCount(width: number, tileSize: number, gap: number): number {
  if (width <= 0 || tileSize <= 0) return 1;
  return Math.max(1, Math.floor((width + gap) / (tileSize + gap)));
}

export interface TileStop {
  /** Nominal tile-size value that first produces this layout. */
  tileSize: number;
  columns: number;
  /** Rendered tile width once the columns stretch across `width`. */
  rendered: number;
}

/**
 * Every nominal tile size that actually changes the grid layout: the minimum
 * plus the first value yielding each new column count. Values between stops
 * only rewrite the setting without moving a single tile.
 */
export function tileStops(
  width: number,
  gap: number,
  min = TILE_SIZE_MIN,
  max = TILE_SIZE_MAX,
): TileStop[] {
  if (!(width > 0) || !(gap >= 0) || !(max >= min)) return [];
  const stops: TileStop[] = [];
  for (let tileSize = Math.ceil(min); tileSize <= max; tileSize++) {
    const columns = columnCount(width, tileSize, gap);
    if (stops.length === 0 || stops[stops.length - 1].columns !== columns) {
      stops.push({
        tileSize,
        columns,
        rendered: (width - (columns - 1) * gap) / columns,
      });
    }
  }
  return stops;
}

/** Index of the stop producing the same layout as `tileSize`. Falls back to
 * the nearest nominal stop when the width changed out from under the value. */
export function tileStopIndex(stops: TileStop[], width: number, gap: number, tileSize: number): number {
  if (stops.length === 0) return 0;
  const columns = columnCount(width, tileSize, gap);
  const exact = stops.findIndex((stop) => stop.columns === columns);
  if (exact >= 0) return exact;
  let best = 0;
  let bestDist = Math.abs(stops[0].tileSize - tileSize);
  for (let i = 1; i < stops.length; i++) {
    const dist = Math.abs(stops[i].tileSize - tileSize);
    if (dist < bestDist) {
      best = i;
      bestDist = dist;
    }
  }
  return best;
}

/**
 * `hostId` = the album whose row the live panel sits in. The expanded row
 * is inserted directly after the row containing the album. An id not
 * present in the list is skipped.
 *
 * `ghostIds` = outgoing panels of in-flight cross-row switches. Each gets
 * its own ghost row directly after its album's row — the ghost is where
 * the panel plays its plain close while the host has already flipped to
 * the destination (the two album animations run in parallel; only the
 * view's travel waits for the ghost list to drain). Ghost ids not present
 * in the list are skipped (their row vanished — the grid cleans them up).
 */
export function buildRows(
  albums: Album[],
  columns: number,
  hostId: string | null,
  ghostIds: string[] = [],
): GridRow[] {
  const rows: GridRow[] = [];
  const cols = Math.max(1, columns);
  // Per-album special row, host first (insertion order is render order when
  // both sit in the same row — a same-row ghost can't happen, host and
  // ghost are always different albums on different rows).
  const special = new Map<string, "expanded" | "ghost">();
  if (hostId && albums.some((a) => a.id === hostId)) special.set(hostId, "expanded");
  for (const id of ghostIds) {
    if (albums.some((a) => a.id === id) && !special.has(id)) special.set(id, "ghost");
  }

  for (let i = 0; i < albums.length; i += cols) {
    const items = albums.slice(i, i + cols);
    rows.push({ kind: "albums", items });
    for (const [id, kind] of special) {
      const album = items.find((a) => a.id === id);
      if (album) rows.push(kind === "expanded" ? { kind: "expanded", album } : { kind: "ghost", id });
    }
  }
  return rows;
}
