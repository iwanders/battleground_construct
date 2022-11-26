use rand::Rng;
use three_d::*;

use super::effects;
use super::instanced_entity;

use battleground_construct::display;
use battleground_construct::display::primitives::Drawable;
use battleground_construct::Construct;
use engine::prelude::*;

use battleground_construct::display::EffectId;
use effects::RenderableEffect;

use instanced_entity::InstancedEntity;

use three_d::renderer::material::PhysicalMaterial;

/// The object used to render a construct.
#[derive(Default)]
pub struct ConstructRender {
    static_gms: Vec<Gm<Mesh, PhysicalMaterial>>,
    instanced_meshes: std::collections::HashMap<
        display::primitives::Primitive,
        InstancedEntity<PhysicalMaterial>,
    >,

    effects: std::collections::HashMap<EffectId, Box<dyn RenderableEffect>>,
}

impl ConstructRender {
    pub fn populate_default(&mut self, context: &Context) {
        let mut ground_plane = Gm::new(
            Mesh::new(&context, &CpuMesh::square()),
            PhysicalMaterial::new_opaque(
                &context,
                &CpuMaterial {
                    albedo: Color::new_opaque(128, 128, 128),
                    ..Default::default()
                },
            ),
        );
        ground_plane.set_transformation(
            Mat4::from_translation(vec3(0.0, 0.0, 0.0)) * Mat4::from_scale(1000.0),
        );
        self.static_gms.push(ground_plane);

        let mut cube = Gm::new(
            Mesh::new(&context, &CpuMesh::cube()),
            three_d::renderer::material::PhysicalMaterial::new_opaque(
                &context,
                &CpuMaterial {
                    albedo: Color {
                        r: 0,
                        g: 0,
                        b: 255,
                        a: 255,
                    },
                    ..Default::default()
                },
            ),
        );
        cube.set_transformation(
            Mat4::from_translation(vec3(0.0, 0.0, 1.0)) * Mat4::from_scale(0.2),
        );
        self.static_gms.push(cube);

        let mut cube = Gm::new(
            Mesh::new(&context, &CpuMesh::cube()),
            three_d::renderer::material::PhysicalMaterial::new_opaque(
                &context,
                &CpuMaterial {
                    albedo: Color {
                        r: 255,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                    ..Default::default()
                },
            ),
        );
        cube.set_transformation(
            Mat4::from_translation(vec3(1.0, 0.0, 0.0)) * Mat4::from_scale(0.2),
        );
        self.static_gms.push(cube);
    }

    /// Return a list of geometrise to be used for shadow calculations.
    pub fn shadow_meshes(&self) -> Vec<&impl Geometry> {
        self.instanced_meshes
            .values()
            .map(|x| &x.gm().geometry)
            .collect::<_>()
    }

    /// Return the objects to be rendered.
    pub fn objects(&self) -> Vec<&dyn Object> {
        let mut renderables: Vec<&dyn Object> = vec![];
        // renderables.push(&fireworks);
        renderables.append(
            &mut self
                .instanced_meshes
                .values()
                .map(|x| x.gm() as &dyn Object)
                .collect::<Vec<&dyn Object>>(),
        );
        renderables.append(
            &mut self
                .static_gms
                .iter()
                .map(|x| x as &dyn Object)
                .collect::<Vec<_>>(),
        );

        renderables.append(
            &mut self
                .effects
                .iter()
                .map(|v| v.1.object())
                .filter(|v| v.is_some())
                .map(|v| v.unwrap())
                .collect::<Vec<_>>(),
        );
        renderables
    }

    fn update_instances(&mut self) {
        for instance_entity in self.instanced_meshes.values_mut() {
            instance_entity.update_instances()
        }
    }

    fn reset_instances(&mut self) {
        self.instanced_meshes.clear();
    }

    pub fn render(&mut self, camera: &Camera, context: &Context, construct: &Construct) {
        // a new cycle, clear the previous instances.
        self.reset_instances();

        // Iterate through all displayables.
        self.component_to_meshes::<display::tank_body::TankBody>(context, construct);
        self.component_to_meshes::<display::tank_tracks::TankTracks>(context, construct);

        self.component_to_meshes::<display::tank_turret::TankTurret>(context, construct);
        self.component_to_meshes::<display::tank_barrel::TankBarrel>(context, construct);
        self.component_to_meshes::<display::tank_bullet::TankBullet>(context, construct);

        self.component_to_meshes::<display::debug_box::DebugBox>(context, construct);
        self.component_to_meshes::<display::debug_sphere::DebugSphere>(context, construct);

        self.component_to_meshes::<display::flag::Flag>(context, construct);

        // Get the current effect keys.
        let mut start_keys = self
            .effects
            .keys()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        let mut effect_ids = vec![];
        effect_ids.append(
            &mut self.component_to_effects::<display::particle_emitter::ParticleEmitter>(
                context, camera, construct,
            ),
        );

        // Now we remove all effects that are no longer present
        for k in effect_ids {
            start_keys.remove(&k);
        }

        // Now, anything that still exists in start_keys no longer exists this cycle and thus should be pruned.
        for k in start_keys {
            self.effects.remove(&k);
        }

        // Update the actual instances
        self.update_instances();
    }

    /// Function to iterate over the components and convert their drawables into elements.
    fn component_to_meshes<C: Component + Drawable + 'static>(
        &mut self,
        context: &Context,
        construct: &Construct,
    ) {
        let current_time = construct.elapsed_as_f64() as f32;

        for (element_id, component_with_drawables) in construct.world().component_iter::<C>() {
            // Get the world pose for this entity, to add draw transform local to this component.
            let world_pose = construct.entity_pose(&element_id);
            for el in component_with_drawables.drawables() {
                self.add_primitive_element(context, &el, world_pose.transform())
            }
        }
    }

