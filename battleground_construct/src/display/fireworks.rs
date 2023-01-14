use crate::components;
use crate::components::timed_function_trigger::TimedFunctionTrigger;
use crate::util::cgmath::Vec3;
use engine::*;

/// Yuck function to create an f32 within 0.0 to 1.0.
fn random(random_bytes: &[u8]) -> f32 {
    let mut v: u32 = 0x3F800000; // up to 0x3FFFFFFF is 1.000000 - 1.9999999
                                 // Thats 23 bits on the right hand side to populate, that's 3 bytes.
    let b0 = random_bytes[0];
    let b1 = random_bytes[1];
    let b2 = random_bytes[2];
    v |= (b0 as u32 >> 1) << 16;
    v |= (b1 as u32) << 8;
    v |= b2 as u32;
    f32::from_bits(v) - 1.0
}

pub fn create_firework(
    start: Vec3,
    color: crate::display::Color,
    world: &mut World,
    random_addition: usize,
) {
    let t = world
        .component_iter::<components::clock::Clock>()
        .next()
        .expect("Should have one clock")
        .1
        .elapsed_as_f32();
    // t is clearly the most amazing entropy source... lets hash it.
    let random_u64 = {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let state = &mut hasher;
        t.to_bits().hash(state);
        random_addition.hash(state);
        hasher.finish()
    };
    // println!("random_u64: {random_u64:0>64b}"); // good enough.
    // Convert to bytes and create a float between 0.0 and 1.0 from it.
    let random_bytes = random_u64.to_le_bytes();
    let direction_rng = random(&random_bytes[0..4]);
    let velocity_rng = random(&random_bytes[4..7]);

    // Spawn the fireworks entity with the provided values.
    let firework_entity = world.add_entity();
    world.add_component(
        firework_entity,
        components::pose::Pose::from_xyz(start.x, start.y, start.z),
    );
    let vz = 6.0 + velocity_rng * 2.0;
    let vx = (direction_rng * 2.0 * std::f32::consts::PI).cos();
    let vy = (direction_rng * 2.0 * std::f32::consts::PI).sin();
    world.add_component(
        firework_entity,
        components::velocity::Velocity::from_linear(Vec3::new(vx, vy, vz)),
    );
    world.add_component(
        firework_entity,
        components::acceleration::Acceleration::gravity(),
    );
    // world.add_component(firework_entity, display::debug_box::DebugBox::cube(0.1));

    let effect_id = components::id_generator::generate_id(world);
    world.add_component(
        firework_entity,
        crate::display::particle_emitter::ParticleEmitter::bullet_trail(effect_id, 0.05, color),
    );

    world.add_component(
        firework_entity,
        TimedFunctionTrigger::after(
            1.0,
            move |firework_entity: EntityId, world: &mut engine::World| {
                // remove the velocity to freeze in place.
                world.remove_component::<components::velocity::Velocity>(firework_entity);

                // Genereate an effect id.
                let effect_id = components::id_generator::generate_id(world);
                // Creat the emitter.
                world.add_component(
                    firework_entity,
                    crate::display::particle_emitter::ParticleEmitter::firework(
                        effect_id, 2.0, color,
                    ),
                );
                world.add_component(
                    firework_entity,
                    crate::components::expiry::Expiry::lifetime(15.0),
                );
            },
        ),
    );
}
