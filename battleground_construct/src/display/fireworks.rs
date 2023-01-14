use engine::*;
use crate::components;
use crate::util::cgmath::Vec3;
use crate::components::timed_function_trigger::TimedFunctionTrigger;
pub fn create_firework(start: Vec3, color: crate::display::Color, world: &mut World) {
    let firework_entity = world.add_entity();
    world.add_component(
        firework_entity,
        components::pose::Pose::from_xyz(start.x, start.y, start.z),
    );
    world.add_component(
        firework_entity,
        components::velocity::Velocity::from_linear(Vec3::new(0.0, 1.0, 8.0)),
    );
    world.add_component(
        firework_entity,
        components::acceleration::Acceleration::gravity(),
    );
    // world.add_component(firework_entity, display::debug_box::DebugBox::cube(0.1));

    let effect_id = components::id_generator::generate_id(world);
    world.add_component(
        firework_entity,
        crate::display::particle_emitter::ParticleEmitter::bullet_trail(
            effect_id,
            0.05,
            color,
        ),
    );

    world.add_component(
        firework_entity,
        TimedFunctionTrigger::after(
            1.0,
            move |firework_entity: EntityId, world: &mut engine::World| {
                // world.remove_entity(firework_entity);
                world.remove_component::<components::velocity::Velocity>(
                    firework_entity,
                );

                let effect_id = components::id_generator::generate_id(world);
                world.add_component(
                    firework_entity,
                    crate::display::particle_emitter::ParticleEmitter::firework(
                        effect_id,
                        2.0,
                        color,
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