import { describe, expect, it } from "vitest";
import { discSplitPlan } from "./discSplit";

// Each case is a real album in the owner's library (2026-09-06 census),
// with the shape the split contract promises in the expanded panel.
describe("discSplitPlan", () => {
  it("renders nothing split when no disc reaches the multi-disc gate", () => {
    expect(discSplitPlan([6, 5])).toEqual([null, null]);
    expect(discSplitPlan([4, 5, 6])).toEqual([null, null, null]);
  });

  it("stacked short discs split — no cover anchor below the first block (01011001 8/7, Twilight Dementia 7/6)", () => {
    expect(discSplitPlan([8, 7])).toEqual([4, 4]); // 4+4 · 4+3
    expect(discSplitPlan([7, 6])).toEqual([4, 4]); // 4+3 · 4+2
  });

  it("followers join disc 1's lead instead of sitting single (Ira Dei 10/8)", () => {
    expect(discSplitPlan([10, 8])).toEqual([5, 5]); // 5+5 · 5+3
  });

  it("all followers align on the lead (Human. :II: Nature. 9/8/9)", () => {
    expect(discSplitPlan([9, 8, 9])).toEqual([5, 5, 5]); // 5+4 · 5+3 · 5+4
  });

  it("disc 1 halves at the gate and gives disc 2 a clean lead (Electric Castle 7/10)", () => {
    expect(discSplitPlan([7, 10])).toEqual([4, 5]); // 4+3 · 5+5
  });

  it("a much longer follower self-balances past the lead minimum (Legends 11/23)", () => {
    expect(discSplitPlan([11, 23])).toEqual([6, 12]); // 6+5 · 12+11
  });

  it("a long follower at or below the lead halves itself, no 12-row stump (Legacy of the Dark Lands 24/12/12/20)", () => {
    expect(discSplitPlan([24, 12, 12, 20])).toEqual([12, 6, 6, 12]); // 12+12 · 6+6 · 6+6 · 12+8
  });

  it("a short tail disc halves at the multi-disc gate (RotK 15/14/16/8)", () => {
    expect(discSplitPlan([15, 14, 16, 8])).toEqual([8, 8, 8, 4]);
  });

  it("disc 1 needs no exception: the general clause gives a tiny disc 1 its miniature half ([2,9] hypothetical)", () => {
    expect(discSplitPlan([2, 9])).toEqual([1, 5]); // 1+1 · 5+4
  });

  it("interlude discs below the gate stay single (Omega 12/4/12/12)", () => {
    expect(discSplitPlan([12, 4, 12, 12])).toEqual([6, null, 6, 6]);
  });

  it("keeps the balanced shape where it already held (Once 11/11/14)", () => {
    expect(discSplitPlan([11, 11, 14])).toEqual([6, 6, 7]);
  });

  it("never leaves a lone item in column 2 on library data", () => {
    const library: number[][] = [
      [11, 23], [11, 11, 14], [7, 10], [15, 17], [18, 19], [11, 12],
      [15, 14, 16, 8], [14, 15], [12, 13], [13, 14], [10, 8], [9, 8, 9],
      [8, 7], [7, 6], [19, 8], [24, 12, 12, 20], [12, 4, 12, 12],
    ];
    for (const counts of library) {
      const plan = discSplitPlan(counts);
      counts.forEach((n, i) => {
        if (plan[i] !== null) expect(n - plan[i]!).toBeGreaterThanOrEqual(2);
      });
    }
  });
});
