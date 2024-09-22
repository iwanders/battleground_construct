use cgmath::{BaseNum, Vector3};
use serde::{Serialize, Deserialize};

/// Generic AxisAlignedBox of given dimensions. AxisAlignedBox is centered around the origin.
/// Technicallly a RectangularAxisAlignedBox.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxisAlignedBox<S: BaseNum> {
    pub x: S, // x
    pub y: S, // y
    pub z: S, // z
}

impl<S: BaseNum + std::fmt::Display> AxisAlignedBox<S> {
    #[inline]
    pub const fn new(length: S, width: S, height: S) -> AxisAlignedBox<S> {
        AxisAlignedBox {
            x: length,
            y: width,
            z: height,
        }
    }

    // Use a < b, NOT a <= b, the latter doesn't optimise to a single instruction.
    fn min(a: S, b: S) -> S {
        if a < b {
            a
        } else {
            b
        }
    }

    fn max(a: S, b: S) -> S {
        if a > b {
            a
        } else {
            b
        }
    }

    fn clamp(v: S, lower: S, upper: S) -> S {
        Self::max(Self::min(v, upper), lower)
    }

    #[inline]
    pub fn min_point(&self) -> Vector3<S> {
        let zero = S::zero();
        self.parametrized_point(zero, zero, zero)
    }

    #[inline]
    pub fn max_point(&self) -> Vector3<S> {
        let one = S::one();
        self.parametrized_point(one, one, one)
    }

    /// Clamp the provided point to be within the axis aligned box.
    pub fn clamp_point(&self, point: Vector3<S>) -> Vector3<S> {
        let min = self.min_point();
        let max = self.max_point();
        Vector3::<S>::new(
            Self::clamp(point.x, min.x, max.x),
            Self::clamp(point.y, min.y, max.y),
            Self::clamp(point.z, min.z, max.z),
        )
    }

    fn parametrized_point(&self, u: S, v: S, w: S) -> Vector3<S> {
        let one = S::one();
        let two = S::one() + S::one();
        let minus_one = one - two;
        Vector3::<S>::new(
            minus_one * (self.x / two) + self.x * u,
            minus_one * (self.y / two) + self.y * v,
            minus_one * (self.z / two) + self.z * w,
        )
    }

    fn v_x(&self) -> Vector3<S> {
        let zero = S::zero();
        Vector3::<S>::new(self.x, zero, zero)
    }

    fn v_y(&self) -> Vector3<S> {
        let zero = S::zero();
        Vector3::<S>::new(zero, self.y, zero)
    }

    fn v_z(&self) -> Vector3<S> {
        let zero = S::zero();
        Vector3::<S>::new(zero, zero, self.z)
    }

    /// Check if the vector is inside the box.
    pub fn is_inside(&self, x: Vector3<S>) -> bool {
        // https://math.stackexchange.com/a/1472080
        /*
            The three important directions are u=P1-P2, v=P1-P4 and w=P1-P5.
            They are three perpendicular edges of the rectangular box.
            A point x lies within the box when the three following constraints are respected:
                - The dot product u.x is between u.P1 and u.P2
                - The dot product v.x is between v.P1 and v.P4
                - The dot product w.x is between w.P1 and w.P5
        */
        use cgmath::InnerSpace;

        let zero = S::zero();
        let one = S::one();

        // Three directions
        let u_x = self.v_x().dot(x);
        let u_y = self.v_y().dot(x);
        let u_z = self.v_z().dot(x);

        // Compare dot products with corners
        let dim1 = self.v_x().dot(self.parametrized_point(zero, zero, zero)) <= u_x
            && u_x <= self.v_x().dot(self.parametrized_point(one, zero, zero));
        let dim2 = self.v_y().dot(self.parametrized_point(zero, zero, zero)) <= u_y
            && u_y <= self.v_y().dot(self.parametrized_point(zero, one, zero));
        let dim3 = self.v_z().dot(self.parametrized_point(zero, zero, zero)) <= u_z
            && u_z <= self.v_z().dot(self.parametrized_point(zero, zero, one));

        // All three constraints satisfied?
        dim1 && dim2 && dim3
    }

    // https://github.com/tavianator/tavianator.com/blob/main/src/2015/ray_box_nan.md
    // has a nice implementation that's branchless.
    ///
    /// Check if a line segment is intersecting the box, returns a boolean, fastest method.
    ///
    pub fn is_intersecting(&self, p0: Vector3<S>, p1: Vector3<S>) -> bool {
        let one = S::one();
        let zero = S::zero();
        let (tmin, tmax) = self.is_intersecting_worker(p0, p1);
        tmax > Self::max(tmin, zero) && Self::max(tmin, zero) < one
    }

