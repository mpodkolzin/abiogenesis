# Learning Journal

Notes are short — one entry per exercise. Edit/replace my drafts freely; subjective parts ("surprised", "clicked") are best in your own voice.

---

## Week 1 — Mass component & avg-mass logger
**Date:** 2026-05-14
**Branch:** `learning/week-1-mass`

**What changed.** Added `Mass(f32)` random-initialized at every spawn site in `spawner.rs` (respawn bundle + recycle insert + new-spawn bundle), and a `log_avg_mass` system on `PostStartup` in `particle.rs`. Updated `learning-plan.md` Week 1 to reflect the actual lessons.

**What surprised me.** *(your edit)* — candidate: that a `Startup` system scheduled "after" the spawn trigger still sees zero particles, because both the trigger and the spawn it produces are queued in `Commands` and only flush *between* schedules. `PostStartup` is the earliest visible point.

**What clicked.** *(your edit)* — candidate: `#[require(T)]` isn't a constraint, it's a fallback. The spawn-site bundle wins; require only fires when `T` is missing. Same shape as a default parameter you can override per call.

**Plan deviation.** The original plan assumed `Mass` didn't exist and suggested a `Default = 1.0`. In practice `Mass` was already drafted in the working tree, and using a hand-rolled `Default` is less natural than initializing at the spawn site. Plan rewritten to match.
