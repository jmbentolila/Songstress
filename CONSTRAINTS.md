# CONSTRAINTS.md — Songstress (STANDING, owner-approved 2026-10-14)

Quality contract for agent work in this repo. Measured baseline below is
the floor: hold it, never lower it. Interview source: owner intent
2026-10-14 (Ciel judges, owner hears only escalations).

## Floor (blocks the work)

- Gates green: `cargo check` clean with zero new warnings,
  `cargo test --lib` green, `npm test` green, `npm run check` 0 errors.
- No silenced checks: no new `#[allow]` (3 pre-existing),
  no `#[ignore]` (0 today), no deleted or skipped tests, no thresholds
  edited down.
- No stubs: every new function called, every new branch tested.
- One behavior change = one proving test, green in the same run.

## Task bar (coverage + security, owner-ordered)

- Coverage on new code: each fix carries a regression test exercising
  the exact flaw; revert-fails proof for security-shaped fixes.
- Security scanning: new or bumped deps get a known-vulnerability
  check (`cargo audit` counts); destructive paths resolve symlinks,
  stay under an allowlisted root, show ownership evidence first.
- Diffs stay small and sliced; no drive-by refactors.
- Version files untouched (package.json, tauri.conf.json, Cargo.toml)
  except on the owner's ship order.

## Supervision (Ciel judges)

- Reads run free. Destructive or out-of-tree writes route through Ciel
  first. Pi dispatches carry tool-scoping; worktree confinement for
  unapproved work.
- Every check failure pings Ciel first: warn or block per assessment,
  owner hears only what needs direct eyes.
- Watchdog per dispatch: probes ~10 min, rebuilds ~30 min, plus grace;
  one poll past the window, stuck gets a kill-or-continue call.

## Measured baseline 2026-10-14 (main f678766)

- cargo test --lib: 157 passed, 0 failed, 0 ignored.
- cargo check: zero warnings. `#[allow]`: 3 pre-existing. vitest:
  150/150. svelte-check: 0 errors.
