use cgmath::{BaseNum, Vector3};

/// Generic cuboid of given dimensions. Cuboid is centered around the origin.
/// Technicallly a RectangularCuboid.
#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Cuboid<S: BaseNum> {
    pub x: S, // x
    pub y: S, // y
    pub z: S, // z
}

impl<S: BaseNum + std::fmt::Display> Cuboid<S> {
    #[inline]
    pub const fn new(width: S, height: S, depth: S) -> Cuboid<S> {
        Cuboid {
            x: width,
            y: height,
            z: depth,
        }
    }

    #[inline]
    pub fn min(&self) -> Vector3<S> {
        let zero = S::zero();
        self.parametrized_point(zero, zero, zero)
    }

    #[inline]
    pub fn max(&self) -> Vector3<S> {
        let one = S::one();
        self.parametrized_point(one, one, one)
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

    /*
    /// Check if the line segment intersects the box
    pub fn is_intersecting(&self, p0: Vector3<S>, p1: Vector3<S>) -> bool {
        // https://math.stackexchange.com/a/1164672
        // but that has singularities.
        /*
         *      Amy Williams, Steve Barrus, R. Keith Morley, and Peter Shirley
         *      "An Efficient and Robust Ray-Box Intersection Algorithm"
         *      Journal of graphics tools, 10(1):49-54, 2005
         */
        // Seems that's pretty much the solution everyone is using.

        let one = S::one();
        let zero = S::zero();

        // Corners of the box
        let bounds = [self.min(), self.max()];

        // Direction of the ray
        let ray_dir = p1 - p0;
        // Normalize that
        let ray_dir_inv = Vector3::<S>::new(one / ray_dir.x, one / ray_dir.y, one / ray_dir.z);

        // To avoid branching, make this index lookup.
        let sign = [(ray_dir_inv.x  < zero) as usize, (ray_dir_inv.y  < zero) as usize, (ray_dir_inv.z  <zero) as usize];

        let mut tmin = (bounds[sign[0]].x - p0.x) * ray_dir_inv.x;
        let mut tmax = (bounds[1 - sign[0]].x - p0.x) * ray_dir_inv.x;

        let tymin = (bounds[sign[0]].y - p0.y) * ray_dir_inv.y;
        let tymax = (bounds[1 - sign[0]].y - p0.y) * ray_dir_inv.y;

        if (tmin > tymax) || (tymin > tmax)
        {
            return false;
        }
        if tymin > tmin
        {
            tmin = tymin;
        }
        if tymax < tmax
        {
            tmax = tymax;
        }

        let tzmin = (bounds[sign[2]].z - p0.z) * ray_dir_inv.z;
        let tzmax = (bounds[1 - sign[2]].z - p0.z) * ray_dir_inv.z;

        if (tmin > tzmax) || (tzmin > tmax)
        {
            return false;
        }

        println!("\nclassic");
        println!("tmax: {tmax}");
        println!("tmin: {tmin}");
        println!("p0: {p0:?}");
        println!("p1: {p1:?}");


        // Add condition that check if the line is in the interval.
        // if (tmin < zero || tmin > one) && (tmax < zero || tmax > one) {
            // return false;
        // }

        fn max<S: std::cmp::PartialOrd>(a: S, b: S) -> S {
            if a > b { a } else { b }
        }
        let inside = tmax > max(tmin, zero) && max(tmin, zero) < one;
        return inside;
    }
    */


    // https://github.com/tavianator/tavianator.com/blob/main/src/2015/ray_box_nan.md
    // has a nice implementation that's branchless.
    pub fn is_intersecting_branchless(&self, p0: Vector3<S>, p1: Vector3<S>) -> bool{
        let bmin = self.min();
        let bmax = self.max();
        let rorigin = p0;
        // Direction of the ray
        let ray_dir = p1 - p0;
        // Normalize that
        let one = S::one();
        let zero = S::zero();
        let rdir_inv = Vector3::<S>::new(one / ray_dir.x, one / ray_dir.y, one / ray_dir.z);

        // Use a < b, NOT a <= b, the latter doesn't optimise to a single instruction.
        fn min<S: std::cmp::PartialOrd>(a: S, b: S) -> S {
            if a < b { a } else { b }
        }
        fn max<S: std::cmp::PartialOrd>(a: S, b: S) -> S {
            if a > b { a } else { b }
        }

        // Unroll manually such that we can avoid the infty.
        let i = 0;
        let t1 = (bmin[i] - rorigin[i])*rdir_inv[i];
        let t2 = (bmax[i] - rorigin[i])*rdir_inv[i];

        let mut tmin = min(t1, t2);
        let mut tmax = max(t1, t2);

        let i = 1;
        let t1 = (bmin[i] - rorigin[i])*rdir_inv[i];
        let t2 = (bmax[i] - rorigin[i])*rdir_inv[i];

        let i = 2;
        tmin = max(tmin, min(t1, t2));
        tmax = min(tmax, max(t1, t2));

        let t1 = (bmin[i] - rorigin[i])*rdir_inv[i];
        let t2 = (bmax[i] - rorigin[i])*rdir_inv[i];

        tmin = max(tmin, min(t1, t2));
        tmax = min(tmax, max(t1, t2));

        // println!();
        // println!("tmax: {tmax}");
        // println!("tmin: {tmin}");
        // println!("p0: {p0:?}");
        // println!("p1: {p1:?}");


        // Original:
        // return tmax > max(tmin, zero);

        // Add the condition to check that the line segment (parametrized to [0.0, 1.0]) is met.
        let inside = tmax > max(tmin, zero) && max(tmin, zero) < one;
        // println!("inside: {inside:?}");
        return inside;
    }
}

