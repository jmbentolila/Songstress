import { describe, expect, it } from "vitest";
import { adjacentAlbum, globalAlbumOrder } from "./albumOrder";
import type { Album, Artist } from "./types";

const artists: Artist[] = [
  { id: "ar-1", name: "The Beatles", sortName: "beatles" },
  { id: "ar-2", name: "Ayreon", sortName: "ayreon" },
];

function album(id: string, artistId: string, title: string, year: number | null): Album {
  return { id, artistId, title, year, cover: null };
}

const albums = [
  album("al-late", "ar-1", "Let It Be", 1970),
  album("al-early", "ar-1", "Help!", 1965),
  album("al-ayreon", "ar-2", "Into the Electric Castle", 1998),
  album("al-noyear", "ar-2", "Unreleased", null),
];

describe("globalAlbumOrder", () => {
  it("sorts by artist sort_name, then year, nulls last", () => {
    expect(globalAlbumOrder(albums, artists).map((a) => a.id)).toEqual([
      "al-ayreon",
      "al-noyear",
      "al-early",
      "al-late",
    ]);
  });
});

describe("adjacentAlbum", () => {
  it("steps forward and backward in the global order", () => {
    expect(adjacentAlbum(albums, artists, "al-early", 1)?.id).toBe("al-late");
    expect(adjacentAlbum(albums, artists, "al-early", -1)?.id).toBe("al-noyear");
  });

  it("wraps around both ends", () => {
    expect(adjacentAlbum(albums, artists, "al-late", 1)?.id).toBe("al-ayreon");
    expect(adjacentAlbum(albums, artists, "al-ayreon", -1)?.id).toBe("al-late");
  });

  it("returns null for unknown albums or empty order", () => {
    expect(adjacentAlbum(albums, artists, "al-nope", 1)).toBeNull();
    expect(adjacentAlbum([], artists, "al-early", 1)).toBeNull();
  });
});
