use three_d::*;

use super::batching::*;
use super::*;

use battleground_construct::display::primitives::Primitive;

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

impl BatchKey for PrimitiveBatchKey {
    fn to_batch_key(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let state = &mut hasher;
        self.primitive.to_batch_key().hash(state);
        self.properties.to_batch_key().hash(state);
        hasher.finish()
    }
}

struct PrimitiveBatch {
    key: PrimitiveBatchKey,
    transforms: Vec<Mat4>,
    colors: Vec<Color>,
}

/// A geometry buffer that accepts `Primitive`s
pub struct PrimitiveGeometry<M: Material + BatchMaterial> {
    participates_in_pass: fn(RenderPass) -> bool,

    /// Batches of primitives with the same properties
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

    pub fn add_primitive(
        &mut self,
        batch_hints: BatchProperties,
        primitive: Primitive,
        transform: Mat4,
        color: Color,
    ) {
        let key = PrimitiveBatchKey::new(primitive, batch_hints);
        let batch = &mut self
            .batches
            .entry(key.to_batch_key())
            .or_insert_with(|| PrimitiveBatch {
                key,
                transforms: Default::default(),
                colors: Default::default(),
            });

        // Some primitives have special transform handling
        batch
            .transforms
            .push(primitive_transform(&primitive, &transform));
        batch.colors.push(color);
    }
}

impl<M: Material + BatchMaterial> RenderableGeometry for PrimitiveGeometry<M> {
    fn objects(&self, pass: RenderPass) -> Vec<&dyn Object> {
        if (self.participates_in_pass)(pass) {
            self.meshes.iter().map(|x| x as &dyn Object).collect()
        } else {
            vec![]
        }
    }

    fn geometries(&self, pass: RenderPass) -> Vec<GeometryRef> {
        if (self.participates_in_pass)(pass) {
            self.meshes
                .iter()
                .map(|x| GeometryRef::InstancedMesh(&x.geometry))
                .collect()
        } else {
            vec![]
        }
    }

    fn prepare_scene(&mut self, _context: &Context) {
        self.batches.clear();
        self.meshes.clear();
    }

    fn finish_scene(&mut self, context: &Context) {
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

/// Produce meshes for primitives
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
        Primitive::ExtrudedRectangle(_extruded_rectangle) => {
            let mut m = CpuMesh::cube();
            m.transform(&Mat4::from_nonuniform_scale(
                1.0 / 2.0,
                1.0 / 2.0,
                1.0 / 2.0,
            ))
            .unwrap();
            m
        }
    }
}

/// Special transform handling for primitives that need it
fn primitive_transform(primitive: &Primitive, transform: &Mat4) -> Mat4 {
    match primitive {
        Primitive::Line(l) => {
            use battleground_construct::util::cgmath::ToHomogenous;
            use battleground_construct::util::cgmath::ToTranslation;
            let p0_original = vec3(l.p0.0, l.p0.1, l.p0.2);
            let p1_original = vec3(l.p1.0, l.p1.1, l.p1.2);
            let p0 = (transform * p0_original.to_h()).to_translation();
            let p1 = (transform * p1_original.to_h()).to_translation();
            let rotation = Quat::from_arc(vec3(1.0, 0.0, 0.0), (p1 - p0).normalize(), None);
            // zero out the roll, we should only need pitch and yaw.
            use battleground_construct::util::cgmath::ToRollPitchYaw;
            use battleground_construct::util::cgmath::RollPitchYawToHomogenous;

            let mut rpy = <_ as Into<Mat4>>::into(rotation).to_rpy();
            rpy.x = 0.0;
            let rotation = rpy.rpy_to_h();
            let scale =
                Mat4::from_nonuniform_scale((p0 - p1).magnitude(), l.width / 2.0, l.width / 2.0);
            Mat4::from_translation(p0) * rotation * scale
        }
        Primitive::ExtrudedRectangle(extruded_rectangle) => {
            let local_offset = vec3(extruded_rectangle.length / 2.0, 0.0, 0.0);
            let scale = Mat4::from_nonuniform_scale(
                extruded_rectangle.length,
                extruded_rectangle.width,
                extruded_rectangle.height,
            );
            transform * Mat4::from_translation(local_offset) * scale
        }
        _ => *transform,
    }
}
