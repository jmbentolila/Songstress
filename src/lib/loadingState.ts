import { columnCount } from "./buildRows";

/**
 * The ONE definition of "this surface has nothing to show yet, and something
 * is in flight to fix it". Both the album grid and the sidebar's artist list
 * read it, because two placeholders that disagree about when they exist is
 * the bug report ("the grid is loading but the sidebar says no albums").
 *
 * It takes plain facts rather than the stores themselves so the rule can be
 * stated — and tested — without a $state graph behind it.
 */
/**
 * How long the boot placeholder is PROUD to be on screen. A placeholder that
 * appears and disappears inside a couple of frames is a delay we manufactured
 * (the import manager's rule), and the measured warm dump is 46–50 ms — so
 * "skeleton from the first frame" only holds together with a floor: the
 * store holds the first dump out until this much time has passed, and the
 * arrival then reads as intentional (owner ruling 2026-09-06: the loader
 * belongs to every launch's first load — a silent content pop, even from a
 * warm cache, reads as a glitch, which is what the RPM launch showed).
 * The old design bet on the grace timer (`BOOT_GRACE_MS`, 150 ms) that a
 * fast dump didn't deserve a placeholder at all; the RPM launch proved the
 * loss reads worse than the flicker the timer bought against.
 *
 * `NOTE_GRACE_MS` gates the sidebar's "Updating library…" on a rescan of a
 * POPULATED library — an incremental watcher pass over unchanged files is
 * ~56ms end to end, so most background rescans should never show it at all
 * (measured 2026-09-02: dump 46–50ms, watcher rescan 83ms; 400ms is above
 * the whole class of background pass, below anything long enough to news).
 */
export const MIN_SKELETON_MS = 250;
export const NOTE_GRACE_MS = 400;

/** How long the arrival entrance may stay armed: the grid's 320ms animation plus
 * its 540ms diagonal cap (the slowest of the surfaces — the list tops out at 740),
 * rounded up. It MUST exceed duration + cap: a row still holding at `opacity: 0`
 * inside its delay snaps visible when the class goes away, which is a pop at the
 * bottom of the cascade. And it must not be permanent — left on, every later filter
 * change would run its freshly-mounted rows through the entrance, and an artist
 * switch or a search keystroke is the frequency tier where motion is disqualified. */
export const ENTER_MS = 900;

export interface LoadingFacts {
  /** The boot `get_library` dump has landed (and been held out for
   * `MIN_SKELETON_MS`, so this flipping is the same moment the skeleton
   * retires — never a race the components must pace). */
  ready: boolean;
  /** Albums currently indexed. Anything more than zero means the library is
   * NOT empty, and a skeleton must never cover content that exists. */
  albums: number;
  /** A run the frontend started (menu / empty state / import) — `scanner.running`. */
  running: boolean;
  /** A `scan-progress` event has been seen. This is the half that `running`
   * cannot cover: a WATCHER-triggered first scan never goes through the
   * frontend's `begin()`, so without it the grid would flash the welcome screen
   * mid-scan and then jump. */
  scanning: boolean;
  /** DEV-only force (see `window.__skel` in main.ts). */
  devLoading?: boolean;
}

/**
 * Boot is a placeholder by right, on every launch (owner ruling 2026-09-06):
 * the launch dump IS a load, and hiding it made a warm launch a content pop.
 * The store holds the first dump until `MIN_SKELETON_MS` has been served, so
 * "from the first frame" never means "a 50 ms flicker". A scan in flight over
 * an EMPTY library is loading too — but never over content that exists.
 */
export function libraryLoading(f: LoadingFacts): boolean {
  if (f.devLoading) return true;
  if (!f.ready) return true;
  if (f.albums > 0) return false;
  return f.running || f.scanning;
}

/** The non-cover height of a real `.tile` (AlbumGrid): 13px cover→caption gap +
 * caption 14/12px lines with a 2px gap + 8px bottom padding, measured against
 * the real thing at 52px. The skeleton's own CSS lands 3px short of that; the
 * count is what matters here, and 3px of a 350px row is not a reflow. */
export const TILE_STACK = 52;

/** Four rows is already ~1400px of fake covers. More is a wall, not a preview. */
export const MAX_SK_ROWS = 4;

/** How many SKELETON rows fill the stage without inventing a second screen of
 * nothing: a placeholder you can scroll past is an infinite empty page.
 * `ceil` deliberately lets the last row run under the playbar shelf — that is
 * what the real first row does too. */
export function skeletonRows(
  gridWidth: number,
  stageHeight: number,
  tileSize: number,
  gap = 20,
): number {
  if (gridWidth <= 0 || stageHeight <= 0) return 1;
  const cols = columnCount(gridWidth, tileSize, gap);
  const tileW = (gridWidth - (cols - 1) * gap) / cols;
  const rowH = tileW + TILE_STACK;
  return Math.max(1, Math.min(MAX_SK_ROWS, Math.ceil(stageHeight / (rowH + gap))));
}

/** How many skeleton artist rows fit the nav: `floor`, because the sidebar is
 * the one surface whose scrollbar the user can actually see, and a loading list
 * that scrolls is a lie about how many artists there are. One slot belongs to the
 * always-visible "All Artists" control, which is real and never skeletonized. */
export function sidebarRows(navHeight: number, rowSize: number): number {
  if (navHeight <= 0 || rowSize <= 0) return 6;
  return Math.max(3, Math.floor(navHeight / rowSize) - 1);
}

/**
 * Pill widths, in fractions of the column. Fixed arrays, not `Math.random()`:
 * a placeholder list re-renders on every progress event — one roughly every
 * 6ms during a scan — and a fresh random width each time shimmers on its own.
 * Varied widths are the point: uniform bars read as a table, not as names.
 */
export const NAME_W = [0.66, 0.44, 0.58, 0.36, 0.5, 0.6];
export const COUNT_W = [0.34, 0.24, 0.3, 0.2];
