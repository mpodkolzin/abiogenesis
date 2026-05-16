# Bevy 101 — through the lens of `abiogenesis`

> **Convention used in this doc:** every type/symbol is tagged on first mention as either **[Bevy]** (provided by the framework — `bevy::prelude::*` or a Bevy crate) or **[custom]** (defined in this project). Names that *sound* Bevy-ish but aren't get explicitly flagged. The whole point of this primer is to teach you where Bevy ends and your code begins.

Bevy is built around **ECS** (Entity-Component-System) plus a **plugin/scheduler** layer. Almost everything you'll see here is one of:

| Concept | What it is | This-repo example | Origin of the example |
|---|---|---|---|
| **Entity** | An ID. Just a number. | Each particle (up to 3000). | n/a — entities are framework-managed IDs |
| **Component** | Data attached to an entity. | `Transform` | **[Bevy]** |
| | | `Particle`, `Velocity`, `ParticleColour`, `Mass` | **[custom]** — note: `Particle` is **not** a Bevy primitive despite the name |
| **Resource** | Singleton data, world-global. | `Time`, `Assets<Mesh>` | **[Bevy]** |
| | | `SimulationParams`, `Model`, `ParticleIndex`, `ParticleAssets` | **[custom]** |
| **System** | A function that reads/writes the world. | `compute_forces`, `spawn_particle` | **[custom]** — Bevy provides the *plumbing*, never the systems |
| **Plugin** | A bundle of resources + systems. | `DefaultPlugins`, `TweeningPlugin` (3rd-party crate) | **[Bevy]** / 3rd-party |
| | | `ParticlePlugin`, `SimulationPlugin`, `UIPlugin` | **[custom]** |
| **Event / Observer** | Decoupled message passing. | `Respawn`, `SpawnParticle` | **[custom]** events; `Trigger<T>` and `EventReader<T>` are **[Bevy]** |
| **Schedule / Set** | When systems run. | `Startup`, `Update` | **[Bevy]** schedules |
| | | `AppSystems::TickTimers` etc. | **[custom]** sets |

> **Heads up — common confusion:** Bevy 0.16 has **no built-in particle system**. The `Particle` component in this repo is a custom unit-struct marker. For graphical particle effects, the Bevy ecosystem has the third-party `bevy_hanabi` crate, which this project does **not** use. Every "particle" here is a real ECS entity rendered with a `Mesh2d(Circle)` (both **[Bevy]**) plus custom physics.

---

## 1. The `App` is the root (`abiogenesis/src/main.rs:29`)

```rust
let mut app = App::new();             // App, World, Schedule are all [Bevy]
bevy_systems(&mut app);               // [custom helper] — registers DefaultPlugins
third_party_systems(&mut app);        // [custom helper] — registers TweeningPlugin etc.
app_systems(&mut app);                // [custom helper] — registers project plugins
app.run()
```

`App::new()` (**[Bevy]**) gives you a `World` (**[Bevy]** — the ECS data store) and a `Schedule` (**[Bevy]** — system runner). Everything else is registration.

`DefaultPlugins` (**[Bevy]**, used at `main.rs:42`) bundles windowing, rendering, input, assets, time. You configure it with `.set(...)` to override sub-plugins (e.g., window title). `AssetPlugin`, `WindowPlugin`, `Window`, `WindowResolution`, `ClearColor` — all **[Bevy]**.

## 2. Plugins are how you organize features

A plugin is anything that implements `Plugin` (**[Bevy]** trait): `impl Plugin for X { fn build(&self, app: &mut App) }`. See `particles.rs:19` — `ParticlePlugin` (**[custom]**) is purely a *bundler* that adds 7 sub-plugins (all **[custom]**: `particle::ParticlePlugin`, `ModelPlugin`, `DecayPlugin`, `SimulationPlugin`, `SimulationSizePlugin`, `SpatialIndexPlugin`, `SpawnerPlugin`). Each sub-plugin owns its own resources, events, and systems. This is how Bevy projects scale: every feature is its own plugin.

## 3. Components describe entities (`particle.rs:18`)

```rust
#[derive(Debug, Reflect, Component)]                              // Reflect, Component: [Bevy]
#[require(Transform, ParticleColour, Velocity)]                   // require: [Bevy] attribute
                                                                  //   Transform: [Bevy]
                                                                  //   ParticleColour, Velocity: [custom]
#[component(immutable, on_add = on_add, on_remove = on_remove)]   // [Bevy] hook attributes
pub struct Particle;                                              // [custom] — unit-struct marker
```

Two key things:
- **`#[require(...)]`** (**[Bevy]**) — when you spawn an entity with `Particle`, Bevy auto-inserts the listed components if missing. This replaces the old "Bundle" pattern from earlier Bevy versions.
- **Component hooks** `on_add` / `on_remove` (**[Bevy]** mechanism, custom functions wired into it) — run code on insertion/removal. Used here to keep the `ParticleIndex` resource (**[custom]**) in sync (`particle.rs:29-51`).

