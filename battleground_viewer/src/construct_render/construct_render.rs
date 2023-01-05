use three_d::*;

use super::effects;
use super::render::{
    BatchProperties, MeshGeometry, PrimitiveGeometry, RenderPass, RenderableGeometry,
};

use battleground_construct::components::unit::UnitId;
use battleground_construct::display;
use battleground_construct::display::primitives::{Drawable, Primitive};
use battleground_construct::Construct;
use engine::prelude::*;

use crate::construct_render::util::ColorConvert;
use effects::RetainedEffect;

use three_d::renderer::material::PhysicalMaterial;

/// The object used to render a construct.
pub struct ConstructRender {
    static_meshes: MeshGeometry<PhysicalMaterial>,

    /// All meshes that are rendered with a physical material (both opaque and translucent)
    base_primitives: PrimitiveGeometry<PhysicalMaterial>,

    /// The 'core' of glowing primitives
    emissive_primitives: PrimitiveGeometry<ColorMaterial>,

    /// All meshes that are used to produce glows
    glow_primitives: PrimitiveGeometry<ColorMaterial>,

    /// All meshes that will be rendered with a fence material
    fence_primitives: PrimitiveGeometry<ColorMaterial>,

    /// element to draw the selection boxes without any shading on the lines.
    overlay_primitives: PrimitiveGeometry<ColorMaterial>,

    /// Tracked effects that are carried over to the next frame.
    effects: std::collections::HashMap<u64, Box<dyn RetainedEffect>>,
}

impl ConstructRender {
    pub fn new() -> Self {
        let mut static_meshes = MeshGeometry::new(|pass| match pass {
            RenderPass::BaseScene | RenderPass::NonEmmisivesDepth => true,
            _ => false,
        });
        let mut base_primitives = PrimitiveGeometry::new(|pass| match pass {
            RenderPass::ShadowMaps | RenderPass::BaseScene | RenderPass::NonEmmisivesDepth => true,
            _ => false,
        });
        let mut emissive_primitives = PrimitiveGeometry::new(|pass| match pass {
            RenderPass::ShadowMaps | RenderPass::BaseScene => true,
            _ => false,
        });
        let mut glow_primitives = PrimitiveGeometry::new(|pass| match pass {
            RenderPass::Emmisives => true,
            _ => false,
        });
        let mut fence_primitives = PrimitiveGeometry::new(|pass| match pass {
            RenderPass::Fences => true,
            _ => false,
        });
        let mut overlay_primitives = PrimitiveGeometry::new(|pass| match pass {
            RenderPass::BaseScene => true,
            _ => false,
        });

        ConstructRender {
            static_meshes,
            base_primitives,
            emissive_primitives,
            glow_primitives,
            fence_primitives,
            overlay_primitives,
            effects: Default::default(),
        }
    }

    // The idea is to use this method to simplify `prepare_frame`, `finish_frame`, `objects` and
    // `geometries` by having them all invoke here `self.do_for_renderables(|r| { ... })`... The
    // reason for the closure is that it is impossible to return a Vec of references (or at least,
    // when I do I get hit by the borrow checker because I'm returning unowned values, which makes
    // sense.)
    fn do_for_renderables(&self, f: impl Fn(&dyn RenderableGeometry) -> ()) {
        f(&self.static_meshes);
        f(&self.base_primitives);
        f(&self.emissive_primitives);
        f(&self.glow_primitives);
        f(&self.fence_primitives);
        f(&self.overlay_primitives);
        for effect in self.effects.values() {
            f(effect.as_renderable());
        }
    }

    fn add_static_meshes(&mut self) {
        // Ground plane
        self.static_meshes.add_mesh(
            &CpuMesh::square(),
            Mat4::from_translation(vec3(0.0, 0.0, 0.0)) * Mat4::from_scale(1000.0),
            Color::new_opaque(128, 128, 128),
        );
    }

