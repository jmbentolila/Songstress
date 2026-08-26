import { describe, expect, it } from "vitest";
import { albumMatches, albumTitleMatches, albumTrackMatches, fold } from "./search";

const album = {
  title: "Land of Light",
  artistName: "Aldaria",
  trackTitles: ["Excitare Ad Lucem", "Another Life", "Guardians of The Light"],
};

describe("fold", () => {
  it("lowercases and strips diacritics", () => {
    expect(fold("Motörhead")).toBe("motorhead");
    expect(fold("Sigur Rós")).toBe("sigur ros");
  });
});

describe("albumMatches", () => {
  it("matches by album title, case-insensitive", () => {
    expect(albumMatches(album, "land of")).toBe(true);
    expect(albumMatches(album, "LAND")).toBe(true);
  });

  it("does NOT match by artist name (sidebar handles artists)", () => {
    expect(albumMatches(album, "aldar")).toBe(false);
  });

  it("matches by any track title", () => {
    expect(albumMatches(album, "guardians")).toBe(true);
    expect(albumMatches(album, "another life")).toBe(true);
  });

  it("rejects non-matches", () => {
    expect(albumMatches(album, "throne")).toBe(false);
    expect(albumMatches(album, "xyzzy")).toBe(false);
  });

  it("folds diacritics in the query and the haystack", () => {
    expect(
      albumMatches({ ...album, title: "Motörhead" }, "motorhead"),
    ).toBe(true);
    expect(albumMatches(album, "excitare ad lucem")).toBe(true);
  });

  it("empty or blank query matches everything", () => {
    expect(albumMatches(album, "")).toBe(true);
    expect(albumMatches(album, "   ")).toBe(true);
  });
});

describe("albumTitleMatches / albumTrackMatches", () => {
  it("title matcher matches album titles only, never artist or track names", () => {
    expect(albumTitleMatches(album, "guardians")).toBe(false);
    expect(albumTitleMatches(album, "aldaria")).toBe(false);
    expect(albumTitleMatches(album, "land of light")).toBe(true);
  });

  it("track matcher ignores album/artist-only matches", () => {
    expect(albumTrackMatches(album, "land of light")).toBe(false);
    expect(albumTrackMatches(album, "another life")).toBe(true);
  });

  it("blank queries match everything in both", () => {
    expect(albumTitleMatches(album, "")).toBe(true);
    expect(albumTrackMatches(album, "  ")).toBe(true);
  });
});