    ///
    /// Check if a line segment is intersecting the box, returning an optional that holds the
    /// parametrized intersection points, None if there are no intersections.
    ///
    pub fn intersections(&self, p0: Vector3<S>, p1: Vector3<S>) -> Option<(S, S)> {
        let one = S::one();
        let zero = S::zero();
        let (tmin, tmax) = self.is_intersecting_worker(p0, p1);
        if tmax > Self::max(tmin, zero) && Self::max(tmin, zero) < one {
            return Some((tmin, tmax));
        }
        None
    }

    #[inline]
    fn is_intersecting_worker(&self, p0: Vector3<S>, p1: Vector3<S>) -> (S, S) {
        let bmin = self.min_point();
        let bmax = self.max_point();
        let rorigin = p0;

        // Direction of the ray
        let ray_dir = p1 - p0;

        // Normalize that
        let one = S::one();
        // let zero = S::zero();
        let rdir_inv = Vector3::<S>::new(one / ray_dir.x, one / ray_dir.y, one / ray_dir.z);

        // Unroll manually such that we can avoid the infty.
        let i = 0;
        let t1 = (bmin[i] - rorigin[i]) * rdir_inv[i];
        let t2 = (bmax[i] - rorigin[i]) * rdir_inv[i];

        let mut tmin = Self::min(t1, t2);
        let mut tmax = Self::max(t1, t2);

        let i = 1;
        let t1 = (bmin[i] - rorigin[i]) * rdir_inv[i];
        let t2 = (bmax[i] - rorigin[i]) * rdir_inv[i];

        let i = 2;
        tmin = Self::max(tmin, Self::min(t1, t2));
        tmax = Self::min(tmax, Self::max(t1, t2));

        let t1 = (bmin[i] - rorigin[i]) * rdir_inv[i];
        let t2 = (bmax[i] - rorigin[i]) * rdir_inv[i];

        tmin = Self::max(tmin, Self::min(t1, t2));
        tmax = Self::min(tmax, Self::max(t1, t2));

        // println!();
        // println!("tmax: {tmax}");
        // println!("tmin: {tmin}");
        // println!("p0: {p0:?}");
        // println!("p1: {p1:?}");

        // Original:
        // return tmax > max(tmin, zero);

        // Add the condition to check that the line segment (parametrized to [0.0, 1.0]) is met.
        // tmax > max(tmin, zero) && max(tmin, zero) < one
        (tmin, tmax)
    }
}

#[cfg(test)]
mod test {
    fn verify_points<S: BaseNum + std::fmt::Display>(
        b: &AxisAlignedBox<S>,
        p0: Vector3<S>,
        p1: Vector3<S>,
        delta: S,
    ) {
        let rorigin = p0;
        // Direction of the ray
        let ray_dir = p1 - p0;
        // Normalize that
        let zero = S::zero();
        let one = S::one();

        let point_on_line = |p: S| {
            use cgmath::ElementWise;

            rorigin + ray_dir.mul_element_wise(p)
        };

        let mut t = zero;
        while t < one {
            let small_p0 = point_on_line(t);
            let small_p1 = point_on_line(t + delta);
            // let line_check1 = b.is_intersecting(small_p0, small_p1);
            let line_check2 = b.is_intersecting(small_p0, small_p1);
            let p_check = b.is_inside(small_p0) || b.is_inside(small_p1);
            // assert_eq!(p_check, line_check1, "left: {p_check}, right: {line_check1}, t: {t}");
            assert_eq!(p_check, line_check2);
            t += delta;
        }
    }

