use three_d::*;

use super::effects;
use super::instanced_entity;

use battleground_construct::components::unit::UnitId;
use battleground_construct::display;
use battleground_construct::display::primitives::{Drawable, Primitive};
use battleground_construct::Construct;
use engine::prelude::*;

use crate::construct_render::util::ColorConvert;
use effects::RenderableEffect;

use instanced_entity::InstancedEntity;

use three_d::renderer::material::PhysicalMaterial;

struct Properties<M: Material> {
    object: InstancedEntity<M>,
    cast_shadow: bool,
    is_emissive: bool,
}

trait DrawableKey {
    fn to_draw_key(&self) -> u64;
}

impl DrawableKey for battleground_construct::display::primitives::Primitive {
    fn to_draw_key(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let state = &mut hasher;
        match *self {
            Primitive::Cuboid(cube) => {
                0usize.hash(state);
                cube.width.to_bits().hash(state);
                cube.length.to_bits().hash(state);
                cube.height.to_bits().hash(state);
            }
            Primitive::Sphere(sphere) => {
                1usize.hash(state);
                sphere.radius.to_bits().hash(state);
            }
            Primitive::Cylinder(cylinder) => {
                2usize.hash(state);
                cylinder.radius.to_bits().hash(state);
                cylinder.height.to_bits().hash(state);
            }
            Primitive::Line(_line) => {
                // All lines hash the same.
                3usize.hash(state);
            }
            Primitive::Cone(cone) => {
                4usize.hash(state);
                cone.radius.to_bits().hash(state);
                cone.height.to_bits().hash(state);
            }
            Primitive::Circle(circle) => {
                5usize.hash(state);
                circle.radius.to_bits().hash(state);
            }
        }
        // val
        hasher.finish()
    }
}

impl DrawableKey for battleground_construct::display::primitives::Material {
    fn to_draw_key(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let state = &mut hasher;
        match *self {
            battleground_construct::display::primitives::Material::FlatMaterial(material) => {
                1usize.hash(state);
                // Key based on whether this is transparent, and or emissive. but not on color.
                material.is_transparent.hash(state);
                // material.is_emissive.hash(state);
                // InstancedShapes doesn't handle emissive, if the material is emissive, key it by
                // the color...
                if material.is_emissive {
                    material.emissive.r.hash(state);
                    material.emissive.g.hash(state);
                    material.emissive.b.hash(state);
                    material.emissive.a.hash(state);
                }
            }
            battleground_construct::display::primitives::Material::FenceMaterial(_) => {
                2usize.hash(state);
            }
        }
        hasher.finish()
    }
}

impl DrawableKey for battleground_construct::display::primitives::Element {
    fn to_draw_key(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let state = &mut hasher;
        self.material.to_draw_key().hash(state);
        self.primitive.to_draw_key().hash(state);
        hasher.finish()
    }
}

/// The object used to render a construct.
pub struct ConstructRender {
    static_geometries: Vec<Gm<Mesh, PhysicalMaterial>>,

    /// All meshes that are rendered with a physical material (both opaque and translucent)
    pbr_meshes: std::collections::HashMap<u64, Properties<PhysicalMaterial>>,

    /// All meshes that are use for emisive rendering
    emissive_meshes: std::collections::HashMap<u64, Properties<ColorMaterial>>,

    /// All meshes that will be rendered with a fence material
    fence_meshes: std::collections::HashMap<u64, Properties<ColorMaterial>>,

    /// element to draw the selection boxes without any shading on the lines.
    select_boxes: InstancedEntity<ColorMaterial>,

    /// Grid.
    grid: InstancedEntity<ColorMaterial>,

    /// Tracked effects that are carried over to the next frame.
    effects: std::collections::HashMap<u64, Box<dyn RenderableEffect>>,
}

