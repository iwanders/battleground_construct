use super::hit_box::HitBox;
use crate::display::primitives::Mat4;
use crate::display::primitives::Vec3;
use crate::util::box_collision::AxisAlignedBox;
use crate::util::cgmath::prelude::*;
use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct HitCollection {
    hit_boxes: Vec<(Mat4, HitBox)>,
}

impl Default for HitCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl HitCollection {
    pub fn new() -> Self {
        HitCollection { hit_boxes: vec![] }
    }

    pub fn from_hit_boxes(hit_boxes: &[(Mat4, HitBox)]) -> Self {
        HitCollection {
            hit_boxes: hit_boxes.iter().map(|(t, b)| (*t, *b)).collect(),
        }
    }
    pub fn from_hit_box(hit_box: HitBox) -> Self {
        use cgmath::SquareMatrix;
        HitCollection {
            hit_boxes: vec![(Mat4::identity(), hit_box)],
        }
    }

    pub fn is_inside(&self, collection_transform: Mat4, point: Vec3) -> bool {
        // convert the projectile pose into the hitbox's local frame.
        // currently, projectile_pose is world -> projectile.
        //            hitbox_pose is world -> hitbox.
        // hitbox -> world -> world -> projectile.
        let projectile_pose = Mat4::from_translation(point);
        for (hitbox_transform, hitbox) in self.hit_boxes.iter() {
            let hitbox_pose = collection_transform * hitbox_transform;
            let point_in_hitbox_frame = hitbox_pose.to_inv_h() * projectile_pose;
            let b = AxisAlignedBox::new(hitbox.length(), hitbox.width(), hitbox.height());
            let inside = b.is_inside(point_in_hitbox_frame.to_translation());
            if inside {
                return true;
            }
        }
        false
    }

    pub fn distance_to(&self, collection_transform: Mat4, point: Vec3) -> f32 {
        let mut distance = f32::MAX;
        let projectile_pose = Mat4::from_translation(point);
        for (hitbox_transform, hitbox) in self.hit_boxes.iter() {
            let hitbox_pose = collection_transform * hitbox_transform;
            // Express the impact pose into the hitbox frame.
            let point_in_hitbox_frame = hitbox_pose.to_inv_h() * projectile_pose;
            let b = AxisAlignedBox::new(hitbox.length(), hitbox.width(), hitbox.height());

            // Clamp the impact pose to be within the box.
            let clamped_point = b.clamp_point(point_in_hitbox_frame.to_translation());

            // Now, the distance between the clamped and point is the distance from the
            // impact to the nearest point inside the hitbox.
            distance = (clamped_point - point_in_hitbox_frame.to_translation())
                .euclid_norm()
                .min(distance);
        }
        distance
    }

    pub fn hit_boxes(&self) -> &[(Mat4, HitBox)] {
        &self.hit_boxes[..]
    }
}
impl Component for HitCollection {}
