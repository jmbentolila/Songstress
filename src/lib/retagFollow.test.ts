import { describe, expect, it } from "vitest";
import { resolveRetagFollow } from "./retagFollow";
import type { Album, Artist, Track } from "./types";

const artists: Artist[] = [
  { id: "ar-ghost", name: "Ghost", sortName: "ghost" },
  { id: "ar-other", name: "Other", sortName: "other" },
];

const albums: Album[] = [
  { id: "al-old", artistId: "ar-ghost", title: "Old", year: 2015, cover: null },
  { id: "al-new", artistId: "ar-other", title: "New", year: 2020, cover: null },
];

function track(id: string, albumId: string): Track {
  return { id, albumId, disc: 1, track: null, title: id, durationSec: 60 };
}

describe("resolveRetagFollow", () => {
  it("follows moved tracks to the destination album tab", () => {
    const byId = new Map([["tr-1", track("tr-1", "al-new")]]);
    expect(
      resolveRetagFollow(albums, artists, (id) => byId.get(id), {
        fromAlbumId: "al-old",
        trackIds: ["tr-1"],
      }),
    ).toEqual({ albumId: "al-new", artistId: "ar-other" });
  });

  it("stays put when the album identity did not move", () => {
    const byId = new Map([["tr-1", track("tr-1", "al-old")]]);
    expect(
      resolveRetagFollow(albums, artists, (id) => byId.get(id), {
        fromAlbumId: "al-old",
        trackIds: ["tr-1"],
      }),
    ).toBeNull();
  });

  it("stays put when the tracks are unknown", () => {
    const byId = new Map<string, Track>();
    expect(
      resolveRetagFollow(albums, artists, (id) => byId.get(id), {
        fromAlbumId: "al-old",
        trackIds: ["tr-gone"],
      }),
    ).toBeNull();
  });

  it("stays put when the destination album is missing", () => {
    const byId = new Map([["tr-1", track("tr-1", "al-vanished")]]);
    expect(
      resolveRetagFollow(albums, artists, (id) => byId.get(id), {
        fromAlbumId: "al-old",
        trackIds: ["tr-1"],
      }),
    ).toBeNull();
  });

  it("follows the first touched track of a multi-album save", () => {
    const byId = new Map([
      ["tr-1", track("tr-1", "al-new")],
      ["tr-2", track("tr-2", "al-old")],
    ]);
    expect(
      resolveRetagFollow(albums, artists, (id) => byId.get(id), {
        fromAlbumId: "al-old",
        trackIds: ["tr-1", "tr-2"],
      }),
    ).toEqual({ albumId: "al-new", artistId: "ar-other" });
  });
});