impl ConstructRender {
    pub fn new(context: &Context) -> Self {
        let mut static_geometries = vec![];

        // Ground plane
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
        static_geometries.push(ground_plane);

        // Grid lines
        let mut grid = InstancedEntity::new_colored(context, &CpuMesh::cylinder(4));
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

        for (p0, p1, width, c) in lines {
            grid.add_line(p0, p1, width, c);
        }
        grid.update_instances();

        let select_boxes = InstancedEntity::new_colored(context, &CpuMesh::cylinder(4));

        ConstructRender {
            static_geometries,
            grid,
            pbr_meshes: Default::default(),
            emissive_meshes: Default::default(),
            effects: Default::default(),
            fence_meshes: Default::default(),
            select_boxes,
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

    /// Return a list of geometries to be used for shadow calculations.
    pub fn shadow_meshes(&self) -> Vec<&impl Geometry> {
        self.pbr_meshes
            .values()
            .filter(|p| p.cast_shadow)
            .map(|x| &x.object.gm().geometry)
            .collect::<_>()
    }

    pub fn non_emissive_meshes(&self) -> Vec<&dyn Object> {
        let mut meshes: Vec<&dyn Object> = vec![];
        meshes.push(self.grid.gm());
        meshes.append(
            &mut self
                .pbr_meshes
                .values()
                .filter(|p| !p.is_emissive)
                .map(|x| x.object.gm() as &dyn Object)
                .collect::<_>(),
        );
        meshes.append(
            &mut self
                .static_geometries
                .iter()
                .map(|x| x as &dyn Object)
                .collect::<_>(),
        );
        meshes.append(
            &mut self
                .effects
                .iter()
                .filter_map(|v| v.1.object())
                .collect::<Vec<_>>(),
        );
        meshes
    }

    pub fn emissive_objects(&self) -> Vec<&dyn Object> {
        self.emissive_meshes
            .values()
            .map(|x| x.object.gm() as &dyn Object)
            .collect::<_>()
    }

    pub fn fence_objects(&self) -> Vec<&dyn Object> {
        self.fence_meshes
            .values()
            .map(|x| x.object.gm() as &dyn Object)
            .collect::<_>()
    }

    /// Return the objects to be rendered.
    pub fn objects(&self) -> Vec<&dyn Object> {
        let mut renderables: Vec<&dyn Object> = vec![];
        renderables.push(self.grid.gm());
        renderables.push(self.select_boxes.object());
        renderables.append(
            &mut self
                .pbr_meshes
                .values()
                .map(|x| x.object.gm() as &dyn Object)
                .collect::<Vec<&dyn Object>>(),
        );
        renderables.append(
            &mut self
                .static_geometries
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
        for instance_entity in self.pbr_meshes.values_mut() {
            instance_entity.object.update_instances();
        }
        for instance_entity in self.emissive_meshes.values_mut() {
            instance_entity.object.update_instances();
        }
        for instance_entity in self.fence_meshes.values_mut() {
            instance_entity.object.update_instances();
        }
        self.select_boxes.update_instances();
    }

    fn reset_instances(&mut self) {
        self.pbr_meshes.clear();
        self.emissive_meshes.clear();
        self.fence_meshes.clear();
        self.select_boxes.clear();
    }

    fn draw_select_boxes(&mut self, construct: &Construct, selected: &[EntityId]) {
        use battleground_construct::util::cgmath::prelude::*;
        let boxes = &mut self.select_boxes;
        let d = 0.01;
        let c = Color::WHITE;

        for e in selected.iter() {
            if let Some(b) =
                construct
                    .world()
                    .component::<battleground_construct::components::select_box::SelectBox>(*e)
            {
                let world_pose = construct.entity_pose(*e);
                let w = (b.width() + (b.width() * 0.1).min(0.5)) / 2.0;
                let l = (b.length() + (b.length() * 0.1).min(0.5)) / 2.0;
                let h = (b.height() + (b.height() * 0.1).min(0.5)) / 2.0;
                let t = |p: Vec3| (world_pose.transform() * p.to_h()).to_translation();
                let pt = |x: f32, y: f32, z: f32| t(vec3(x, y, z));
                let points = [
                    pt(l, w, h),    // 0
                    pt(l, w, -h),   // 1
                    pt(l, -w, -h),  // 2
                    pt(l, -w, h),   // 3
                    pt(-l, w, h),   // 4
                    pt(-l, w, -h),  // 5
                    pt(-l, -w, -h), // 6
                    pt(-l, -w, h),  // 7
                ];

                boxes.add_line(points[0], points[1], d, c);
                boxes.add_line(points[1], points[2], d, c);
                boxes.add_line(points[2], points[3], d, c);
                boxes.add_line(points[4], points[0], d, c);

                boxes.add_line(points[0], points[3], d, c);
                boxes.add_line(points[1], points[5], d, c);
                boxes.add_line(points[2], points[6], d, c);
                boxes.add_line(points[3], points[7], d, c);

                boxes.add_line(points[4], points[5], d, c);
                boxes.add_line(points[5], points[6], d, c);
                boxes.add_line(points[6], points[7], d, c);
                boxes.add_line(points[7], points[4], d, c);
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
        self.reset_instances();

        self.draw_select_boxes(
            construct,
            &selected.iter().copied().collect::<Vec<EntityId>>(),
        );

        let units = Self::selected_to_units(construct, &selected);

        // Iterate through all displayables to collect meshes
        self.component_to_meshes::<display::artillery_turret::ArtilleryTurret>(context, construct);
        self.component_to_meshes::<display::artillery_barrel::ArtilleryBarrel>(context, construct);
        self.component_to_meshes::<display::artillery_body::ArtilleryBody>(context, construct);

        self.component_to_meshes::<display::tracks_side::TracksSide>(context, construct);

        self.component_to_meshes::<display::tank_body::TankBody>(context, construct);
        self.component_to_meshes::<display::tank_turret::TankTurret>(context, construct);
        self.component_to_meshes::<display::tank_barrel::TankBarrel>(context, construct);
        self.component_to_meshes::<display::tank_bullet::TankBullet>(context, construct);

        self.component_to_meshes::<display::radar_model::RadarModel>(context, construct);

        // We could also pre-calculate all entities that have the correct unit members, and then
        // filter based on that...
        self.component_to_meshes_filtered::<display::draw_module::DrawComponent, _>(
            context,
            construct,
            |e| {
                construct
                    .world()
                    .component::<battleground_construct::components::unit_member::UnitMember>(e)
                    .map(|v| units.contains(&v.unit()))
                    .unwrap_or(false)
            },
        );

        self.component_to_meshes::<display::debug_box::DebugBox>(context, construct);
        self.component_to_meshes::<display::debug_sphere::DebugSphere>(context, construct);
        self.component_to_meshes::<display::debug_lines::DebugLines>(context, construct);
        self.component_to_meshes::<display::debug_elements::DebugElements>(context, construct);

        self.component_to_meshes::<display::flag::Flag>(context, construct);
        self.component_to_meshes::<display::display_control_point::DisplayControlPoint>(
            context, construct,
        );

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
    fn component_to_meshes_filtered<C: Component + Drawable + 'static, F: Fn(EntityId) -> bool>(
        &mut self,
        context: &Context,
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
                self.add_primitive_element(context, &el, world_pose.transform())
            }
        }
    }

    /// Function to iterate over the components and convert their drawables into elements.
    fn component_to_meshes<C: Component + Drawable + 'static>(
        &mut self,
        context: &Context,
        construct: &Construct,
    ) {
        self.component_to_meshes_filtered::<C, _>(context, construct, |_| true);
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
        context: &Context,
        el: &display::primitives::Element,
        entity_transform: &Matrix4<f32>,
    ) {
        match el.material {
            battleground_construct::display::primitives::Material::FlatMaterial(flat_material) => {
                self.add_primitive_element_pbr(context, el, entity_transform);
                if flat_material.is_emissive {
                    self.add_primitive_element_emissive(context, el, entity_transform);
                }
            }
            battleground_construct::display::primitives::Material::FenceMaterial(_) => {
                self.add_primitive_element_fence(context, el, entity_transform);
            }
        }
    }

    // All functions in the add_primitive_element_* family resemble each other way too much...
    fn add_primitive_element_pbr(
        &mut self,
        context: &Context,
        el: &display::primitives::Element,
        entity_transform: &Matrix4<f32>,
    ) {
        // Add the elements to the pbr_meshes
        let instanced = &mut self
            .pbr_meshes
            .entry(el.to_draw_key())
            .or_insert_with(|| {
                let primitive_mesh = Self::primitive_to_mesh(el);

                let cast_shadow = match el.primitive {
                    display::primitives::Primitive::Line(_) => false,
                    _ => true,
                };

                let is_emissive = match el.material {
                    battleground_construct::display::primitives::Material::FlatMaterial(
                        flat_material,
                    ) => flat_material.is_emissive,
                    battleground_construct::display::primitives::Material::FenceMaterial(_) => {
                        unimplemented!()
                    }
                };

                // Need to make the appropriate material here based on the material passed in.
                let material =
                    if let battleground_construct::display::primitives::Material::FlatMaterial(
                        flat_material,
                    ) = el.material
                    {
                        let emissive = if flat_material.is_emissive {
                            flat_material.emissive.to_color()
                        } else {
                            Color::BLACK
                        };
                        let fun = if flat_material.is_transparent {
                            three_d::renderer::material::PhysicalMaterial::new_transparent
                        } else {
                            three_d::renderer::material::PhysicalMaterial::new_opaque
                        };
                        fun(
                            context,
                            &CpuMaterial {
                                albedo: Color {
                                    r: 255,
                                    g: 255,
                                    b: 255,
                                    a: 255,
                                },
                                emissive,
                                ..Default::default()
                            },
                        )
                    } else {
                        panic!("unsupported material");
                    };

                Properties {
                    object: InstancedEntity::new(context, &primitive_mesh, material),
                    cast_shadow,
                    is_emissive,
                }
            })
            .object;
        let transform = entity_transform * el.transform;
        // At some point, we have to handle different material types here.
        let material = if let display::primitives::Material::FlatMaterial(material) = el.material {
            material
        } else {
            panic!("unsupported material");
        };
        let color = material.color.to_color();

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
        };
    }

    fn add_primitive_element_emissive(
        &mut self,
        context: &Context,
        el: &display::primitives::Element,
        entity_transform: &Matrix4<f32>,
    ) {
        // Add the elements to the emissive meshes
        let instanced = &mut self
            .emissive_meshes
            .entry(el.to_draw_key())
            .or_insert_with(|| {
                let primitive_mesh = Self::primitive_to_mesh(el);

                // Need to make the appropriate material here based on the material passed in.
                let material =
                    if let battleground_construct::display::primitives::Material::FlatMaterial(
                        flat_material,
                    ) = el.material
                    {
                        let fun = if flat_material.is_transparent {
                            three_d::renderer::material::ColorMaterial::new_transparent
                        } else {
                            three_d::renderer::material::ColorMaterial::new_opaque
                        };
                        fun(
                            context,
                            &CpuMaterial {
                                albedo: Color {
                                    r: 255,
                                    g: 255,
                                    b: 255,
                                    a: 255,
                                },
                                ..Default::default()
                            },
                        )
                    } else {
                        panic!("unsupported material");
                    };

                Properties {
                    object: InstancedEntity::new(context, &primitive_mesh, material),
                    cast_shadow: false,
                    is_emissive: true,
                }
            })
            .object;

        let transform = entity_transform * el.transform;
        // At some point, we have to handle different material types here.
        let material = if let display::primitives::Material::FlatMaterial(material) = el.material {
            material
        } else {
            panic!("unsupported material");
        };
        let color = material.emissive.to_color();

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
        };
    }