`Velocity` (`particle.rs:23`, **[custom]**) is a newtype wrapping `Vec2` (**[Bevy]** glam re-export) with `Deref/DerefMut` (**[std]**) — that's why `**velocity` works inside `compute_forces`.

## 4. Resources are singletons (`particle.rs:14`)

```rust
app.insert_resource(ParticleIndex(Vec::with_capacity(MAX_PARTICLES)));
//   insert_resource: [Bevy] App method
//   ParticleIndex:   [custom] Resource (Vec<Entity> wrapper)
```

Inside a system you ask for them by parameter type: `Res<X>` (read) and `ResMut<X>` (write) — both **[Bevy]**. See `compute_forces` (`simulation.rs:67`):

```rust
mut spatial_index: ResMut<SpatialIndex>,   // ResMut: [Bevy] · SpatialIndex: [custom]
model: Res<Model>,                          // Res: [Bevy]    · Model: [custom]
params: Res<SimulationParams>,              // SimulationParams: [custom]
time: Res<Time>,                            // Time: [Bevy]
```

`Time` (**[Bevy]**) is provided automatically by `DefaultPlugins`. `time.delta_secs()` is the per-frame delta — multiply velocities by this to be framerate-independent.

## 5. Systems are functions, parameters tell Bevy what to fetch

The big one — `compute_forces` (`simulation.rs:67`, **[custom]** system):

```rust
fn compute_forces(
    mut particles: Query<(Entity, &mut Transform, &mut Velocity, &ParticleColour),
                          With<Particle>>,
    //  Query, Entity, With: [Bevy]
    //  Transform: [Bevy]
    //  Velocity, ParticleColour, Particle: [custom]
    mut spatial_index: ResMut<SpatialIndex>,    // [custom] resource via [Bevy] ResMut
    model: Res<Model>,                          // [custom] resource
    params: Res<SimulationParams>,              // [custom] resource
    simulation_size: SimulationSize,            // [custom] SystemParam
    time: Res<Time>,                            // [Bevy]
) -> Result<()> { ... }                         // Result: [Bevy] alias for system error returns
```

- **`Query<(...), With<Particle>>`** (**[Bevy]** type, custom filter type) — iterate every entity that has those components and the `Particle` (**[custom]**) marker.
- **`&mut T`** marks write access; **`&T`** read. Bevy uses this to schedule disjoint systems in parallel.
- **`par_iter_mut()`** (**[Bevy]** Query method, used at `simulation.rs:90`) — Bevy will run particle updates across threads automatically because the borrow checker on components guarantees safety.
- Returning `Result<()>` (the **[Bevy]** result alias, not `std::result::Result`) lets you `?`-propagate errors without panicking.

Register it with `add_systems` (**[Bevy]** App method, at `simulation.rs:23`):

```rust
.add_systems(Update, compute_forces.in_set(AppSystems::Update));
//           ^^^^^^                  ^^^^^^^^^^^^^^^^^^^^^^^^^
//           [Bevy]                  in_set: [Bevy] · AppSystems::Update: [custom]
```

## 6. Schedules and ordering

Bevy has named **schedules** (all **[Bevy]**): `Startup` (once), `Update` (every frame), `FixedUpdate`, `PostUpdate`, etc.

Within a schedule, ordering is undefined unless you specify it. This project uses **system sets** — `SystemSet` is **[Bevy]**, but `AppSystems` itself is **[custom]** (defined in `abiogenesis/src/systems.rs`). Wired up at `main.rs:95`:

```rust
app.configure_sets(Update, (              // configure_sets, Update: [Bevy]
    AppSystems::TickTimers,               // [custom] enum variants
    AnimationSystem::AnimationUpdate,     // [bevy_tweening] 3rd-party crate
    AppSystems::RecordInput,              // [custom]
    AppSystems::Update,                   // [custom]
    AppSystems::Camera,                   // [custom]
).chain());                               // chain: [Bevy] — strict sequential ordering
```

Then individual systems opt into a set with `.in_set(AppSystems::Update)`. Reading order: input recorded before physics, physics before camera follow.

## 7. Spawning entities (`spawner.rs:190`)

```rust
commands.spawn((
    Particle,                                  // [custom] marker
    transform(color),                          // returns Transform — [Bevy]
    color,                                     // ParticleColour — [custom]
    Mesh2d(particle_assets.mesh.clone()),      // Mesh2d: [Bevy] component
                                               // Handle<Mesh>: [Bevy] refcounted handle
));
```

`Commands` (**[Bevy]**) is a deferred queue — the actual spawn happens after the system runs. You hand it a tuple of components (the tuple itself is plain Rust, but each entry must be a `Component`-implementing type — Bevy or custom).

