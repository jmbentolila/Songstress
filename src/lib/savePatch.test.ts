import { describe, expect, it } from "vitest";
import { applySavePatchToDump, type SavePatch } from "./savePatch";
import type { Album, Artist, Track } from "./types";

function artist(id: string, name: string): Artist {
  return { id, name, sortName: name.toLowerCase() };
}

function album(id: string, artistId: string, staged = false): Album {
  return {
    id,
    artistId,
    title: `title-${id}`,
    year: 2015,
    cover: null,
    staged,
  };
}

function track(id: string, albumId: string, title: string, staged = false): Track {
  return {
    id,
    albumId,
    disc: 1,
    track: null,
    title,
    durationSec: 60,
    staged,
  };
}

describe("applySavePatchToDump", () => {
  it("renames moved rows in place and clears staged", () => {
    const artists = [artist("ar-1", "Ghost")];
    const albums = [album("al-1", "ar-1", true)];
    const tracks = [track("tr-old", "al-1", "probe", true)];
    const patch: SavePatch = {
      moved: [{ oldId: "tr-old", newId: "tr-new" }],
      removed: [],
    };
    const out = applySavePatchToDump(artists, albums, tracks, patch);
    expect(out.applied).toBe(1);
    expect(out.tracks).toEqual([{ ...tracks[0], id: "tr-new", staged: false }]);
    // Same album row (tag-derived ids never move), staged recomputed.
    expect(out.albums).toEqual([{ ...albums[0], staged: false }]);
    expect(out.artists).toEqual(artists);
  });

  it("drops removed rows and prunes emptied albums and artists", () => {
    const artists = [artist("ar-1", "Ghost"), artist("ar-2", "Other")];
    const albums = [album("al-1", "ar-1", true), album("al-2", "ar-2")];
    const tracks = [
      track("tr-gone", "al-1", "ghost", true),
      track("tr-keep", "al-2", "keep"),
    ];
    const out = applySavePatchToDump(artists, albums, tracks, {
      moved: [],
      removed: ["tr-gone"],
    });
    expect(out.applied).toBe(1);
    expect(out.tracks.map((t) => t.id)).toEqual(["tr-keep"]);
    expect(out.albums.map((a) => a.id)).toEqual(["al-2"]);
    expect(out.artists.map((a) => a.id)).toEqual(["ar-2"]);
  });

  it("keeps order and leaves unknown ids as skips", () => {
    const artists = [artist("ar-1", "Ghost")];
    const albums = [album("al-1", "ar-1")];
    const tracks = [track("tr-a", "al-1", "a"), track("tr-b", "al-1", "b")];
    const out = applySavePatchToDump(artists, albums, tracks, {
      moved: [{ oldId: "tr-nope", newId: "tr-new" }],
      removed: ["tr-missing"],
    });
    expect(out.applied).toBe(0);
    expect(out.tracks).toEqual(tracks);
    expect(out.albums).toEqual(albums);
    expect(out.artists).toEqual(artists);
  });

  it("an empty patch changes nothing", () => {
    const artists = [artist("ar-1", "Ghost")];
    const albums = [album("al-1", "ar-1")];
    const tracks = [track("tr-a", "al-1", "a")];
    const out = applySavePatchToDump(artists, albums, tracks, {
      moved: [],
      removed: [],
    });
    expect(out.applied).toBe(0);
    expect(out.tracks).toEqual(tracks);
  });

  it("does not mutate its inputs", () => {
    const artists = [artist("ar-1", "Ghost")];
    const albums = [album("al-1", "ar-1", true)];
    const tracks = [track("tr-old", "al-1", "probe", true)];
    applySavePatchToDump(artists, albums, tracks, {
      moved: [{ oldId: "tr-old", newId: "tr-new" }],
      removed: [],
    });
    expect(tracks[0].id).toBe("tr-old");
    expect(tracks[0].staged).toBe(true);
    expect(albums[0].staged).toBe(true);
  });
});
