import { describe, expect, it } from "vitest";
import { buildRows, columnCount } from "./buildRows";
import type { Album } from "./types";

function album(id: string): Album {
  return { id, artistId: "a1", title: id, year: 2000, cover: null };
}

describe("columnCount", () => {
  it("matches CSS auto-fill minmax math", () => {
    // floor((W + gap) / (tile + gap))
    expect(columnCount(1000, 180, 20)).toBe(5);
    expect(columnCount(999, 180, 20)).toBe(5);
    expect(columnCount(960, 180, 20)).toBe(4);
    expect(columnCount(300, 180, 20)).toBe(1);
  });

  it("clamps to at least one column", () => {
    expect(columnCount(0, 180, 20)).toBe(1);
    expect(columnCount(500, -1, 20)).toBe(1);
  });
});

describe("buildRows", () => {
  const albums = ["a", "b", "c", "d", "e"].map(album);

  it("chunks albums into rows of N", () => {
    const rows = buildRows(albums, 2, null);
    expect(rows).toHaveLength(3);
    expect(rows[0]).toEqual({ kind: "albums", items: [albums[0], albums[1]] });
    expect(rows[2]).toEqual({ kind: "albums", items: [albums[4]] });
  });

  it("inserts an expanded row directly after the row containing the album", () => {
    const rows = buildRows(albums, 2, "c");
    expect(rows.map((r) => r.kind)).toEqual(["albums", "albums", "expanded", "albums"]);
    expect(rows[1]).toEqual({
      kind: "albums",
      items: [albums[2], albums[3]],
    });
    expect(rows[2]).toEqual({ kind: "expanded", album: albums[2] });
  });

  it("handles expansion in the last partial row", () => {
    const rows = buildRows(albums, 2, "e");
    expect(rows.map((r) => r.kind)).toEqual(["albums", "albums", "albums", "expanded"]);
    expect(rows[3]).toEqual({ kind: "expanded", album: albums[4] });
  });

  it("ignores an expanded id not present in the filtered set", () => {
    const rows = buildRows(albums, 2, "missing");
    expect(rows.every((r) => r.kind === "albums")).toBe(true);
  });

  it("handles empty input and degenerate column counts", () => {
    expect(buildRows([], 3, null)).toEqual([]);
    expect(buildRows(albums, 0, null)).toHaveLength(5);
  });

});
