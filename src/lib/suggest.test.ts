import { describe, expect, it } from "vitest";
import { fold, highlightIndex, rankSuggestions, shouldFlipAbove } from "./suggest";

const v = (value: string, count = 1) => ({ value, count });

describe("fold", () => {
  it("strips case, diacritics and surrounding space", () => {
    expect(fold("  Motörhead ")).toBe("motorhead");
  });

  it("keeps a leading article (unlike sort.ts's sortKey)", () => {
    expect(fold("The Beatles")).toBe("the beatles");
  });
});

describe("rankSuggestions", () => {
  it("puts prefix matches before substring matches", () => {
    const out = rankSuggestions(
      [v("Classic Rock"), v("Rock"), v("Rockabilly")],
      "rock",
    );
    expect(out).toEqual(["Rock", "Rockabilly", "Classic Rock"]);
  });

  it("orders by closeness — earliest inclusion, then shortest value", () => {
    const out = rankSuggestions(
      [v("Alternative", 9), v("Acoustic", 1), v("Ambient", 1)],
      "a",
    );
    expect(out).toEqual(["Ambient", "Acoustic", "Alternative"]);
  });

  it("ignores popularity entirely", () => {
    // Counts still arrive from the DB; they must not decide the order (owner
    // ruling 2026-09-26). Shorter, earlier inclusion wins over 99 uses.
    const out = rankSuggestions(
      [v("Classic Rock", 99), v("Punk Rock", 1)],
      "rock",
    );
    expect(out).toEqual(["Punk Rock", "Classic Rock"]);
  });

  it("prefers the earliest inclusion when lengths do not decide", () => {
    const out = rankSuggestions(
      [v("Classic Rock", 50), v("A Rock Anthem", 1)],
      "rock",
    );
    expect(out).toEqual(["A Rock Anthem", "Classic Rock"]);
  });

  it("lets the prefer list win its class outright", () => {
    // …and then closeness orders the rest: Blackfield (10) before Black
    // Sabbath (12), despite Sabbath being the most used of the three.
    const out = rankSuggestions(
      [v("Blackwater Park", 1), v("Black Sabbath", 40), v("Blackfield", 3)],
      "bl",
      { prefer: ["Blackwater Park"] },
    );
    expect(out).toEqual(["Blackwater Park", "Blackfield", "Black Sabbath"]);
  });

  it("leads with the exact match, however popular the alternatives are", () => {
    // Owner report 2026-09-26: typing DAWN offered "Dawn of Victory" first
    // (more tracks → more popular), so Tab/Enter would have replaced a correct
    // value with a different album.
    const out = rankSuggestions(
      [v("Dawn of Victory", 120), v("DAWN", 1), v("A New Dawn Ending", 5)],
      "dawn",
    );
    expect(out).toEqual(["DAWN", "Dawn of Victory", "A New Dawn Ending"]);
  });

  it("lets an exact match beat a prefer entry too", () => {
    const out = rankSuggestions(
      [v("Blackwater Park", 1), v("Black", 30)],
      "black",
      { prefer: ["Blackwater Park"] },
    );
    expect(out).toEqual(["Black", "Blackwater Park"]);
  });

  it("matches through diacritics and case", () => {
    const out = rankSuggestions([v("Motörhead"), v("Motorhead")], "MOTOR");
    expect(out).toEqual(["Motorhead", "Motörhead"]);
  });

  it("does not fuzz: only prefixes and substrings match", () => {
    expect(rankSuggestions([v("Meliora")], "mlr")).toEqual([]);
  });

  it("returns nothing for a blank query", () => {
    expect(rankSuggestions([v("Rock")], "   ")).toEqual([]);
  });

  it("caps at the limit and drops case-folded duplicates", () => {
    const out = rankSuggestions(
      [v("Rock"), v("rock"), v("Rockabilly"), v("Rocksteady")],
      "rock",
      { limit: 2 },
    );
    expect(out).toEqual(["Rock", "Rockabilly"]);
  });

  it("keeps the DB's spelling of a case-folded pair", () => {
    // The vocabulary command already folded these; the list must not show two.
    expect(rankSuggestions([v("Rock"), v("ROCK")], "roc")).toHaveLength(1);
  });
});

describe("shouldFlipAbove", () => {
  it("keeps the list below when there is room for it", () => {
    expect(
      shouldFlipAbove({ inputTop: 200, inputBottom: 230, panelBottom: 700, count: 8 }),
    ).toBe(false);
  });

  it("flips above a low field whose list would run past the panel", () => {
    expect(
      shouldFlipAbove({ inputTop: 600, inputBottom: 630, panelBottom: 700, count: 8 }),
    ).toBe(true);
  });

  it("counts the list's own height, not just the field's position", () => {
    const args = { inputTop: 400, inputBottom: 430, panelBottom: 700 };
    expect(shouldFlipAbove({ ...args, count: 1 })).toBe(false);
    expect(shouldFlipAbove({ ...args, count: 8 })).toBe(true);
  });

  it("stays below when there is no room above either (it scrolls)", () => {
    expect(
      shouldFlipAbove({ inputTop: 20, inputBottom: 50, panelBottom: 100, count: 8 }),
    ).toBe(false);
  });
});

describe("highlightIndex", () => {
  it("arms the first suggestion", () => {
    expect(highlightIndex(["Rock", "Rockabilly"])).toBe(0);
  });

  it("arms the exact match too — the stored spelling is the point", () => {
    // Owner case: type `dawn`, the library holds `DAWN`; refusing to highlight
    // it left no keyboard way to take the decorated spelling.
    expect(highlightIndex(["DAWN", "Dawn of Victory"])).toBe(0);
  });

  it("arms nothing when there is nothing to offer", () => {
    expect(highlightIndex([])).toBe(-1);
  });
});