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

struct Properties {
    object: InstancedEntity<PhysicalMaterial>,
    cast_shadow: bool,
}

/// The object used to render a construct.
pub struct ConstructRender {
    static_gms: Vec<Gm<Mesh, PhysicalMaterial>>,
    instanced_meshes: std::collections::HashMap<u64, Properties>,
    grid_lines: InstancedEntity<ColorMaterial>,

    effects: std::collections::HashMap<EffectId, Box<dyn RenderableEffect>>,
}

impl ConstructRender {
    pub fn new(context: &Context) -> Self {
        let mut ground_plane = Gm::new(
            Mesh::new(context, &CpuMesh::square()),
            PhysicalMaterial::new_opaque(
                context,
                &CpuMaterial {
                    albedo: Color::new_opaque(128, 128, 128),
                    ..Default::default()
                },
            ),
        );
        ground_plane.set_transformation(
            Mat4::from_translation(vec3(0.0, 0.0, 0.0)) * Mat4::from_scale(1000.0),
        );
        let static_gms = vec![ground_plane];

        // At some point we should try to render the grid only underneath robots, or something.
        let mut grid_lines = InstancedEntity::new_colored(context, &CpuMesh::cylinder(4));
        let mut lines = vec![];
        let lower = -10isize;
        let upper = 10;
        let main = 5;
        let t = 0.01;
        let sub_color = Color::new_opaque(150, 150, 150);
        let main_color = Color::new_opaque(255, 255, 255);
        fn line(
            x0: isize,
            y0: isize,
            x1: isize,
            y1: isize,
            width: f32,
            color: Color,
        ) -> (Vec3, Vec3, f32, Color) {
            (
                vec3(x0 as f32, y0 as f32, 0.0),
                vec3(x1 as f32, y1 as f32, 0.0),
                width,
                color,
            )
        }
        for x in lower + 1..upper {
            let color = if x.rem_euclid(main) == 0 {
                main_color
            } else {
                sub_color
            };
            lines.push(line(x, upper, x, lower, t, color));
            lines.push(line(lower, x, upper, x, t, color));
        }
        lines.push(line(lower - 5, upper, upper + 5, upper, t, main_color));
        lines.push(line(lower - 5, lower, upper + 5, lower, t, main_color));

        lines.push(line(upper, lower - 5, upper, upper + 5, t, main_color));
        lines.push(line(lower, lower - 5, lower, upper + 5, t, main_color));

        grid_lines.set_lines(&lines);

        ConstructRender {
            static_gms,
            grid_lines,
            instanced_meshes: Default::default(),
            effects: Default::default(),
        }
    }

    /// Return a list of geometrise to be used for shadow calculations.
    pub fn shadow_meshes(&self) -> Vec<&impl Geometry> {
        self.instanced_meshes
            .values()
            .filter(|p| p.cast_shadow)
            .map(|x| &x.object.gm().geometry)
            .collect::<_>()
    }

    /// Return the objects to be rendered.
    pub fn objects(&self) -> Vec<&dyn Object> {
        let mut renderables: Vec<&dyn Object> = vec![];
        renderables.push(self.grid_lines.gm());
        // renderables.push(&fireworks);
        renderables.append(
            &mut self
                .instanced_meshes
                .values()
                .map(|x| x.object.gm() as &dyn Object)
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
                .filter_map(|v| v.1.object())
                .collect::<Vec<_>>(),
        );
        renderables
    }

    fn update_instances(&mut self) {
        for instance_entity in self.instanced_meshes.values_mut() {
            instance_entity.object.update_instances()
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
        self.component_to_meshes::<display::radar_model::RadarModel>(context, construct);

        self.component_to_meshes::<display::draw_module::DrawComponent>(context, construct);

        self.component_to_meshes::<display::debug_box::DebugBox>(context, construct);
        self.component_to_meshes::<display::debug_sphere::DebugSphere>(context, construct);
        self.component_to_meshes::<display::debug_lines::DebugLines>(context, construct);

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
        effect_ids.append(
            &mut self.component_to_effects::<display::deconstructor::Deconstructor>(
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
        for (element_id, component_with_drawables) in construct.world().component_iter::<C>() {
            // Get the world pose for this entity, to add draw transform local to this component.
            let world_pose = construct.entity_pose(element_id);
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
        let current_time = construct.elapsed_as_f32();
        let mut res = vec![];

        for (element_id, component_with_drawables) in construct.world().component_iter::<C>() {
            // Get the world pose for this entity, to add draw transform local to this component.
            let world_pose = construct.entity_pose(element_id);

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
        self.effects.entry(effect.id).or_insert_with(|| {
            // add this effect.
            match effect.effect {
                display::primitives::EffectType::ParticleEmitter { particle_type, .. } => {
                    Box::new(effects::ParticleEmitter::new(
                        context,
                        *entity_transform,
                        timestamp,
                        &particle_type,
                    ))
                }
                display::primitives::EffectType::Deconstructor {
                    ref elements,
                    ref impacts,
                    ..
                } => Box::new(effects::Deconstructor::new(
                    context,
                    *entity_transform,
                    timestamp,
                    elements,
                    impacts,
                )),
            }
        });
        let effect_renderable = self
            .effects
            .get_mut(&effect.id)
            .expect("just checked it, will be there");
        effect_renderable.update(&effect.effect, camera, *entity_transform, timestamp);
    }

    /// Add elements to the instances.
    fn add_primitive_element(
        &mut self,
        context: &Context,
        el: &display::primitives::Element,
        entity_transform: &Matrix4<f32>,
    ) {
        self.instanced_meshes
            .entry(el.primitive.to_draw_key())
            .or_insert_with(|| {
                let mut cast_shadow = true;
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
                    display::primitives::Primitive::Line(_line) => {
                        cast_shadow = false;
                        CpuMesh::cylinder(4)
                    }
                };

                Properties {
                    object: InstancedEntity::new_physical(context, &primitive_mesh),
                    cast_shadow,
                }
            });

        let instanced = &mut self
            .instanced_meshes
            .get_mut(&el.primitive.to_draw_key())
            .expect("just checked it, will be there")
            .object;
        let transform = entity_transform * el.transform;
        let color = Color {
            r: el.color.r,
            g: el.color.g,
            b: el.color.b,
            a: el.color.a,
        };
        match &el.primitive {
            display::primitives::Primitive::Line(l) => {
                use battleground_construct::util::cgmath::ToHomogenous;
                use battleground_construct::util::cgmath::ToTranslation;
                let p0_original = vec3(l.p0.0, l.p0.1, l.p0.2);
                let p1_original = vec3(l.p1.0, l.p1.1, l.p1.2);
                let p0_transformed = (transform * p0_original.to_h()).to_translation();
                let p1_transformed = (transform * p1_original.to_h()).to_translation();
                instanced.add_line(p0_transformed, p1_transformed, l.width, color);
            }
            _ => instanced.add(transform, color),
        }
    }
}
