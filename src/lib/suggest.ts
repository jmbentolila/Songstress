/**
 * As-you-type matching for the tag editors' suggestion lists (owner ask,
 * 2026-09-26). Pure, so the ranking can be tested without a DOM — the popover
 * is the part that needs eyes, the ORDER is the part that silently rots.
 *
 * The shape of it, and why:
 *
 *  - A folded PREFIX beats a folded SUBSTRING, and nothing else counts as a
 *    match. No subsequence fuzz: a field that offers "Meliora" for "mlr" is
 *    guessing about data the user owns.
 *  - An EXACT folded match leads its class before anything else: typing a value
 *    the library already holds must offer that value first, so the stored
 *    spelling (case, decoration and all) can be taken with Tab/Enter instead of
 *    being retyped. See `highlightIndex`: the first row is always armed, which
 *    is what makes that completion automatic.
 *  - Within a class, a `prefer` list wins (the album door puts the same album
 *    artist's titles first, exactly as the datalist did). Then CLOSENESS:
 *    earliest match position, then the shortest value — the completion nearest
 *    what was typed. Popularity is deliberately NOT part of the order (owner
 *    ruling 2026-09-26: "it's not about popularity"), even though the counts
 *    still arrive from the DB: ranking the list by how much of the library a
 *    value happens to name makes the top suggestion depend on library size
 *    instead of on what the user typed. Folding it out also makes the order
 *    reproducible down to the last tie.
 *  - Folding is case + diacritics only. `sort.ts`'s leading-article rule is
 *    deliberately NOT used here: it would fold "The Beatles" to "beatles", and
 *    typing "the" would find nothing, which is the opposite of helpful.
 */

export interface SuggestionValue {
  value: string;
  count?: number;
}

/** Lowercase + diacritic-stripped + trimmed. */
export function fold(s: string): string {
  return s
    .trim()
    .toLowerCase()
    .normalize("NFD")
    .replace(/\p{M}/gu, "");
}

/** Case-only folding: the identity the vocabulary command itself merges on
 *  (it sums "rock" + "Rock" into one value). Diacritics deliberately survive
 *  it — "Motörhead" and "Motorhead" are two spellings the user may hold on
 *  purpose, so they are two suggestions; only their MATCHING is folded. */
function caseKey(s: string): string {
  return s.trim().toLowerCase();
}

export interface RankOptions {
  /** How many values the popover will render (the UI cap, not a query cap). */
  limit?: number;
  /** Values that win their match class outright, in the order given. */
  prefer?: readonly string[];
}

/** 0 = prefix, 1 = substring, -1 = not a match. */
function matchClass(foldedValue: string, foldedQuery: string): number {
  if (foldedValue.startsWith(foldedQuery)) return 0;
  if (foldedValue.includes(foldedQuery)) return 1;
  return -1;
}

/**
 * Ranked suggestion values for `query`, longest-unseen last. A blank query has
 * no suggestions — a list that opens on focus is a menu, not a completion.
 */
export function rankSuggestions(
  values: readonly SuggestionValue[],
  query: string,
  opts: RankOptions = {},
): string[] {
  const q = fold(query);
  if (q === "") return [];
  const limit = opts.limit ?? 8;
  const preferIndex = new Map<string, number>();
  for (const [i, v] of (opts.prefer ?? []).entries()) {
    const key = caseKey(v);
    if (!preferIndex.has(key)) preferIndex.set(key, i);
  }

  type Row = {
    value: string;
    cls: number;
    /** The value IS what was typed (folded): the identity case, which must lead
     *  its class regardless of how the library uses it. Owner report: typing
     *  `DAWN` offered `Dawn of Victory` first (more tracks → more popular),
     *  so Tab/Enter would have replaced a correct value with a different one. */
    exact: number;
    pref: number;
    /** Where the query matched, and how long the value is: together these are
     *  "closest completion" (earliest inclusion, then fewest extra letters). */
    index: number;
    len: number;
    folded: string;
  };
  const rows: Row[] = [];
  const seen = new Set<string>();
  for (const v of values) {
    const key = caseKey(v.value);
    if (key === "" || seen.has(key)) continue;
    const folded = fold(v.value);
    const cls = matchClass(folded, q);
    if (cls < 0) continue;
    seen.add(key);
    rows.push({
      value: v.value,
      cls,
      exact: folded === q ? 0 : 1,
      pref: preferIndex.get(key) ?? Number.MAX_SAFE_INTEGER,
      index: folded.indexOf(q),
      len: folded.length,
      folded,
    });
  }

  rows.sort(
    (a, b) =>
      a.cls - b.cls ||
      a.exact - b.exact ||
      a.pref - b.pref ||
      a.index - b.index ||
      a.len - b.len ||
      (a.folded < b.folded ? -1 : a.folded > b.folded ? 1 : 0) ||
      // Last resort: the raw value, so two spellings that fold alike
      // ("Motörhead" vs "Motorhead") never depend on input order.
      (a.value < b.value ? -1 : a.value > b.value ? 1 : 0),
  );
  return rows.slice(0, limit).map((r) => r.value);
}

/**
 * Would a list of `count` suggestions hang past the panel's visible bottom?
 * Then it should open ABOVE its field instead: the modal body both scrolls and
 * clips, so a list below a low field would be cut off rather than overlapped.
 * The estimated height is the popover's own budget (surface padding + ~38px
 * rows, capped at the list's max-height); a second measuring pass would paint
 * the list in the wrong place for a frame first, which is worse than an
 * estimate that only decides a flip.
 *
 * When there is no room above either, the list stays below: it is capped and
 * scrolls internally, so the bottom of it is reachable by scrolling the panel.
 */
export function shouldFlipAbove(args: {
  inputTop: number;
  inputBottom: number;
  panelBottom: number;
  count: number;
}): boolean {
  const estimated = Math.min(args.count * 38 + 12, 320);
  return (
    args.inputBottom + 4 + estimated > args.panelBottom - 8 &&
    args.inputTop - estimated > 0
  );
}

/**
 * Which item starts highlighted: always the first one, when there is one.
 *
 * The earlier rule ("highlight nothing when the top suggestion merely repeats
 * the query") was wrong for a DB-backed list — the owner's case: type `dawn`,
 * the library holds `DAWN`, and refusing to highlight it left no way to take
 * the stored spelling by keyboard. The list is the library's answer to what was
 * typed, so the first row is always armed; ranking already puts an exact match
 * first, which is exactly the spelling that gets completed (Tab/Enter) rather
 * than retyped. A second Tab then moves on as usual.
 */
export function highlightIndex(items: readonly string[]): number {
  return items.length === 0 ? -1 : 0;
}