    fn add_grid(&mut self) {
        // Grid goes into overlay for now...
        let lower = -10isize;
        let upper = 10;
        let main = 5;
        let t = 0.01;
        let sub_color = Color::new_opaque(150, 150, 150);
        let main_color = Color::new_opaque(255, 255, 255);
        let batch_hints = BatchProperties::Basic {
            is_transparent: false,
        };
        let no_transform = Matrix4::one();

        fn line(x0: isize, y0: isize, x1: isize, y1: isize, width: f32) -> Primitive {
            Primitive::Line(battleground_construct::display::primitives::Line {
                p0: (x0 as f32, y0 as f32, 0.0),
                p1: (x1 as f32, y1 as f32, 0.0),
                width,
            })
        }

        let mut lines = vec![];
        for x in lower + 1..upper {
            let color = if x.rem_euclid(main) == 0 {
                main_color
            } else {
                sub_color
            };
            lines.push((line(x, upper, x, lower, t), color));
            lines.push((line(lower, x, upper, x, t), color));
        }
        lines.push((line(lower - 5, upper, upper + 5, upper, t), main_color));
        lines.push((line(lower - 5, lower, upper + 5, lower, t), main_color));

        lines.push((line(upper, lower - 5, upper, upper + 5, t), main_color));
        lines.push((line(lower, lower - 5, lower, upper + 5, t), main_color));

        for (line, color) in lines {
            self.overlay_primitives
                .add_primitive(batch_hints, line, no_transform, color);
        }
    }

    pub fn camera_view(&self, camera: &Camera, construct: &Construct) -> Option<(Vec3, Vec3)> {
        let mut current_pos = *camera.position();
        let mut current_target = *camera.target();
        let mut modified: bool = false;
        use battleground_construct::components::camera_position::CameraPosition;
        use battleground_construct::components::camera_target::CameraTarget;
        use battleground_construct::util::cgmath::ToTranslation;
        let opt_camera_position = construct.world.component_iter::<CameraPosition>().next();
        if let Some((entity, _)) = opt_camera_position {
            let world_pose = construct.entity_pose(entity);
            current_pos = world_pose.to_translation();
            modified = true;
        }

        let opt_camera_target = construct.world.component_iter::<CameraTarget>().next();
        if let Some((entity, _)) = opt_camera_target {
            let world_pose = construct.entity_pose(entity);
            current_target = world_pose.to_translation();
            modified = true;
        }

        if modified {
            Some((current_pos, current_target))
        } else {
            None
        }
    }

    pub fn geometries(&self, pass: RenderPass) -> Vec<&impl Geometry> {
        let mut result = vec![];
        result.append(
            &mut self
                .static_meshes
                .geometries(pass)
                .unwrap_or_else(|| vec![]),
        );
        result.append(
            &mut self
                .base_primitives
                .geometries(pass)
                .unwrap_or_else(|| vec![]),
        );
        result.append(
            &mut self
                .emissive_primitives
                .geometries(pass)
                .unwrap_or_else(|| vec![]),
        );
        result.append(
            &mut self
                .glow_primitives
                .geometries(pass)
                .unwrap_or_else(|| vec![]),
        );
        result.append(
            &mut self
                .fence_primitives
                .geometries(pass)
                .unwrap_or_else(|| vec![]),
        );
        result.append(
            &mut self
                .overlay_primitives
                .geometries(pass)
                .unwrap_or_else(|| vec![]),
        );
        for renderable_effect in self.effects.values() {
            result.append(&mut renderable_effect.geometries(pass).unwrap_or_else(|| vec![]));
        }
        result
    }

    pub fn objects(&self, pass: RenderPass) -> Vec<&dyn Object> {
        let mut result = vec![];
        result.append(&mut self.static_meshes.objects(pass).unwrap_or_else(|| vec![]));
        result.append(&mut self.base_primitives.objects(pass).unwrap_or_else(|| vec![]));
        result.append(
            &mut self
                .emissive_primitives
                .objects(pass)
                .unwrap_or_else(|| vec![]),
        );
        result.append(&mut self.glow_primitives.objects(pass).unwrap_or_else(|| vec![]));
        result.append(
            &mut self
                .fence_primitives
                .objects(pass)
                .unwrap_or_else(|| vec![]),
        );
        result.append(
            &mut self
                .overlay_primitives
                .objects(pass)
                .unwrap_or_else(|| vec![]),
        );
        for renderable_effect in self.effects.values() {
            result.append(&mut renderable_effect.objects(pass).unwrap_or_else(|| vec![]));
        }
        result
    }

