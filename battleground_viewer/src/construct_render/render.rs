use three_d::*;

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

// TODO: Rename this (it's a key for looking up batches, not drawbles)
trait DrawableKey {
    fn to_draw_key(&self) -> u64;
}

impl DrawableKey for Primitive {
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
struct PrimitiveBatchKey {
    primitive: Primitive,
    properties: BatchProperties,
}

impl PrimitiveBatchKey {
    fn new(primitive: Primitive, properties: BatchProperties) -> Self {
        Self {
            primitive,
            properties,
        }
    }
}

impl DrawableKey for PrimitiveBatchKey {
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
    fn finish_frame(&mut self, context: &Context);
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

    fn finish_frame(&mut self, context: &Context) {
    }
}


struct PrimitiveBatch {
    key: PrimitiveBatchKey,
    transforms: Vec<Mat4>,
    colors: Vec<Color>,
}

pub struct PrimitiveGeometry<M: Material + BatchMaterial> {
    participates_in_pass: fn(RenderPass) -> bool,

    // TODO: Move InstancedEntity code into here so we can drop InstancedEntity, and build all buffers in one go
    batches: std::collections::HashMap<u64, PrimitiveBatch>,

    /// The meshes produced from the baches
    meshes: Vec<Gm<InstancedMesh, M>>,
}

impl<M: Material + BatchMaterial> PrimitiveGeometry<M> {
    pub fn new(participates_in_pass: fn(RenderPass) -> bool) -> Self {
        Self {
            participates_in_pass,
            batches: Default::default(),
            meshes: Default::default(),
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

    pub fn add_primitive(
        &mut self,
        context: &Context,
        batch_hints: BatchProperties,
        primitive: Primitive,
        transform: Mat4,
        color: Color,
    ) {
        let key = PrimitiveBatchKey::new(primitive, batch_hints);
        let batch = &mut self
            .batches
            .entry(key.to_draw_key())
            .or_insert_with(|| {
                PrimitiveBatch {
                    key,
                    transforms: Default::default(),
                    colors: Default::default(),
                }
            });

        batch.transforms.push(match primitive {
            Primitive::Line(l) => {
                use battleground_construct::util::cgmath::ToHomogenous;
                use battleground_construct::util::cgmath::ToTranslation;
                let p0_original = vec3(l.p0.0, l.p0.1, l.p0.2);
                let p1_original = vec3(l.p1.0, l.p1.1, l.p1.2);
                let p0 = (transform * p0_original.to_h()).to_translation();
                let p1 = (transform * p1_original.to_h()).to_translation();
                let rotation = Quat::from_arc(vec3(1.0, 0.0, 0.0), (p1 - p0).normalize(), None);
                let scale = Mat4::from_nonuniform_scale((p0 - p1).magnitude(), l.width / 2.0, l.width / 2.0);
                Mat4::from_translation(p0) * <_ as Into<Mat4>>::into(rotation) * scale
            }
            _ => {
                transform
            }
        });
        batch.colors.push(color);
    }
}

impl<M: Material + BatchMaterial> RenderableGeometry for PrimitiveGeometry<M> {
    fn objects(&self, pass: RenderPass) -> Option<Vec<&dyn Object>> {
        self.meshes_for_pass(pass)
            .map(|xs| xs.map(|x| x as &dyn Object).collect::<_>())
    }

    fn geometries(&self, pass: RenderPass) -> Option<Vec<&InstancedMesh>> {
        self.meshes_for_pass(pass)
            .map(|xs| xs.map(|x| &x.geometry).collect::<_>())
    }

    fn prepare_frame(&mut self) {
        self.batches.clear();
        self.meshes.clear();
    }

    fn finish_frame(&mut self, context: &Context) {
        for batch in self.batches.values() {
            let cpu_mesh = primitive_to_mesh(&batch.key.primitive);
            let material = M::new_for_batch(context, batch.key.properties);
            let instances = three_d::renderer::geometry::Instances {
                transformations: batch.transforms.clone(),
                colors: Some(batch.colors.clone()),
                ..Default::default()
            };
            let gm = Gm::new(InstancedMesh::new(context, &instances, &cpu_mesh), material);
            self.meshes.push(gm);
        }
    }
}

fn primitive_to_mesh(primitive: &Primitive) -> CpuMesh {
    match primitive {
        Primitive::Cuboid(cuboid) => {
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
        Primitive::Sphere(sphere) => {
            let mut m = CpuMesh::sphere(32);
            m.transform(&Mat4::from_scale(sphere.radius)).unwrap();
            m
        }
        Primitive::Cylinder(cylinder) => {
            let mut m = CpuMesh::cylinder(128);
            m.transform(&Mat4::from_nonuniform_scale(
                cylinder.height,
                cylinder.radius,
                cylinder.radius,
            ))
            .unwrap();
            m
        }
        Primitive::Cone(cone) => {
            let mut m = CpuMesh::cone(128);
            m.transform(&Mat4::from_nonuniform_scale(
                cone.height,
                cone.radius,
                cone.radius,
            ))
            .unwrap();
            m
        }
        Primitive::Line(_line) => CpuMesh::cylinder(4),
        Primitive::Circle(circle) => {
            let mut m = CpuMesh::circle(128);
            m.transform(&Mat4::from_scale(circle.radius)).unwrap();
            m
        }
    }
}
