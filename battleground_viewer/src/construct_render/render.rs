use three_d::*;

use super::instanced_entity::InstancedEntity;

use battleground_construct::display;
use battleground_construct::display::primitives::Primitive;

// Brend: This render pass enumeration omits non-geometry passes (such as the bloom filter application, or the gui render).
#[derive(Debug, Copy, Clone)]
pub enum RenderPass {
    ShadowMaps,
    BaseScene,
    NonEmmisivesDepth,
    Emmisives,
    Fences,
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

#[derive(Debug, Copy, Clone)]
pub enum BatchProperties {
    None,
    Basic { is_transparent: bool },
}

impl DrawableKey for BatchProperties {
    fn to_draw_key(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let state = &mut hasher;
        match *self {
            BatchProperties::None => {
                1usize.hash(state);
            }
            BatchProperties::Basic { is_transparent } => {
                2usize.hash(state);
                is_transparent.hash(state);
            }
        }
        hasher.finish()
    }
}

#[derive(Debug, Copy, Clone)]
struct BatchablePrimitive {
    primitive: Primitive,
    properties: BatchProperties,
}

impl BatchablePrimitive {
    fn new(primitive: Primitive, properties: BatchProperties) -> Self {
        Self {
            primitive,
            properties,
        }
    }
}

impl DrawableKey for BatchablePrimitive {
    fn to_draw_key(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let state = &mut hasher;
        self.primitive.to_draw_key().hash(state);
        self.properties.to_draw_key().hash(state);
        hasher.finish()
    }
}

pub trait RenderableGeometry {
    /// Produces the objects to render for this render pass
    fn objects(&self, pass: RenderPass) -> Option<Vec<&dyn Object>>;

    /// Produces the geometries for this render pass.
    fn geometries(&self, pass: RenderPass) -> Option<Vec<&InstancedMesh>>;

    fn prepare_frame(&mut self);
    fn finish_frame(&mut self);
}

pub trait BatchMaterial {
    fn new_for_batch(context: &Context, batch_properties: BatchProperties) -> Self
    where
        Self: Material;
}

impl BatchMaterial for PhysicalMaterial {
    fn new_for_batch(context: &Context, batch_properties: BatchProperties) -> PhysicalMaterial {
        let material_new = match batch_properties {
            BatchProperties::None => PhysicalMaterial::new_opaque,
            BatchProperties::Basic {
                is_transparent: false,
            } => PhysicalMaterial::new_opaque,
            BatchProperties::Basic {
                is_transparent: true,
            } => PhysicalMaterial::new_transparent,
        };
        material_new(
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
    }
}

impl BatchMaterial for ColorMaterial {
    fn new_for_batch(context: &Context, batch_properties: BatchProperties) -> ColorMaterial {
        let material_new = match batch_properties {
            BatchProperties::None => ColorMaterial::new_opaque,
            BatchProperties::Basic {
                is_transparent: false,
            } => ColorMaterial::new_opaque,
            BatchProperties::Basic {
                is_transparent: true,
            } => ColorMaterial::new_transparent,
        };
        material_new(
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
    }
}

pub struct MeshGeometry<M: Material + BatchMaterial> {
    participates_in_pass: fn(RenderPass) -> bool,
    meshes: Vec<Gm<InstancedMesh, M>>,
}

impl<M: Material + BatchMaterial> MeshGeometry<M> {
    pub fn new(participates_in_pass: fn(RenderPass) -> bool) -> Self {
        Self {
            participates_in_pass,
            meshes: Default::default()
        }
    }

    fn meshes_for_pass(
        &self,
        pass: RenderPass,
    ) -> Option<impl Iterator<Item = &Gm<InstancedMesh, M>>> {
        if (self.participates_in_pass)(pass) {
            Some(self.meshes.iter())
        } else {
            None
        }
    }

    pub fn add_mesh(&mut self, context: &Context, batch_hints: BatchProperties, mesh: &CpuMesh, transform: Mat4, color: Color) {
        let instanced = Gm::new(
            InstancedMesh::new(
                context,
                &Instances {
                    transformations: vec![transform],
                    colors: Some(vec![color]),
                    ..Default::default()
                },
                mesh,
            ),
            M::new_for_batch(context, batch_hints)
        );
        self.meshes.push(instanced);
    }
}

impl<M: Material + BatchMaterial> RenderableGeometry for MeshGeometry<M> {
    fn objects(&self, pass: RenderPass) -> Option<Vec<&dyn Object>> {
        self.meshes_for_pass(pass)
            .map(|xs| xs.map(|x| x as &dyn Object).collect::<_>())
    }

    fn geometries(&self, pass: RenderPass) -> Option<Vec<&InstancedMesh>> {
        self.meshes_for_pass(pass)
            .map(|xs| xs.map(|x| &x.geometry).collect::<_>())
    }

    fn prepare_frame(&mut self) {
    }

    fn finish_frame(&mut self) {
    }
}


pub struct PrimitiveGeometry<M: Material + BatchMaterial> {
    participates_in_pass: fn(RenderPass) -> bool,
    /// The meshes in this physical geometry container (keyed on drawable key to batch up geometries that are the same into instanced entities)
    meshes: std::collections::HashMap<u64, InstancedEntity<M>>,
}

impl<M: Material + BatchMaterial> PrimitiveGeometry<M> {
    pub fn new(participates_in_pass: fn(RenderPass) -> bool) -> Self {
        Self {
            participates_in_pass,
            meshes: Default::default(),
        }
    }

    fn meshes_for_pass(
        &self,
        pass: RenderPass,
    ) -> Option<impl Iterator<Item = &InstancedEntity<M>>> {
        if (self.participates_in_pass)(pass) {
            Some::<std::collections::hash_map::Values<'_, u64, InstancedEntity<M>>>(
                self.meshes.values().into(),
            )
        } else {
            None
        }
    }

    pub fn add_primitive(
        &mut self,
        context: &Context,
        batch_hints: BatchProperties,
        primitive: display::primitives::Primitive,
        transform: Mat4,
        color: Color,
    ) {
        // Add the elements to the pbr_meshes
        let is_transparent = color.a < 255;
        let batch_key = BatchablePrimitive::new(primitive, batch_hints);

        let instanced = &mut self
            .meshes
            .entry(batch_key.to_draw_key())
            .or_insert_with(|| {
                let primitive_mesh = primitive_to_mesh(&primitive);
                let material = M::new_for_batch(context, batch_hints);
                InstancedEntity::new(context, &primitive_mesh, material)
            });

        // Special handling for lines...
        match primitive {
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
}

impl<M: Material + BatchMaterial> RenderableGeometry for PrimitiveGeometry<M> {
    fn objects(&self, pass: RenderPass) -> Option<Vec<&dyn Object>> {
        self.meshes_for_pass(pass)
            .map(|xs| xs.map(|x| x.object()).collect::<_>())
    }

    fn geometries(&self, pass: RenderPass) -> Option<Vec<&InstancedMesh>> {
        self.meshes_for_pass(pass)
            .map(|xs| xs.map(|x| &x.gm().geometry).collect::<_>())
    }

    fn prepare_frame(&mut self) {
        self.meshes.clear();
    }

    fn finish_frame(&mut self) {
        for instance_entity in self.meshes.values_mut() {
            instance_entity.update_instances();
        }
    }
}

fn primitive_to_mesh(primitive: &display::primitives::Primitive) -> CpuMesh {
    match primitive {
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