    fn finish_frame(&mut self, context: &Context) {
        self.static_meshes.finish_frame(context);
        self.base_primitives.finish_frame(context);
        self.emissive_primitives.finish_frame(context);
        self.glow_primitives.finish_frame(context);
        self.fence_primitives.finish_frame(context);
        self.overlay_primitives.finish_frame(context);
        for e in self.effects.values_mut() {
            e.finish_frame(context);
        }
    }

    fn prepare_frame(&mut self) {
        self.static_meshes.prepare_frame();
        self.base_primitives.prepare_frame();
        self.emissive_primitives.prepare_frame();
        self.glow_primitives.prepare_frame();
        self.fence_primitives.prepare_frame();
        self.overlay_primitives.prepare_frame();
        for e in self.effects.values_mut() {
            e.prepare_frame();
        }
    }

    fn add_select_boxes(&mut self, construct: &Construct, selected: &[EntityId]) {
        let boxes = &mut self.overlay_primitives;
        let width = 0.01;
        let color = Color::WHITE;
        let batch_hints = BatchProperties::Basic {
            is_transparent: false,
        };
        for e in selected.iter() {
            if let Some(b) =
                construct
                    .world()
                    .component::<battleground_construct::components::select_box::SelectBox>(*e)
            {
                let transform = construct.entity_pose(*e);
                let w = (b.width() + (b.width() * 0.1).min(0.5)) / 2.0;
                let l = (b.length() + (b.length() * 0.1).min(0.5)) / 2.0;
                let h = (b.height() + (b.height() * 0.1).min(0.5)) / 2.0;
                let points = [
                    (l, w, h),    // 0
                    (l, w, -h),   // 1
                    (l, -w, -h),  // 2
                    (l, -w, h),   // 3
                    (-l, w, h),   // 4
                    (-l, w, -h),  // 5
                    (-l, -w, -h), // 6
                    (-l, -w, h),  // 7
                ];
                let lines = [
                    (0, 1),
                    (1, 2),
                    (2, 3),
                    (4, 0),
                    (0, 3),
                    (1, 5),
                    (2, 6),
                    (3, 7),
                    (4, 5),
                    (5, 6),
                    (6, 7),
                    (7, 4),
                ];
                for (ip0, ip1) in lines {
                    let primtitive =
                        Primitive::Line(battleground_construct::display::primitives::Line {
                            p0: points[ip0],
                            p1: points[ip1],
                            width,
                        });
                    self.overlay_primitives.add_primitive(
                        batch_hints,
                        primtitive,
                        *transform,
                        color,
                    );
                }
            }
        }
    }

    fn selected_to_units(
        construct: &Construct,
        selected: &std::collections::HashSet<EntityId>,
    ) -> std::collections::HashSet<UnitId> {
        construct
            .world()
            .component_iter::<battleground_construct::components::unit_member::UnitMember>()
            .filter(|(e, _m)| selected.contains(e))
            .map(|(_e, m)| m.unit())
            .collect()
    }

