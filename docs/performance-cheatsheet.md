# Performance Cheatsheet

> Source: https://github.com/emilkowalski/skills/blob/main/performance-cheatsheet.md
> Saved as a local reference 2026-09-26. Upstream is a 7-row table; check back for updates.

| Problem                             | Solution                                                  |
| ----------------------------------- | --------------------------------------------------------- |
| Animation stutters                  | Animate `transform`/`opacity`, not `width`/`top`          |
| Long list scrolls slowly            | Virtualize — only render what's visible                   |
| Blur causes perf issues             | Keep animated `blur()` under 20px                         |
| Motion's `x`/`y` drops frames       | Animate the full `transform` string instead               |
| Random properties animate           | Don't do `transition: all`, list exact properties         |
| React re-renders every frame        | Write to `ref.current.style`, not state                   |
| Element shifts 1px as motion starts | `will-change: transform` (only once you see it)           |
