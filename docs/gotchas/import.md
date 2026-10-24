# Import staging

The full staging and ownership model. The data-loss rules are repeated inline in `AGENTS.md`; this is the whole contract.

Moved out of `AGENTS.md` on 2026-10-24 to keep the injected file under
the 20,000-char context cap. Rules stay in `AGENTS.md`; this file carries the
measurements and the reproduction.

- **Staging is a row flag, not a folder.** `tracks.staged` (migration v2) is the entire definition of "pending": `import_music` indexes the
  user's files **where they are** — no copy, no cache dir, no second scan root — so a pending import gets artwork, waveforms, play counts and
  scan-safety the moment it lands, and importing a 400-file folder costs a scan instead of 400 writes. The scan flags exactly the files that
  were requested (`run_scan_files(.., only: Some(&set), ..)`), which is why importing one file out of a 400-file folder stages one.
- The flow: import flags → the user decides per album → `save_imports` **moves** the file into its resolved folder and re-points the row
  (`scan::relink_track`, keeping play history; id derives from path, so id and path change together or the next re-import hits UNIQUE),
  `discard_imports` **forgets the row and deletes nothing**. `album_destination` resolves the folder: the album's own folder → the artist's
  layout → the directory the majority of the library's albums live in → the primary root.
- **Three rules, all about ownership.** At Apply, a file whose resolved name the destination folder already holds **byte for byte** is a
  duplicate: **the file the user just pointed at is deleted**, the library's copy stays, and the receipt says so — a silent deletion of
  someone's file is not acceptable even when it is a copy of one they own. (Same name, different bytes ⇒ a ` (2)` name, never an overwrite.
  This is a collision rule, not library-wide content dedupe: the same track on a compilation and on an album is legitimate, so a hash column
  would delete files the user wants twice over.) A file the library already indexes is **not staged at all** and is reported under
  "Already in your library". And the app **never removes a folder it did not write**: it prunes only directories under its own staging root,
  and the folder you imported from is left alone even when it is now empty.
- **Import reports what it did.** `import_music` returns `ImportReport { staged, already }`; `imports.report` renders it in the window the
  import opens, and `imports.applied` renders Save/Discard counts after Apply — including the deletion above. A progress ring that ends with
  the screen unchanged reads as a failure, and "turns out you already own this" is the most likely outcome of importing music.
- Legacy: rows whose path sits under the cache `import/` dir are flagged at startup, so a copy-era pile survives the upgrade and its next Save
  is a rename out of the app's own directory. The copy importer (`import_paths`, `KnownFiles`, name+size dedupe) is gone; identity is **path**
  (import), **blake3 hash** (save), and `(artist, album, disc, track)` (scan), each covered by a test.