    pub fn render(
        &mut self,
        camera: &Camera,
        context: &Context,
        construct: &Construct,
        selected: &std::collections::HashSet<EntityId>,
    ) {
        // a new cycle, clear the previous instances.
        self.prepare_frame();

        // World geometry
        self.add_static_meshes();

        // Overlays
        self.add_grid();
        self.add_select_boxes(
            construct,
            &selected.iter().copied().collect::<Vec<EntityId>>(),
        );

        // Iterate through all displayables to collect meshes
        self.component_to_meshes::<display::artillery_turret::ArtilleryTurret>(construct);
        self.component_to_meshes::<display::artillery_barrel::ArtilleryBarrel>(construct);
        self.component_to_meshes::<display::artillery_body::ArtilleryBody>(construct);

        self.component_to_meshes::<display::tracks_side::TracksSide>(construct);

        self.component_to_meshes::<display::tank_body::TankBody>(construct);
        self.component_to_meshes::<display::tank_turret::TankTurret>(construct);
        self.component_to_meshes::<display::tank_barrel::TankBarrel>(construct);
        self.component_to_meshes::<display::tank_bullet::TankBullet>(construct);

        self.component_to_meshes::<display::radar_model::RadarModel>(construct);

        // We could also pre-calculate all entities that have the correct unit members, and then
        // filter based on that...
        let units = Self::selected_to_units(construct, &selected);
        self.component_to_meshes_filtered::<display::draw_module::DrawComponent, _>(
            construct,
            |e| {
                construct
                    .world()
                    .component::<battleground_construct::components::unit_member::UnitMember>(e)
                    .map(|v| units.contains(&v.unit()))
                    .unwrap_or(false)
            },
        );

        self.component_to_meshes::<display::debug_box::DebugBox>(construct);
        self.component_to_meshes::<display::debug_sphere::DebugSphere>(construct);
        self.component_to_meshes::<display::debug_lines::DebugLines>(construct);
        self.component_to_meshes::<display::debug_elements::DebugElements>(construct);

        self.component_to_meshes::<display::flag::Flag>(construct);
        self.component_to_meshes::<display::display_control_point::DisplayControlPoint>(construct);

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
        self.finish_frame(context);
    }

    /// Function to iterate over the components and convert their drawables into elements.
    fn component_to_meshes_filtered<C: Component + Drawable + 'static, F: Fn(EntityId) -> bool>(
        &mut self,
        construct: &Construct,
        filter_function: F,
    ) {
        for (element_id, component_with_drawables) in construct.world().component_iter::<C>() {
            if !filter_function(element_id) {
                continue;
            }
            // Get the world pose for this entity, to add draw transform local to this component.
            let world_pose = construct.entity_pose(element_id);
            for el in component_with_drawables.drawables() {
                self.add_primitive_element(&el, world_pose.transform())
            }
        }
    }

    /// Function to iterate over the components and convert their drawables into elements.
    fn component_to_meshes<C: Component + Drawable + 'static>(&mut self, construct: &Construct) {
        self.component_to_meshes_filtered::<C, _>(construct, |_| true);
    }

    /// Function to iterate over the components and convert their drawables into elements.
    fn component_to_effects<C: Component + Drawable + 'static>(
        &mut self,
        context: &Context,
        camera: &Camera,
        construct: &Construct,
    ) -> Vec<u64> {
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
        el: &display::primitives::Element,
        entity_transform: &Matrix4<f32>,
    ) {
        let element_transform = *entity_transform * el.transform;
        match el.material {
            battleground_construct::display::primitives::Material::FlatMaterial(flat_material) => {
                let batch_properties = BatchProperties::Basic {
                    is_transparent: flat_material.is_transparent,
                };
                if flat_material.is_emissive {
                    self.emissive_primitives.add_primitive(
                        batch_properties,
                        el.primitive,
                        element_transform,
                        flat_material.color.to_color(),
                    );
                    self.glow_primitives.add_primitive(
                        batch_properties,
                        el.primitive,
                        element_transform,
                        flat_material.emissive.to_color(),
                    );
                } else {
                    self.base_primitives.add_primitive(
                        batch_properties,
                        el.primitive,
                        element_transform,
                        flat_material.color.to_color(),
                    );
                }
            }
            battleground_construct::display::primitives::Material::FenceMaterial(
                fence_material,
            ) => {
                self.fence_primitives.add_primitive(
                    BatchProperties::None,
                    el.primitive,
                    element_transform,
                    fence_material.color.to_color(),
                );
            }
        }
    }
}
