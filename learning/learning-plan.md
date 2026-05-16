# Bevy Learning Plan — abiogenesis edition

**Profile:** ~2 hrs/week × 8 weeks · 10+ yrs C++/C#/Java · new-ish to Rust · hands-on by modifying this repo + light theory.

**Format per week:** ~30 min theory · ~80 min hands-on exercise · ~10 min reflect/commit. Each exercise is a real change to this codebase on a feature branch, designed to fit the time budget.

**Throughline:** treat each week as a small PR. Branch from `main`, commit when done, open a self-PR (or just `git diff main`). The point is the doing, not the merging.

---

## Mental-model primer (read before Week 1)

Coming from OOP, two reframes do most of the work:

1. **No objects.** An `Entity` is just an integer ID. There's no `Particle` *object* with methods — there's a row in a table, with columns like `Transform`, `Velocity`, `ParticleColour`. Behavior lives in free functions ("systems") that query the table.
2. **No `this`.** Where you'd write `particle.update(dt)`, Bevy writes `fn update_particles(mut q: Query<&mut Particle>, time: Res<Time>)`. The system declares its data dependencies in its parameter list; the framework figures out parallelism and ordering.

If you internalize that, everything else is API surface.

C++ comparison that helps: think `entt` (the C++ ECS lib) for the data layout, plus a job system that schedules systems based on read/write declarations. You already know why you'd want disjoint reads in parallel — Bevy proves disjointness via Rust's borrow checker.

---

## Week 1 — ECS mental model + read tour

**Concepts:** Entity / Component / System / Resource / Plugin. Why ECS over OOP for simulation-heavy code. Composition by component vs inheritance.

**Read (~30 min):**
- `learning/bevy-101.md` (the primer you saved)
- `abiogenesis/src/main.rs` end to end
- `abiogenesis/src/particles.rs` (just the plugin bundler)
- `abiogenesis/src/particles/particle.rs` end to end

**Exercise (~80 min):** Give particles a `Mass` and log the population average.

If `Mass` already exists in `particles/particle.rs` (it may, as a stub), keep the definition and proceed. Otherwise:
1. Define `#[derive(Debug, Reflect, Component, Default, Clone, Copy, Deref, DerefMut)] struct Mass(pub f32);`.
2. Add it to `#[require(...)]` on `Particle`. The derived `Default` gives `Mass(0.0)`; that's fine because we override at the spawn site.

Then:

3. **Initialize mass at spawn time, not via `Default`.** In `spawner.rs`, include `Mass(0.5 + rand::random::<f32>() * 2.0)` in:
   - the `respawn_particles` bundle,
   - the new-particle branch of `spawn_particle`,
   - the recycle branch's `insert(...)` tuple (so recycled entities get fresh mass too).
4. Add a `log_avg_mass` system that queries `&Mass` filtered by `With<Particle>`, sums, divides, `info!`s the average. Register it in `ParticlePlugin::build` under **`PostStartup`** (not `Startup` — see below).
5. Don't wire mass into physics yet — Week 2.

**Rust note (OOP→Rust):** `#[derive(Component)]` is a marker, not an interface. There's no v-table. The "polymorphism" is the schedule deciding which systems run on which entities based on which components they have.

**Rust note (Default):** `#[derive(Default)]` on a tuple struct uses each field's `Default`, so `Mass(pub f32)` defaults to `Mass(0.0)`. No attribute-driven default value like C#'s `[DefaultValue(1.0)]` — you'd write `impl Default` by hand. Bundle-site override is usually cleaner than custom defaults.

**Bevy note (`#[require]` vs bundle):** `#[require(T)]` inserts `T::default()` if `T` is missing from the spawn bundle. Passing `T(x)` explicitly in the bundle overrides the default. The require list is the *fallback*, not a constraint.

**Bevy note (Startup vs PostStartup):** Spawning here is doubly deferred — `Startup` triggers a `Respawn` observer, which calls `commands.spawn(...)`. Both are queued via `Commands` and only land in the world at the next command flush. Bevy flushes commands **between** `Startup` → `PostStartup`, so a `PostStartup` system is the earliest place the new particles are queryable. A logger added to `Startup` after spawn would see zero particles.

**Done when:** `cargo run` boots, the log shows `avg mass over 3000 particles: ~1.5`, no warnings.

---

## Week 2 — Queries, filters, change detection

**Concepts:** `Query<T, F>`, filters (`With`, `Without`, `Changed`, `Added`, `Or`), `Res::is_changed()`, `Local<T>` for per-system state, `Time::delta_secs()`.

**Read (~30 min):**
- `abiogenesis/src/particles/simulation.rs:67-137` (the big query in `compute_forces`)
- `abiogenesis/src/particles/spawner.rs:244-268` (`update_colours_on_num_change` — Local + change detection)

