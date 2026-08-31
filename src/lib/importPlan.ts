/**
 * The shape and the reading of a staged-import plan.
 *
 * The plan comes from Rust (`staged_import_plan`) already resolved: every staged
 * album carries the folder its files would move to and WHICH RULE chose it. That
 * decision is what the modal is for, so the sentence that shows it lives here,
 * pure and tested, rather than being assembled inside a template where a wording
 * change breaks nothing and a wrong path breaks everything.
 */

export type Rule = "merge" | "artist" | "new";
export type Decision = "save" | "discard";

export interface StagedTrack {
  id: string;
  title: string;
  durationSec: number;
  disc: number;
  track: number | null;
}

export interface Destination {
  /** Absolute path the files would move to. */
  folder: string;
  /** Which cascade rule picked it, so the UI can say so out loud. */
  rule: Rule;
  /** The album these files join, for "merge". */
  mergedInto?: string | null;
}

export interface StagedAlbum {
  albumId: string;
  artist: string;
  title: string;
  year: number | null;
  /** The staged files, in listing order (disc, then track number). */
  tracks: StagedTrack[];
  destination: Destination;
}

/** One album as the import receipt names it: no destination, because the point
 *  of the receipt is that there was nothing to decide. */
export interface ImportedAlbum {
  artist: string;
  title: string;
  tracks: number;
}

/** What an import did. `already` is the half that needs explaining: pointing at
 *  files the library already indexes is not a failure, but a progress ring that
 *  ends with nothing on screen reads as one. */
export interface ImportReport {
  staged: ImportedAlbum[];
  already: ImportedAlbum[];
}

/** What one Save did, from Rust. `duplicates` means the user's file was removed
 *  because the library already held those bytes — the only deletion of a file
 *  they own that this app performs. */
export interface SaveReport {
  moved: number;
  duplicates: number;
  vanished: number;
}

export interface ArtistGroup {
  artist: string;
  albums: StagedAlbum[];
  trackCount: number;
}

export function plural(n: number, one: string, many = `${one}s`): string {
  return `${n.toLocaleString()} ${n === 1 ? one : many}`;
}

export function totalTracks(plan: StagedAlbum[]): number {
  return plan.reduce((n, a) => n + a.tracks.length, 0);
}

/**
 * Grouped by artist, because the destination cascade is artist-shaped (merge
 * into the album, else join the artist's folder, else start one) and the tree
 * should read like the layout it is describing. Album order within a group is
 * also the order Apply works through, so the list on screen and the order of
 * operations cannot disagree.
 */
export function groupByArtist(plan: StagedAlbum[]): ArtistGroup[] {
  const byArtist = new Map<string, StagedAlbum[]>();
  for (const a of plan) {
    const list = byArtist.get(a.artist);
    if (list) list.push(a);
    else byArtist.set(a.artist, [a]);
  }
  return [...byArtist.entries()]
    .sort((x, y) => x[0].localeCompare(y[0]))
    .map(([artist, albums]) => ({
      artist,
      albums: [...albums].sort((x, y) => x.title.localeCompare(y.title)),
      trackCount: albums.reduce((n, a) => n + a.tracks.length, 0),
    }));
}

/** The order Apply performs the decisions in: on-screen order, group by group. */
export function applyOrder(
  plan: StagedAlbum[],
  decisions: Record<string, Decision>,
): { albumId: string; label: string; decision: Decision }[] {
  const ops: { albumId: string; label: string; decision: Decision }[] = [];
  for (const group of groupByArtist(plan)) {
    for (const a of group.albums) {
      const d = decisions[a.albumId];
      if (d) ops.push({ albumId: a.albumId, label: `${a.artist} — ${a.title}`, decision: d });
    }
  }
  return ops;
}

export interface Summary {
  albums: number;
  tracks: number;
  decided: number;
  save: number;
  discard: number;
}

export function summarize(plan: StagedAlbum[], decisions: Record<string, Decision>): Summary {
  let save = 0;
  let discard = 0;
  for (const a of plan) {
    if (decisions[a.albumId] === "save") save++;
    else if (decisions[a.albumId] === "discard") discard++;
  }
  return { albums: plan.length, tracks: totalTracks(plan), decided: save + discard, save, discard };
}

/**
 * The folder relative to the music folder that contains it — `Music Files/Ghost/
 * Impera`, not `/home/yossi/Music/Music Files/Ghost/Impera`. A rooted path wastes
 * the line on a prefix the user already knows and cannot change; the full path
 * stays available as the title. Falls back to the whole path when no configured
 * folder contains it (the staging dir, a folder removed from settings).
 */
export function shortFolder(folder: string, roots: string[]): string {
  let best = "";
  for (const r of roots) {
    const prefix = r.endsWith("/") ? r : `${r}/`;
    if (folder.startsWith(prefix) && prefix.length > best.length) best = prefix;
  }
  return best ? folder.slice(best.length) : folder;
}

/**
 * The sentence under an album: what will happen, then where. Deliberately does
 * not repeat the album title ("joins Impera" is enough — the title is the line
 * above it), and never says "save" (that is the button's word, not the
 * destination's).
 */
export function destinationLine(a: StagedAlbum, roots: string[]): { lead: string; path: string } {
  const path = shortFolder(a.destination.folder, roots);
  switch (a.destination.rule) {
    case "merge":
      return {
        lead: a.destination.mergedInto
          ? `merges into ${a.destination.mergedInto}`
          : "merges into the copy already in your library",
        path,
      };
    case "artist":
      return { lead: "new album under", path };
    case "new":
    default:
      return { lead: "new artist folder", path };
  }
}

/** "3:47", the same shape the playbar and the expanded panel use. */
export function fmtDuration(sec: number): string {
  const s = Math.max(0, Math.floor(sec));
  const m = Math.floor(s / 60);
  return `${m}:${String(s % 60).padStart(2, "0")}`;
}

/** Track number for a listing: the tag if it has one, else its position. Disc
 *  numbers appear only when the album actually has more than one. */
export function trackNumbers(album: StagedAlbum): string[] {
  const multi = album.tracks.some((t) => t.disc > 1);
  return album.tracks.map((t, i) => {
    const n = t.track ?? i + 1;
    return multi ? `${t.disc}-${n}` : String(n);
  });
}
