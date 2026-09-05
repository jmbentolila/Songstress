import { describe, expect, it } from "vitest";
import {
  libraryLoading,
  COUNT_W,
  MIN_SKELETON_MS,
  NAME_W,
  sidebarRows,
  skeletonRows,
  type LoadingFacts,
} from "./loadingState";

const f = (over: Partial<LoadingFacts>): LoadingFacts => ({
  ready: true,
  albums: 0,
  running: false,
  scanning: false,
  ...over,
});

describe("libraryLoading", () => {
  it("is loading while a run is in flight over an empty library", () => {
    expect(libraryLoading(f({ running: true }))).toBe(true);
    // a watcher-triggered scan: the frontend never asked, so `running` is false
    expect(libraryLoading(f({ scanning: true }))).toBe(true);
  });

  it("is never loading over content that exists", () => {
    // a re-scan of a populated library keeps the real grid + the sidebar note
    expect(libraryLoading(f({ albums: 251, running: true, scanning: true }))).toBe(false);
  });

  it("is not loading when the library is simply empty", () => {
    // that is the welcome state's job, not a placeholder's
    expect(libraryLoading(f({}))).toBe(false);
  });

  it("is loading for the whole of every boot, fast dump or not", () => {
    // owner ruling 2026-09-06 (the RPM launch): the boot dump IS a load;
    // hiding it made a warm launch a content pop. The store holds the first
    // dump out for MIN_SKELETON_MS so "from the first frame" is no flicker.
    expect(libraryLoading(f({ ready: false }))).toBe(true);
    // ...and a scan that starts before the dump lands changes nothing.
    expect(libraryLoading(f({ ready: false, running: true }))).toBe(true);
  });

  it("has a floor so the boot placeholder is a hand-over, not a flicker", () => {
    // Below ~2 frames a placeholder is a manufactured delay; 250 ms is
    // "reads as intentional", and the measured 46–50 ms warm dump means
    // every launch pays the floor and nothing more.
    expect(MIN_SKELETON_MS).toBeGreaterThanOrEqual(200);
    expect(MIN_SKELETON_MS).toBeLessThanOrEqual(300);
  });
});

describe("skeletonRows", () => {
  it("fills the user's 924x536 stage with two rows of 220px tiles", () => {
    // 3 cols, tileW 294.7 → rowH ~347 → ceil(536 / 367) = 2
    expect(skeletonRows(924, 536, 220)).toBe(2);
  });

  it("does not stack an infinite page on small tiles", () => {
    // 6 cols of 100px at a tall stage: capped, not 14 rows
    expect(skeletonRows(700, 2400, 100)).toBe(4);
  });

  it("always shows at least one row, and survives an unmeasured stage", () => {
    expect(skeletonRows(0, 0, 220)).toBe(1);
    expect(skeletonRows(924, 100, 220)).toBe(1);
  });

  it("tracks the column count of the real grid", () => {
    // one 400px-wide column at 300px tiles: a single tall row, not a grid
    expect(skeletonRows(400, 300, 300)).toBe(1);
  });
});

describe("sidebarRows", () => {
  it("fits the nav without overflowing it, minus the real All Artists row", () => {
    expect(sidebarRows(400, 40)).toBe(9);
    expect(sidebarRows(415, 40)).toBe(9);
  });

  it("keeps a floor for a short or unmeasured nav", () => {
    expect(sidebarRows(60, 40)).toBe(3);
    expect(sidebarRows(0, 40)).toBe(6);
  });
});

describe("pill widths", () => {
  it("are deterministic and inside their column", () => {
    for (const w of [...NAME_W, ...COUNT_W]) {
      expect(w).toBeGreaterThan(0.1);
      expect(w).toBeLessThan(1);
    }
    expect(NAME_W).toHaveLength(6);
  });
});
