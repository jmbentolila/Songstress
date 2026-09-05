// Multi-disc tracklist split contract (owner rulings, 2026-09-06).
//
// One album, one visual grammar. Disc blocks stack vertically in the
// same panel, so the eye reads their column edges top-down; discs that
// split at different rows, or split while a neighbor of similar size
// sits single, read as unrelated lists glued together (the Ira Dei /
// Human. :II: Nature. / Legacy complaints, all converged into this):
//
//  1. The album splits at all only if some disc has ≥ MD_SPLIT_MIN
//     tracks. Otherwise every disc is one column: uniform by absence.
//  2. The LEAD is disc 1's own balance, ceil(n1/2). Disc 1 is on top
//     and dictates the edge expectation. (Disc 1 needs no special
//     treatment: n1 ≤ lead never holds for n1 ≥ 2, so rule 3's first
//     branch hands disc 1 exactly the lead. A disc 1 shorter than the
//     gate splits into a miniature half ([2, 9] → 1+1 · 5+4) — the
//     owner's call, and no album we own has a disc 1 below 7.
//  3. Every disc splits into two columns when it has ≥ MD_SPLIT_MIN
//     tracks OR has more tracks than the lead (a short disc longer
//     than a small lead still breaks the edge if it doesn't join:
//     Twilight Dementia d2, 6 > lead 4 → 4+2, owner-reviewed). Column 1
//     holds
//         n > lead ? max(lead, ceil(n/2)) : ceil(n/2)
//     The lead is a FLOOR, never a mandate and never an inflation:
//     Ira Dei d2 (8, lead 5) joins at 5+3 so the edge holds; Legacy
//     d2 (12, lead 12) keeps its own 6+6 rather than a 12-row stump —
//     the minimum there was the disc's own tracklist length, not disc
//     1's (owner ruling, same day as the rule); Legends d2 (23, lead
//     6) self-balances to 12+11 past the floor rather than a saw —
//     the library's only visible fallback.
//  4. A disc below MD_SPLIT_MIN and not longer than the lead stays
//     single: short interludes read as pauses, not failed splits
//     (Omega's two 4-track discs, the 2-track bonus discs).
//
// Why MD_SPLIT_MIN = 7 and not the single-disc SPLIT_MIN = 9: "short
// albums don't need the split — the cover square pins the panel
// height" (the owner's reason for 9) only anchors the FIRST thing in
// the panel; every disc block after it adds pure incremental height
// with a void to its right. So stacked discs split from 7
// (01011001 8/7 → 4+4 · 4+3, Twilight Dementia 7/6 → 4+3 · 4+2; on
// library data the smallest split the rule produces is 4+2).
//
// Traced against the whole library (64 multi-disc albums): no 1-item
// columns, no saws, max edge drift one row outside Legends' blessed
// fallback. Single-disc albums are NOT this file's business: they
// keep the 8-single / 7-cap / 11-balance band in ExpandedPanel.

/** A SINGLE-disc list splits at all only from this many tracks (the
 *  cover-square anchor applies — see ExpandedPanel for the band). */
export const SPLIT_MIN = 9;

/** A MULTI-DISC block splits from this many tracks — lower than the
 *  single-disc gate, because disc blocks after the first have no
 *  cover-square anchoring their height (see the contract above). */
export const MD_SPLIT_MIN = 7;

/**
 * Per-disc column-1 row counts for a multi-disc album, in disc order.
 * `null` = that disc block renders as a single column.
 */
export function discSplitPlan(trackCounts: number[]): (number | null)[] {
  if (!trackCounts.some((n) => n >= MD_SPLIT_MIN)) return trackCounts.map(() => null);
  const lead = Math.ceil(trackCounts[0] / 2);
  return trackCounts.map((n) => {
    const half = Math.ceil(n / 2);
    if (n > lead) return Math.max(lead, half);
    return n >= MD_SPLIT_MIN ? half : null;
  });
}
