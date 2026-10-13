import type { Album, Artist, Track } from "./types";

/**
 * A tag save arming the tab-follow: the album being edited and the track ids
 * it holds. Track ids survive a retag (paths don't move); album ids don't
 * (they're tag-derived) — so the follow resolves the tracks post-scan and
 * lands wherever they regrouped.
 */
export interface RetagFollow {
  fromAlbumId: string;
  trackIds: string[];
}

/**
 * Where a tag save should land the view, or null for stay-put. Rules:
 * destination unknown (failed save, deleted rows) → stay; same album (a
 * title/genre touch-up that kept identity) → stay; destination album or its
 * artist missing from the dump → stay (never strand on a missing tab).
 * Multi-album saves follow the first touched track — one album per editor
 * in practice, and the stepper saves track by track. Pure so the store
 * stays a thin shell (covered in retagFollow.test.ts).
 */
export function resolveRetagFollow(
  albums: Album[],
  artists: Artist[],
  trackById: (id: string) => Track | undefined,
  follow: RetagFollow,
): { albumId: string; artistId: string } | null {
  const hit = follow.trackIds.map(trackById).find((t) => t !== undefined);
  if (!hit) return null;
  if (hit.albumId === follow.fromAlbumId) return null;
  const dest = albums.find((a) => a.id === hit.albumId);
  if (!dest) return null;
  if (!artists.some((a) => a.id === dest.artistId)) return null;
  return { albumId: dest.id, artistId: dest.artistId };
}
