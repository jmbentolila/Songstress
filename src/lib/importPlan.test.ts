import { describe, expect, it } from "vitest";
import {
  applyOrder,
  destinationLine,
  fmtDuration,
  groupByArtist,
  plural,
  shortFolder,
  summarize,
  trackNumbers,
  totalTracks,
  type Decision,
  type StagedAlbum,
} from "./importPlan";

const ROOTS = ["/home/yossi/Music"];

function staged(
  albumId: string,
  artist: string,
  title: string,
  rule: "merge" | "artist" | "new",
  folder: string,
  tracks = 2,
  mergedInto: string | null = null,
): StagedAlbum {
  return {
    albumId,
    artist,
    title,
    year: 2022,
    tracks: Array.from({ length: tracks }, (_, i) => ({
      id: `${albumId}-t${i}`,
      title: `Track ${i + 1}`,
      durationSec: 61 + i,
      disc: 1,
      track: i + 1,
    })),
    destination: { folder, rule, mergedInto },
  };
}

describe("groupByArtist", () => {
  it("groups by artist and orders albums by title inside the group", () => {
    const groups = groupByArtist([
      staged("b", "Ghost", "Prequelle", "artist", "/home/yossi/Music/Music Files/Ghost/Prequelle"),
      staged("a", "Axel Rudi Pell", "The Ballads III", "artist", "/home/yossi/Music/Music Files/ARP/Ballads"),
      staged("c", "Ghost", "Impera", "merge", "/home/yossi/Music/Music Files/Ghost/Impera", 11),
    ]);
    expect(groups.map((g) => g.artist)).toEqual(["Axel Rudi Pell", "Ghost"]);
    expect(groups[1].albums.map((a) => a.title)).toEqual(["Impera", "Prequelle"]);
    expect(groups[1].trackCount).toBe(13); // 2 + 11, counted from the plan, not the labels
  });

  it("apply walks the list in the order it is shown in", () => {
    const plan = [
      staged("b", "Ghost", "Prequelle", "artist", "/r/Ghost/Prequelle"),
      staged("a", "Axel Rudi Pell", "The Ballads III", "artist", "/r/ARP/Ballads"),
      staged("c", "Ghost", "Impera", "merge", "/r/Ghost/Impera"),
    ];
    const decisions: Record<string, Decision> = { c: "save", a: "discard" };
    expect(applyOrder(plan, decisions).map((o) => [o.label, o.decision])).toEqual([
      ["Axel Rudi Pell — The Ballads III", "discard"],
      ["Ghost — Impera", "save"],
    ]);
  });
});

describe("summarize", () => {
  it("counts the pile and the decisions separately", () => {
    const plan = [
      staged("a", "Ghost", "Impera", "merge", "/r/Ghost/Impera", 11),
      staged("b", "Ghost", "Popestar", "artist", "/r/Ghost/Popestar", 5),
      staged("c", "Ayreon", "The Human Equation", "new", "/r/Ayreon/HE", 20),
    ];
    const s = summarize(plan, { a: "save", b: "discard" });
    expect(s).toEqual({ albums: 3, tracks: 36, decided: 2, save: 1, discard: 1 });
    expect(totalTracks(plan)).toBe(36);
  });

  it("ignores a decision for an album that is no longer staged", () => {
    // Apply removes rows as it goes; a stale key must not inflate the count.
    expect(summarize([], { gone: "save" }).decided).toBe(0);
  });
});

describe("shortFolder", () => {
  it("drops the music folder the user already knows", () => {
    expect(shortFolder("/home/yossi/Music/Music Files/Ghost/Impera", ROOTS)).toBe(
      "Music Files/Ghost/Impera",
    );
  });

  it("uses the longest containing folder when several are configured", () => {
    const roots = ["/home/yossi/Music", "/home/yossi/Music/Music Files"];
    expect(shortFolder("/home/yossi/Music/Music Files/Ghost/Impera", roots)).toBe(
      "Ghost/Impera",
    );
  });

  it("keeps the whole path when nothing contains it", () => {
    // A folder removed from settings, or a staging path: showing a tail would
    // imply it is inside something it is not.
    expect(shortFolder("/tmp/staging/Impera", ROOTS)).toBe("/tmp/staging/Impera");
  });

  it("tolerates a root written with or without a trailing slash", () => {
    expect(shortFolder("/home/yossi/Music/Music Files/Ghost", ["/home/yossi/Music/"])).toBe(
      "Music Files/Ghost",
    );
  });
});

describe("destinationLine", () => {
  it("names the album a merge joins", () => {
    const a = staged("a", "Ghost", "Impera", "merge", "/home/yossi/Music/Music Files/Ghost/Impera", 1, "Impera");
    expect(destinationLine(a, ROOTS)).toEqual({
      lead: "merges into Impera",
      path: "Music Files/Ghost/Impera",
    });
  });

  it("says a new album joins the artist's folder", () => {
    const a = staged("b", "Ghost", "Popestar", "artist", "/home/yossi/Music/Music Files/Ghost/Popestar");
    expect(destinationLine(a, ROOTS).lead).toBe("new album under");
  });

  it("says a new artist starts a folder", () => {
    const a = staged("c", "Kadavar", "For The Dead", "new", "/home/yossi/Music/Music Files/Kadavar/For The Dead");
    expect(destinationLine(a, ROOTS)).toEqual({
      lead: "new artist folder",
      path: "Music Files/Kadavar/For The Dead",
    });
  });

  it("never claims a merge target it does not have", () => {
    const a = staged("d", "Ghost", "Impera", "merge", "/r/Impera", 1, null);
    expect(destinationLine(a, ROOTS).lead).toContain("already");
  });
});

describe("listing", () => {
  it("shows disc numbers only when the album has more than one disc", () => {
    const one = staged("a", "Ghost", "Impera", "merge", "/r/Impera", 3);
    expect(trackNumbers(one)).toEqual(["1", "2", "3"]);
    const two = staged("b", "Ghost", "Impera", "merge", "/r/Impera", 3);
    two.tracks[2].disc = 2;
    expect(trackNumbers(two)).toEqual(["1-1", "1-2", "2-3"]);
  });

  it("falls back to the position when the tag has no track number", () => {
    const a = staged("c", "Ghost", "Impera", "merge", "/r/Impera", 2);
    a.tracks.forEach((t) => (t.track = null));
    expect(trackNumbers(a)).toEqual(["1", "2"]);
  });

  it("formats durations as m:ss and never goes negative", () => {
    expect(fmtDuration(187)).toBe("3:07");
    expect(fmtDuration(0)).toBe("0:00");
    expect(fmtDuration(-5)).toBe("0:00");
  });

  it("pluralizes the footer", () => {
    expect(plural(1, "album")).toBe("1 album");
    expect(plural(2, "album")).toBe("2 albums");
    expect(plural(1234, "track")).toBe("1,234 tracks");
  });
});