**Exercise (~80 min):** Build a histogram logger.
1. New system `log_color_histogram` that counts particles per `ParticleColour` and logs every 5 seconds.
2. Use `Local<Timer>` for the cadence (initialize via `Local::set` pattern or a `FromWorld` impl, or just `*timer = Timer::from_seconds(5.0, TimerMode::Repeating)` on first tick).
3. Register it in `SimulationPlugin` under `AppSystems::Update`.
4. Add a second system that only runs when `SimulationParams` changed, logging the new num_colours.

**Rust note:** `Query` borrows the world for the duration of the system call only — you can't store a query reference in a struct field. If you want persistent state per system, that's `Local<T>`.

**Done when:** Histogram logs every 5s; param-change log fires only on actual changes (test with the egui inspector).

---

## Week 3 — Resources, plugins, schedule, system sets

**Concepts:** Resource vs Component (singleton vs per-entity), `App::insert_resource`, plugin layout, `Startup` / `Update` / `FixedUpdate` / `PostUpdate`, `configure_sets`, `.chain()`, `.before(X)`, `.after(X)`.

**Read (~30 min):**
- `abiogenesis/src/main.rs:95-105` (set ordering)
- `abiogenesis/src/systems.rs` (the `AppSystems` enum — small file)
- `abiogenesis/src/particles/simulation.rs:19-25` (how a system opts into a set)

**Exercise (~80 min):** Build a `MetricsPlugin`.
1. New file `abiogenesis/src/metrics.rs`. Define `MetricsPlugin`, a `SimulationMetrics` resource with `spawns: u64`, `despawns: u64`, `avg_speed: f32`, `last_updated: f64`.
2. Add a system that updates `avg_speed` each frame from the `Velocity` query.
3. Add a new variant to `AppSystems` called `Metrics`, schedule it after `AppSystems::Update`.
4. Wire `MetricsPlugin` into `app_systems()` in `main.rs`.
5. Spawn/despawn counts come in Week 4 (events).

**Rust note:** Any `'static + Send + Sync` type can be a `Resource` — just `#[derive(Resource)]`. There's no DI container; resource access is statically typed via `Res<T>`.

**Done when:** Inspector (egui) shows the metrics resource updating live.

---

## Week 4 — Entity lifecycle, hooks, Commands

**Concepts:** `Commands` (deferred mutation), `commands.spawn(bundle)`, `commands.entity(e).despawn()`, `#[require(...)]`, component hooks (`on_add`, `on_remove`, `on_insert`), why hooks run on the `DeferredWorld`.

**Read (~30 min):**
- `abiogenesis/src/particles/particle.rs:18-51` (hooks keep `ParticleIndex` in sync)
- `abiogenesis/src/particles/spawner.rs` end-to-end (spawn, despawn, recycle)

**Exercise (~80 min):** Particle ghost effect.
1. New `Lifetime(Timer)` component.
2. New system `tick_lifetime` that ticks all `Lifetime` components and despawns when finished.
3. Modify `spawn_particle` (or add an observer on the recycle path) so when a particle is recycled, a "ghost" entity is spawned at the old position: same mesh, dim color, `Lifetime(Timer::from_seconds(0.5, Once))`.
4. Hook your `MetricsPlugin` into spawn/despawn counts via component hooks on `Particle` (mirror what `ParticleIndex` does).

**Rust note:** `Commands` is a queue. Mutations land between systems, not inside them — that's how Bevy keeps parallel systems sound. If you need to read a value you just inserted in the same frame, you usually need `commands.run_system` or a separate system in a later set.

**Done when:** Recycled particles leave a fading trail; spawn/despawn counters in the inspector match reality.

---

## Week 5 — Events vs Observers

**Concepts:** Buffered events (`EventReader`, `EventWriter`, double-buffer with 2-frame retention), observers (`Trigger<T>`, `add_observer`, `commands.trigger(...)`), when to choose which.

**Read (~30 min):**
- `abiogenesis/src/particles/spawner.rs:14, 211` (`SpawnParticle` event flow)
- `abiogenesis/src/particles/spawner.rs:20, 135` (`Respawn` observer)
- `abiogenesis/src/controls.rs` (observers attached to `UIRoot`)

**Exercise (~80 min):** Collision events.
1. Add a `ParticleCollision { a: Entity, b: Entity, position: Vec2 }` event via `app.add_event::<...>()`.
2. In `compute_forces`, when two particles end up within `repulsion_radius / 4`, fire the event. (Watch out for double-counting — only fire when `a.index() < b.index()`.)
3. Add a system that drains collisions and bumps a `MetricsPlugin` collision counter.
4. Add an observer-based `ResetMetrics` trigger, fired from a new keybind (e.g., `R` while shift-held). Wire it through `ControlsPlugin`.