    use super::*;
    #[test]
    fn test_is_inside() {
        let b = AxisAlignedBox::new(1.0f32, 1.0, 1.0);

        // x direction.
        assert!(!b.is_inside(Vector3::new(-1.0, 0.0, 0.0)));
        assert!(b.is_inside(Vector3::new(-0.4, 0.0, 0.0)));
        assert!(b.is_inside(Vector3::new(0.0, 0.0, 0.0)));
        assert!(b.is_inside(Vector3::new(0.4, 0.0, 0.0)));
        assert!(!b.is_inside(Vector3::new(1.0, 0.0, 0.0)));

        // y direction.
        assert!(!b.is_inside(Vector3::new(0.0, -1.0, 0.0)));
        assert!(b.is_inside(Vector3::new(0.0, -0.4, 0.0)));
        assert!(b.is_inside(Vector3::new(0.0, 0.0, 0.0)));
        assert!(b.is_inside(Vector3::new(0.0, 0.4, 0.0)));
        assert!(!b.is_inside(Vector3::new(0.0, 1.0, 0.0)));

        // z direction.
        assert!(!b.is_inside(Vector3::new(0.0, 0.0, -1.0)));
        assert!(b.is_inside(Vector3::new(0.0, 0.0, -0.4)));
        assert!(b.is_inside(Vector3::new(0.0, 0.0, 0.0)));
        assert!(b.is_inside(Vector3::new(0.0, 0.0, 0.4)));
        assert!(!b.is_inside(Vector3::new(0.0, 0.0, 1.0)));

        // diagonal
        assert!(!b.is_inside(Vector3::new(-1.0, -1.0, -1.0)));
        assert!(b.is_inside(Vector3::new(-0.4, -0.4, -0.4)));
        assert!(b.is_inside(Vector3::new(0.0, 0.0, 0.0)));
        assert!(b.is_inside(Vector3::new(0.4, 0.4, 0.4)));
        assert!(!b.is_inside(Vector3::new(1.0, 1.0, 1.0)));
    }

    #[test]
    fn test_is_intersecting() {
        use cgmath::vec3;
        let b = AxisAlignedBox::new(1.0f32, 1.0, 1.0);
        // assert_eq!(b.is_intersecting(vec3(1.0, 1.0, 1.0), vec3(0.0, 0.0, 0.0), ), true);
        assert!(b.is_intersecting(vec3(1.0, 1.0, 1.0), vec3(0.0, 0.0, 0.0),));
        // assert_eq!(b.is_intersecting(vec3(1.0, 1.5, 1.0), vec3(2.0, 2.0, 2.0), ), false);
        assert!(!b.is_intersecting(vec3(1.0, 1.5, 1.0), vec3(2.0, 2.0, 2.0),));

        // pointing at box, but not into it.
        // assert_eq!(b.is_intersecting(vec3(3.0, 0.0, 0.0), vec3(5.0, 0.0, 0.0)), false);
        assert!(!b.is_intersecting(vec3(3.0, 0.0, 0.0), vec3(5.0, 0.0, 0.0)));

        verify_points(&b, vec3(1.0, 1.0, 1.0), vec3(0.0, 0.0, 0.0), 0.001);
        verify_points(&b, vec3(2.0, 0.0, 0.0), vec3(3.0, 0.0, 0.0), 0.001);
        verify_points(&b, vec3(3.0, 0.0, 0.0), vec3(2.0, 0.0, 0.0), 0.001);
        verify_points(&b, vec3(2.25, 0.0, 0.0), vec3(0.25, 0.0, 0.0), 0.0001);
        verify_points(&b, vec3(0.25, 0.0, 0.0), vec3(2.25, 0.0, 0.0), 0.01);
        verify_points(&b, vec3(-2.25, 0.0, 0.0), vec3(-0.25, 0.0, 0.0), 0.0001);
        verify_points(&b, vec3(-0.25, 0.0, 0.0), vec3(-2.25, 0.0, 0.0), 0.01);
    }

    #[test]
    fn test_fuzz_is_inside() {
        use cgmath::vec3;
        use rand::prelude::*;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(2);
        for _j in 0..100 {
            let l = rng.gen::<f32>();
            let w = rng.gen::<f32>();
            let h = rng.gen::<f32>();
            let b = AxisAlignedBox::new(l, w, h);
            for _i in 0..1000 {
                let x = rng.gen::<f32>() * 5.0;
                let y = rng.gen::<f32>() * 5.0;
                let z = rng.gen::<f32>() * 5.0;
                let inside = (x.abs() < l / 2.0) && (y.abs() < w / 2.0) && (z.abs() < h / 2.0);
                assert_eq!(inside, b.is_inside(vec3(x, y, z)));
            }
        }
    }

    #[test]
    fn test_fuzz_is_intersecting() {
        use cgmath::vec3;
        use rand::prelude::*;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(2);
        for _j in 0..100 {
            let l = rng.gen::<f32>();
            let w = rng.gen::<f32>();
            let h = rng.gen::<f32>();
            let b = AxisAlignedBox::new(l, w, h);
            for _i in 0..1000 {
                let x = rng.gen::<f32>() * 5.0;
                let y = rng.gen::<f32>() * 5.0;
                let z = rng.gen::<f32>() * 5.0;
                assert!(b.is_intersecting(vec3(x, y, z), vec3(0.0, 0.0, 0.0)));
            }
        }
    }
}