    /// Function to iterate over the components and convert their drawables into elements.
    fn component_to_effects<C: Component + Drawable + 'static>(
        &mut self,
        context: &Context,
        camera: &Camera,
        construct: &Construct,
    ) -> Vec<EffectId> {
        let current_time = construct.elapsed_as_f64() as f32;
        let mut res = vec![];

        for (element_id, component_with_drawables) in construct.world().component_iter::<C>() {
            // Get the world pose for this entity, to add draw transform local to this component.
            let world_pose = construct.entity_pose(&element_id);

            for effect in component_with_drawables.effects() {
                self.update_effect(
                    context,
                    camera,
                    &effect,
                    world_pose.transform(),
                    current_time,
                );
                res.push(effect.id);
            }
        }
        res
    }

    fn update_effect(
        &mut self,
        context: &Context,
        camera: &Camera,
        effect: &display::primitives::Effect,
        entity_transform: &Matrix4<f32>,
        timestamp: f32,
    ) {
        if !self.effects.contains_key(&effect.id) {
            // add this effect.
            match effect.effect {
                display::primitives::EffectType::ParticleEmitter(particle_emitter) => {
                    self.effects.insert(
                        effect.id,
                        Box::new(effects::ParticleEmitter::new(
                            context,
                            *entity_transform,
                            timestamp,
                            &particle_emitter,
                        )),
                    );
                }
            }
        }
        let effect = self
            .effects
            .get_mut(&effect.id)
            .expect("just checked it, will be there");
        effect.update(camera, *entity_transform, timestamp);
    }

    /// Add elements to the instances.
    fn add_primitive_element(
        &mut self,
        context: &Context,
        el: &display::primitives::Element,
        entity_transform: &Matrix4<f32>,
    ) {
        if !self.instanced_meshes.contains_key(&el.primitive) {
            let primitive_mesh = match el.primitive {
                display::primitives::Primitive::Cuboid(cuboid) => {
                    let mut m = CpuMesh::cube();
                    // Returns an axis aligned unconnected cube mesh with positions in the range [-1..1] in all axes.
                    // So default box is not identity.
                    m.transform(&Mat4::from_nonuniform_scale(
                        cuboid.length / 2.0,
                        cuboid.width / 2.0,
                        cuboid.height / 2.0,
                    ))
                    .unwrap();
                    m
                }
                display::primitives::Primitive::Sphere(sphere) => {
                    let mut m = CpuMesh::sphere(16);
                    m.transform(&Mat4::from_scale(sphere.radius)).unwrap();
                    m
                }
                display::primitives::Primitive::Cylinder(cylinder) => {
                    let mut m = CpuMesh::cylinder(16);
                    m.transform(&Mat4::from_nonuniform_scale(
                        cylinder.height,
                        cylinder.radius,
                        cylinder.radius,
                    ))
                    .unwrap();
                    m
                }
            };
            self.instanced_meshes.insert(
                el.primitive,
                InstancedEntity::new_physical(context, &primitive_mesh),
            );
        }

        let instanced = self
            .instanced_meshes
            .get_mut(&el.primitive)
            .expect("just checked it, will be there");
        let transform = entity_transform * el.transform;
        let color = Color {
            r: el.color.r,
            g: el.color.g,
            b: el.color.b,
            a: el.color.a,
        };
        instanced.add(transform, color);
    }
}