**Rust note:** Buffered events live for 2 frames — if no reader runs in that window, they're dropped. Observers fire synchronously when commands flush. Rule of thumb: many-to-many = events; one-shot UI/lifecycle = observers.

**Done when:** Collision counter visibly climbs; `R+shift` zeros all metrics instantly.

---

## Week 6 — Rendering, assets, 2D pipeline

**Concepts:** `Assets<T>`, `Handle<T>` (refcounted, like `Arc`), `Mesh2d` + `MeshMaterial2d`, the main world / render world boundary (high level — don't go deep), `Camera2d`.

**Read (~30 min):**
- `abiogenesis/src/particles/spawner.rs:56-78` (`init_assets`)
- `abiogenesis/src/camera.rs` end-to-end
- Bevy book chapter on 2D rendering (skim, ~10 min)

**Exercise (~80 min):** Particle trails.
1. Pick a "focused" particle (e.g., the first one in `ParticleIndex`).
2. Every N frames (use `Local<u32>` counter), spawn a small mesh entity at its current position.
3. Give it a `Lifetime(Timer)` and a faded variant of its color.
4. Reuse the existing handles from `ParticleAssets` — don't allocate new meshes per trail point.
5. Bonus: tween its scale/alpha down using `bevy_tweening` (already in deps).

**Rust note:** `Handle<T>` clones are cheap (refcount bump). Storing a handle in 1000 components is fine — they all point to one underlying mesh.

**Done when:** Visible trail behind one particle; FPS doesn't tank.

---

## Week 7 — Input, UI, picking

**Concepts:** `ButtonInput<KeyCode>`, `ButtonInput<MouseButton>`, `MouseMotion`, `MouseWheel`, the Bevy UI tree (it's an entity tree with `Node` components), `bevy_picking` for click targets.

**Read (~30 min):**
- `abiogenesis/src/controls.rs` end-to-end
- `abiogenesis/src/ui.rs` (focus on `respawn_ui`)

**Exercise (~80 min):** Debug overlay.
1. Press `D` to toggle a `DebugOverlay` resource.
2. When on, render a UI panel (top-left) showing: FPS, particle count, collision count, avg speed.
3. Use `FrameTimeDiagnosticsPlugin` for FPS (free from Bevy).
4. Bonus: draw the spatial-hash grid as a wireframe overlay (use `Gizmos` system param).

**Rust note:** `ButtonInput::pressed` is "currently held"; `just_pressed` is "transitioned this frame." A common bug is using `pressed` for toggles — use `just_pressed`.

**Done when:** `D` toggles a panel that updates live; numbers match what's in the inspector.

---

## Week 8 — Performance, parallelism, fixed timestep

**Concepts:** `par_iter_mut` (and why it's safe), `FixedUpdate` schedule + decoupling sim rate from frame rate, profiling via `LogDiagnosticsPlugin`, recognizing the spatial-hash sweet spot.

**Read (~30 min):**
- `abiogenesis/src/particles/simulation.rs:88-134` (par_iter, query reuse via `spatial_index`)
- `abiogenesis/src/spatial_hash.rs` (skim — it's the data structure under the hood)

**Exercise (~80 min):** Decouple sim from render + measure.
1. Move `compute_forces` from `Update` to `FixedUpdate`. Pick a fixed step (e.g., 60Hz).
2. Add `FrameTimeDiagnosticsPlugin` and `LogDiagnosticsPlugin` so FPS + frame time print to console every second.
3. Bump `MAX_PARTICLES` to 5000 and 10000. Note where it breaks down.
4. Pick one bottleneck and write a paragraph (in `learning/week8-notes.md`) about what's expensive: spatial index rebuild? Force sum? Render submission?

**Rust note:** `par_iter_mut` works because Bevy proves at compile time that no two entities alias each other through the query — that's a guarantee the C++ entt equivalent can't statically give you.

**Done when:** Sim runs at fixed rate, render decouples (you can resize window without sim slowing); notes file written.

---

## After week 8

You'll have:
- Touched every major Bevy subsystem in this repo.
- A `MetricsPlugin`, debug overlay, ghost effect, trails, and collision events you wrote.
- A real feel for ECS, the schedule, and parallel queries.

**Natural next steps** (pick one based on goal):
- **Master this codebase:** Tackle a stub from `MEMORY.md` — implement `scenes.rs` (saved scenarios) or wire native import/export properly.
- **Ship your own game:** Start a fresh Bevy project (Pong, then Asteroids). Reuse the patterns you internalized here.
- **Go deeper:** Read the Bevy source for one subsystem you're curious about — `bevy_ecs` schedule code is famously readable.

---

## Tracking

Make a branch per week (`learning/week-1-mass`, etc.). At the end of each week, jot one paragraph in `learning/journal.md` answering: what surprised you, what felt awkward, what clicked.

Bevy version pinned: 0.16. If something looks different on the web, check that first — the API has churned a lot pre-1.0.
