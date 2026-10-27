# Tooling gotchas (Ponytail, project-local Pi packages)

Moved out of `AGENTS.md` on 2026-10-27. Rules stay in `AGENTS.md`; this file
carries the pin and the reason.

- **Ponytail is installed into this repo only, at a fixed commit.** It lives in
  `.pi/` as a Pi project package, not in the global arsenal, because the
  2026-10-21 ruling scoped it to Songstress: the lazy-senior-dev ruleset governs
  code the crew writes here, and nowhere else.
  - package: `git:github.com/DietrichGebert/ponytail`
  - pinned: **v5.3.2, commit `09c434a`** (2026-10-18)
  - declared in `.pi/settings.json`, which is tracked so a fresh clone knows what
    to install: `pi install -l git:github.com/DietrichGebert/ponytail`
  - the clone itself (`.pi/git/`, ~25 MB with its own `.git`) is **ignored**, so
    nobody commits a vendored checkout by accident.
  - why the pin matters: an unpinned `git:` package reinstalls whatever `main` is
    that day, so the ruleset can change under a session with no diff to review.
    To move it, bump the commit here in the same change.
