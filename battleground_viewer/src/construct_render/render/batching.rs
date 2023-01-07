use three_d::*;

use battleground_construct::display::primitives::Primitive;

pub trait BatchKey {
    fn to_batch_key(&self) -> u64;
}

impl BatchKey for Primitive {
    fn to_batch_key(&self) -> u64 {
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
            Primitive::ExtrudedRectangle(_extruded_rectangle) => {
                // All extruded rectangles hash the same.
                6usize.hash(state);
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

impl BatchKey for BatchProperties {
    fn to_batch_key(&self) -> u64 {
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

/// Produces materials based on batch property
pub trait BatchMaterial
where
    Self: Material,
{
    /// Produce a material for the given batch properties with unbiased color
    fn new_for_batch(context: &Context, batch_properties: BatchProperties) -> Self;

    /// Produce a material for the given batch properties for biased for the given color
    fn new_for_batch_colored(
        context: &Context,
        batch_properties: BatchProperties,
        color: Color,
    ) -> Self;
}

impl BatchMaterial for PhysicalMaterial {
    fn new_for_batch(context: &Context, batch_properties: BatchProperties) -> PhysicalMaterial {
        Self::new_for_batch_colored(context, batch_properties, Color::WHITE)
    }

    fn new_for_batch_colored(
        context: &Context,
        batch_properties: BatchProperties,
        color: Color,
    ) -> PhysicalMaterial {
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
                albedo: color,
                ..Default::default()
            },
        )
    }
}

impl BatchMaterial for ColorMaterial {
    fn new_for_batch(context: &Context, batch_properties: BatchProperties) -> ColorMaterial {
        Self::new_for_batch_colored(context, batch_properties, Color::WHITE)
    }

    fn new_for_batch_colored(
        context: &Context,
        batch_properties: BatchProperties,
        color: Color,
    ) -> ColorMaterial {
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
                albedo: color,
                ..Default::default()
            },
        )
    }
}
