export interface Artist {
  id: string;
  name: string;
  sortName: string;
}

export interface Album {
  id: string;
  artistId: string;
  title: string;
  year: number | null;
  cover: string | null;
  /** Panel gradient colors, hex — computed during scan (Phase 2 M3). */
  colorC1?: string | null;
  colorC2?: string | null;
  /** Lives in the import staging area (Step 2a), not the library dir yet. */
  staged?: boolean;
}

export interface Track {
  id: string;
  albumId: string;
  disc: number;
  /** Track number — null for unnumbered tracks (sorted alphabetically). */
  track: number | null;
  title: string;
  durationSec: number;
  /** Per-track artist from the file's tags — null/empty means fall back to
   * the album artist (pre-v3 rows, untagged files). */
  artist?: string | null;
  /** Lives in the import staging area (Step 2a), not the library dir yet. */
  staged?: boolean;
  /** The file behind this row vanished from disk (kept for relink/removal). */
  missing?: boolean;
}

export interface PlaybackContext {
  albumId: string;
  trackIndex: number;
  /** Anchors the context across rescans: the index alone drifts when a
   * scan re-orders/re-groups the album while something is playing. */
  trackId?: string;
}

export type Theme = "light" | "dark";
