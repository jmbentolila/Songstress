import { describe, expect, it } from "vitest";
import { compareByName, sortKey } from "./sort";

describe("sortKey", () => {
  it("lowercases and strips leading articles", () => {
    expect(sortKey("The Beatles")).toBe("beatles");
    expect(sortKey("THE  WHO")).toBe("who");
  });

  it("folds diacritics", () => {
    expect(sortKey("Motörhead")).toBe("motorhead");
    expect(sortKey("Sigur Rós")).toBe("sigur ros");
  });
});

describe("compareByName", () => {
  it("sorts ignoring case, articles and diacritics", () => {
    const input = ["the Who", "Motörhead", "Ayreon", "The Beatles"];
    expect([...input].sort(compareByName)).toEqual([
      "Ayreon",
      "The Beatles",
      "Motörhead",
      "the Who",
    ]);
  });
});