`Mesh2d` (**[Bevy]**) is what makes the entity render in 2D; the `Handle<Mesh>` (**[Bevy]**) inside it points to a mesh stored in `Assets<Mesh>` (**[Bevy]**).

Despawn is symmetric (`spawner.rs:149`): `commands.entity(entity).despawn()` — both methods **[Bevy]**.

## 8. Assets (`spawner.rs:56`)

```rust
fn init_assets(
    mut commands: Commands,                          // [Bevy]
    mut meshes: ResMut<Assets<Mesh>>,                // ResMut, Assets, Mesh: all [Bevy]
    mut materials: ResMut<Assets<ColorMaterial>>,    // ColorMaterial: [Bevy]
) {
    let mesh = meshes.add(Circle::new(1.0));         // Circle: [Bevy] primitive shape
    let red = materials.add(Color::from(RED));       // Color: [Bevy] · RED: [custom] const
    ...
}
```

`Assets<T>` (**[Bevy]**) is a typed asset store. `add()` returns a `Handle<T>` (**[Bevy]**) — cheap to clone, refcounted. Store handles in a custom resource (`ParticleAssets` is **[custom]**) and reuse — never re-add the same mesh per entity.

## 9. Events vs Observers — two flavours of messaging

**Buffered events** (`spawner.rs:14, 211`):

```rust
app.add_event::<SpawnParticle>();    // add_event: [Bevy] · SpawnParticle: [custom] event

fn spawn_particle(mut spawn_particles: EventReader<SpawnParticle>) {
    //                                   ^^^^^^^^^^^ [Bevy]
    for ev in spawn_particles.read() { ... }
}
```

`EventReader<T>` / `EventWriter<T>` are **[Bevy]**; the event type itself (`SpawnParticle`) is **[custom]**. Producers call `EventWriter::send`, consumers drain `EventReader`. Decoupled, framewise, double-buffered for 2 frames.

**Observers** (`spawner.rs:20, 135`) — newer, fire-and-forget:

```rust
app.add_observer(respawn_particles);   // add_observer: [Bevy]

fn respawn_particles(_trigger: Trigger<Respawn>, ...) { ... }
                              // Trigger: [Bevy] · Respawn: [custom] event
// Anywhere:
commands.trigger(Respawn);             // trigger: [Bevy] Commands method
```

Triggers run **synchronously** when `commands` flushes. Use observers for one-shot UI-driven actions (reset, randomise, import).

## 10. Putting it together — the data flow each frame

For one Update tick of this app:

1. **Input phase** — `ControlsPlugin` (**[custom]**) reads keyboard/mouse via `ButtonInput<KeyCode>` etc. (all **[Bevy]**), fires events (`SpawnParticle`, `Respawn` — **[custom]**).
2. **Physics phase** — `compute_forces` (**[custom]**) queries every particle, asks the `SpatialIndex` (**[custom]**) for neighbours, sums forces from the `Model` (**[custom]** — the NxN interaction matrix), updates `Velocity` (**[custom]**) and `Transform` (**[Bevy]**).
3. **Spawning phase** — `spawn_particle` (**[custom]**) drains pending `SpawnParticle` events; if at capacity, recycles the oldest entity in `ParticleIndex` (**[custom]**).
4. **Camera phase** — `CameraPlugin` (**[custom]**) adjusts pan/zoom (which moves *particles*, not the `Camera2d` (**[Bevy]**), in this project).
5. **Render** — Bevy's renderer (**[Bevy]**, fully internal) picks up every `Mesh2d` + `Transform` and draws it.

## 11. Three Bevy idioms worth internalizing

- **Newtype + `Deref`** for cheap, typed components — pure Rust pattern, but applied here to make custom components like `Velocity` and `ParticleIndex` (both **[custom]**) feel like their inner types.
- **Plugin per concern**, with its own resources/events/systems, then bundle into a parent plugin (`ParticlePlugin` in `particles.rs` — **[custom]** parent of 7 **[custom]** sub-plugins).
- **Sets + `.chain()`** in one place (`main.rs:95`) — `.chain()` is **[Bevy]**, the set enum (`AppSystems`) is **[custom]**. Reading `add_systems(... .in_set(X))` then tells you when a system runs at a glance.

## Where to look next in this repo

(All files below are **[custom]** — this is your project's source.)

- `src/particles/model.rs` — the interaction matrix and presets (**[custom]** resource using **[Bevy]** change detection).
- `src/particles/spatial_index.rs` — wrapping a **[custom]** `SpatialHashGrid` as a **[Bevy]** resource.
- `src/controls.rs` — input observers (**[Bevy]** mechanism) attached to a **[custom]** `UIRoot` entity.
- `src/ui.rs` — Bevy's UI tree (**[Bevy]** `Node` components etc.) built declaratively in a **[custom]** `respawn_ui` system.
