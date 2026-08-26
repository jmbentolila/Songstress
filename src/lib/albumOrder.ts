import type { Album, Artist } from "./types";

/** Global grid order (Step 5b): artist sort_name A→Z, then year (nulls
 *  last), then title — mirrors AlbumGrid's All Artists sort. */
export function globalAlbumOrder(albums: Album[], artists: Artist[]): Album[] {
  const byId = new Map(artists.map((a) => [a.id, a]));
  return [...albums].sort((a, b) => {
    const an = byId.get(a.artistId)?.sortName ?? "";
    const bn = byId.get(b.artistId)?.sortName ?? "";
    if (an !== bn) return an.localeCompare(bn);
    if (a.year === null && b.year !== null) return 1;
    if (b.year === null && a.year !== null) return -1;
    if (a.year !== b.year) return (a.year ?? 0) - (b.year ?? 0);
    return a.title.localeCompare(b.title);
  });
}

/** The album `delta` steps from `currentId` in the global order, wrapping
 *  around. Null when the order is empty or the album is unknown. */
export function adjacentAlbum(
  albums: Album[],
  artists: Artist[],
  currentId: string,
  delta: 1 | -1,
): Album | null {
  const order = globalAlbumOrder(albums, artists);
  const i = order.findIndex((a) => a.id === currentId);
  if (i === -1 || order.length === 0) return null;
  const next = (i + delta + order.length) % order.length;
  return order[next];
}
