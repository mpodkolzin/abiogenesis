use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::particles::colour::ParticleColour;

pub const MAX_PARTICLES: usize = 3000;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ParticleIndex(Vec::with_capacity(MAX_PARTICLES)))
            .add_systems(PostStartup, log_avg_mass);
    }
}

fn log_avg_mass(masses: Query<&Mass, With<Particle>>) {
    let (sum, count) = masses
        .iter()
        .fold((0.0_f32, 0u32), |(s, c), m| (s + **m, c + 1));

    if count > 0 {
        info!(
            "avg mass over {count} particles: {:.3}",
            sum / count as f32
        );
    }
}

#[derive(Debug, Reflect, Component)]
#[require(Transform, ParticleColour, Velocity, Mass)]
#[component(immutable, on_add = on_add, on_remove = on_remove)]
pub struct Particle;

#[derive(Debug, Reflect, Component, Default, Clone, Copy, Deref, DerefMut)]
pub struct Velocity(Vec2);

#[derive(Debug, Reflect, Component, Default, Clone, Copy, Deref, DerefMut)]
pub struct Mass(pub f32);

#[derive(Debug, Reflect, Resource, Deref, DerefMut)]
pub struct ParticleIndex(pub Vec<Entity>);

fn on_add(mut world: DeferredWorld, ctx: HookContext) {
    let mut particle_index = world.resource_mut::<ParticleIndex>();

    if particle_index.len() >= MAX_PARTICLES {
        tracing::warn!("max particles reached");
        return;
    }

    particle_index.push(ctx.entity);
}

fn on_remove(mut world: DeferredWorld, ctx: HookContext) {
    let mut particle_index = world.resource_mut::<ParticleIndex>();

    let Some(index) = particle_index
        .iter()
        .position(|&entity| entity == ctx.entity)
    else {
        return;
    };

    particle_index.remove(index);
}