    fn add_primitive_element_fence(
        &mut self,
        context: &Context,
        el: &display::primitives::Element,
        entity_transform: &Matrix4<f32>,
    ) {
        // Add the elements to the emissive meshes
        let instanced = &mut self
            .fence_meshes
            .entry(el.to_draw_key())
            .or_insert_with(|| {
                let primitive_mesh = Self::primitive_to_mesh(el);
                Properties {
                    object: InstancedEntity::new(
                        context,
                        &primitive_mesh,
                        ColorMaterial::new(
                            context,
                            &CpuMaterial {
                                albedo: Color {
                                    r: 255,
                                    g: 255,
                                    b: 255,
                                    a: 255,
                                },
                                ..Default::default()
                            },
                        ),
                    ),
                    cast_shadow: false,
                    is_emissive: false,
                }
            })
            .object;

        let transform = entity_transform * el.transform;
        // At some point, we have to handle different material types here.
        let material = if let display::primitives::Material::FenceMaterial(material) = el.material {
            material
        } else {
            panic!("unsupported material");
        };
        let color = material.color.to_color();

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
        };
    }

    fn primitive_to_mesh(el: &display::primitives::Element) -> CpuMesh {
        match el.primitive {
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
                let mut m = CpuMesh::sphere(32);
                m.transform(&Mat4::from_scale(sphere.radius)).unwrap();
                m
            }
            display::primitives::Primitive::Cylinder(cylinder) => {
                let mut m = CpuMesh::cylinder(128);
                m.transform(&Mat4::from_nonuniform_scale(
                    cylinder.height,
                    cylinder.radius,
                    cylinder.radius,
                ))
                .unwrap();
                m
            }
            display::primitives::Primitive::Cone(cone) => {
                let mut m = CpuMesh::cone(128);
                m.transform(&Mat4::from_nonuniform_scale(
                    cone.height,
                    cone.radius,
                    cone.radius,
                ))
                .unwrap();
                m
            }
            display::primitives::Primitive::Line(_line) => CpuMesh::cylinder(4),
            display::primitives::Primitive::Circle(circle) => {
                let mut m = CpuMesh::circle(128);
                m.transform(&Mat4::from_scale(circle.radius)).unwrap();
                m
            }
        }
    }
}