#[cfg(test)]
mod test {
    fn verify_points<S: BaseNum+ std::fmt::Display>(b: &Cuboid<S>, p0: Vector3<S>, p1: Vector3<S>, delta: S) {
        let rorigin = p0;
        // Direction of the ray
        let ray_dir = p1 - p0;
        // Normalize that
        let zero = S::zero();
        let one = S::one();

        let point_on_line = |p: S| { use cgmath::ElementWise;

            rorigin + ray_dir.mul_element_wise(p)
        };

        let mut t = zero;
        while t < one {
            let small_p0 = point_on_line(t);
            let small_p1 = point_on_line(t + delta);
            // let line_check1 = b.is_intersecting(small_p0, small_p1);
            let line_check2 = b.is_intersecting_branchless(small_p0, small_p1);
            let p_check = b.is_inside(small_p0) || b.is_inside(small_p1);
            // assert_eq!(p_check, line_check1, "left: {p_check}, right: {line_check1}, t: {t}");
            assert_eq!(p_check, line_check2);
            t += delta;
        }
    }

    use super::*;
    #[test]
    fn test_is_inside() {
        let b = Cuboid::new(1.0f32, 1.0, 1.0);

        // x direction.
        assert_eq!(b.is_inside(Vector3::new(-1.0, 0.0, 0.0)), false);
        assert_eq!(b.is_inside(Vector3::new(-0.4, 0.0, 0.0)), true);
        assert_eq!(b.is_inside(Vector3::new(0.0, 0.0, 0.0)), true);
        assert_eq!(b.is_inside(Vector3::new(0.4, 0.0, 0.0)), true);
        assert_eq!(b.is_inside(Vector3::new(1.0, 0.0, 0.0)), false);

        // y direction.
        assert_eq!(b.is_inside(Vector3::new(0.0, -1.0, 0.0)), false);
        assert_eq!(b.is_inside(Vector3::new(0.0, -0.4, 0.0)), true);
        assert_eq!(b.is_inside(Vector3::new(0.0, 0.0, 0.0)), true);
        assert_eq!(b.is_inside(Vector3::new(0.0, 0.4, 0.0)), true);
        assert_eq!(b.is_inside(Vector3::new(0.0, 1.0, 0.0)), false);

        // z direction.
        assert_eq!(b.is_inside(Vector3::new(0.0, 0.0, -1.0)), false);
        assert_eq!(b.is_inside(Vector3::new(0.0, 0.0, -0.4)), true);
        assert_eq!(b.is_inside(Vector3::new(0.0, 0.0, 0.0)), true);
        assert_eq!(b.is_inside(Vector3::new(0.0, 0.0, 0.4)), true);
        assert_eq!(b.is_inside(Vector3::new(0.0, 0.0, 1.0)), false);

        // diagonal
        assert_eq!(b.is_inside(Vector3::new(-1.0, -1.0, -1.0)), false);
        assert_eq!(b.is_inside(Vector3::new(-0.4, -0.4, -0.4)), true);
        assert_eq!(b.is_inside(Vector3::new(0.0, 0.0, 0.0)), true);
        assert_eq!(b.is_inside(Vector3::new(0.4, 0.4, 0.4)), true);
        assert_eq!(b.is_inside(Vector3::new(1.0, 1.0, 1.0)), false);
    }

    #[test]
    fn test_is_intersecting() {
        use cgmath::vec3;
        let b = Cuboid::new(1.0f32, 1.0, 1.0);
        // assert_eq!(b.is_intersecting(vec3(1.0, 1.0, 1.0), vec3(0.0, 0.0, 0.0), ), true);
        assert_eq!(b.is_intersecting_branchless(vec3(1.0, 1.0, 1.0), vec3(0.0, 0.0, 0.0), ), true);
        // assert_eq!(b.is_intersecting(vec3(1.0, 1.5, 1.0), vec3(2.0, 2.0, 2.0), ), false);
        assert_eq!(b.is_intersecting_branchless(vec3(1.0, 1.5, 1.0), vec3(2.0, 2.0, 2.0), ), false);

        // pointing at box, but not into it.
        // assert_eq!(b.is_intersecting(vec3(3.0, 0.0, 0.0), vec3(5.0, 0.0, 0.0)), false);
        assert_eq!(b.is_intersecting_branchless(vec3(3.0, 0.0, 0.0), vec3(5.0, 0.0, 0.0)), false);

        verify_points(&b, vec3(1.0, 1.0, 1.0), vec3(0.0, 0.0, 0.0), 0.001);
        verify_points(&b, vec3(2.0, 0.0, 0.0), vec3(3.0, 0.0, 0.0), 0.001);
        verify_points(&b, vec3(3.0, 0.0, 0.0), vec3(2.0, 0.0, 0.0), 0.001);
        verify_points(&b, vec3(2.25, 0.0, 0.0), vec3(0.25, 0.0, 0.0), 0.0001);
        verify_points(&b, vec3(0.25, 0.0, 0.0), vec3(2.25, 0.0, 0.0), 0.01);
        verify_points(&b, vec3(-2.25, 0.0, 0.0), vec3(-0.25, 0.0, 0.0), 0.0001);
        verify_points(&b, vec3(-0.25, 0.0, 0.0), vec3(-2.25, 0.0, 0.0), 0.01);
    }
}
