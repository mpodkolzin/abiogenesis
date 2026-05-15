# CLAUDE.md — Rust + Bevy Learning Project

## What this repo is right now

A Rust + Bevy learning project. `abiogenesis/` is a working particle-life simulator (Bevy 0.16) used as a **seed** for hands-on learning. The structured plan is in `learning/learning-plan.md`.

Two phases:

1. **Modify abiogenesis** — exercises land as small changes on this codebase, branch per exercise (`learning/week-1-mass`, etc.).
2. **Build our own** — once the plan is done, start a fresh Bevy project using this repo as a **template**: lift patterns, plugins, idioms.

When this repo stops being a learning project (or phase 2 starts in earnest), rewrite this file.

---

## How we work together

### Three modes: Design, Build, Reflect

We are always in exactly one mode. **Default is Design.**

**Design mode** (default at the start of every new task)
- Brainstorm the approach before any source edits.
- **Read the current source first.** The plan and the code drift — surface that divergence during Design rather than discovering it mid-Build. (Already happened in Week 1: `Mass` was already defined and in `#[require]`.)
- Sketch the shape: which Bevy components/resources/events/systems, which set ordering, which queries — at the type/signature level, not the implementation.
- Explain the Rust and Bevy concepts in play (see *Explaining*, below).
- Pseudocode or illustrative code snippets in chat are fine. **Do not edit source files in Design mode.**
- End with: "Design ready — say *build* to implement."

**Build mode**
- Entered only on an explicit user signal: "build", "go", "implement", "do it", or similar.
- Implement the agreed design. Stay inside its scope.
- For multi-step exercises, use `TaskCreate` to track sub-tasks; mark each `completed` as it lands.
- If implementing reveals a flaw in the design, **stop, switch back to Design**, surface the issue. Don't silently redesign mid-build.
- After the change compiles and runs, switch to Reflect.

**Reflect mode**
- Append an entry to `learning/journal.md` covering:
  - **What changed** — one sentence, files/components touched.
  - **What surprised me** — a Rust or Bevy concept that didn't behave as expected.
  - **What clicked** — concept that now feels natural.
  - **Plan deviations** — if the exercise diverged from `learning-plan.md`, note why. If the deviation generalizes, propose a plan edit.
- Suggest a commit on the exercise branch (don't auto-commit).
- Drop back to Design for the next task.

If unsure which mode we're in, ask.

### Explaining Rust and Bevy

Every meaningful change touches both. When a concept enters the design or the code, explain it briefly:

- **Rust note:** ownership/borrow rules, lifetimes, traits, idioms. Lean on C++/C# analogies — Max has 10+ years of both and is newer to Rust.
- **Bevy note:** ECS patterns — queries, system params, schedule sets, observers vs events, component hooks, commands, resource vs component, plugin layout.

**Always disambiguate Bevy framework code from abiogenesis project code** in explanations. Don't blur "Bevy does X" with "this repo does X." (See `feedback_explanation_docs` in auto-memory.)

Skip an explanation once it's been covered earlier in the session.

### Design before code — no exceptions for small edits

Even one-line changes go through Design first. The ceremony isn't the point; the point is that *designing forces the Rust/Bevy concepts to the surface where they can be learned*. A silent edit teaches nothing.

---

## Workflow defaults

- **Branch per exercise** (e.g., `learning/week-3-metrics-plugin`).
- **Don't auto-commit.** Ask when work feels done.
- **Don't open PRs** unless asked.
- **Don't expand scope.** Surface tangential issues; don't silently fix them.
- Keep prose tight. Learning is in the doing.

---

## Pointers

- **Learning plan & weekly exercises:** `learning/learning-plan.md`
- **Bevy mental-model primer:** `learning/bevy-101.md` (read before Week 1)
- **Repo architecture, key files, stubs:** see auto-memory `MEMORY.md` — already loaded.
- **Bevy version:** pinned to **0.16**. APIs churned heavily pre-1.0 — verify any web reference against this version.
- **Toolchain:** nightly Rust required (`gen_blocks`, `coroutines`, `trait_alias`, `iter_collect_into`).
