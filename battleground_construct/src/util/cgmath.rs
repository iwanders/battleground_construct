use cgmath::{BaseFloat, Matrix3, Matrix4};

pub mod prelude {
    pub use super::Adjoint;
    pub use super::EuclideanNorm;
    pub use super::HomogenousTruncate;
    pub use super::InvertHomogeneous;
    pub use super::ToAdjoint;
    pub use super::ToCross; // ToSkew?
    pub use super::ToHomogenous;
    pub use super::ToQuaternion;
    pub use super::ToRollPitchYaw;
    pub use super::ToRotation;
    pub use super::ToRotationH;
    pub use super::ToTranslation;
    pub use super::Twist;
    pub use cgmath::MetricSpace;
}

// https://github.com/rustgd/cgmath/issues/461
pub trait ToQuaternion<S: BaseFloat> {
    fn to_quaternion(&self) -> cgmath::Quaternion<S>;
}

impl<S: BaseFloat> ToQuaternion<S> for Matrix4<S> {
    fn to_quaternion(&self) -> cgmath::Quaternion<S> {
        // Build a 3x3 matrix.
        let m = cgmath::Matrix3::<S>::from_cols(
            self.x.truncate(),
            self.y.truncate(),
            self.z.truncate(),
        );
        m.into()
    }
}

pub trait HomogenousTruncate<S: BaseFloat> {
    fn to_truncate_h(&self) -> Matrix3<S>;
}

impl<S: BaseFloat> HomogenousTruncate<S> for Matrix4<S> {
    fn to_truncate_h(&self) -> cgmath::Matrix3<S> {
        cgmath::Matrix3::<S>::from_cols(self.x.truncate(), self.y.truncate(), self.z.truncate())
    }
}

pub trait ToRotationH<S: BaseFloat> {
    fn to_rotation_h(&self) -> Matrix4<S>;
}

impl<S: BaseFloat> ToRotationH<S> for Matrix4<S> {
    fn to_rotation_h(&self) -> cgmath::Matrix4<S> {
        cgmath::Matrix4::<S>::from_cols(
            self.x,
            self.y,
            self.z,
            cgmath::Vector4::<S>::new(S::zero(), S::zero(), S::zero(), S::one()),
        )
    }
}
pub trait ToRotation<S: BaseFloat> {
    fn to_rotation(&self) -> Matrix3<S>;
}

impl<S: BaseFloat> ToRotation<S> for Matrix4<S> {
    fn to_rotation(&self) -> cgmath::Matrix3<S> {
        cgmath::Matrix3::<S>::from_cols(self.x.truncate(), self.y.truncate(), self.z.truncate())
    }
}

pub trait ToHomogenous<S: BaseFloat> {
    fn to_h(&self) -> Matrix4<S>;
}

impl<S: BaseFloat> ToHomogenous<S> for cgmath::Vector3<S> {
    fn to_h(&self) -> cgmath::Matrix4<S> {
        cgmath::Matrix4::<S>::from_cols(
            cgmath::Vector4::<S>::new(S::one(), S::zero(), S::zero(), S::zero()),
            cgmath::Vector4::<S>::new(S::zero(), S::one(), S::zero(), S::zero()),
            cgmath::Vector4::<S>::new(S::zero(), S::zero(), S::one(), S::zero()),
            cgmath::Vector4::<S>::new(self.x, self.y, self.z, S::one()),
        )
    }
}

pub trait InvertHomogeneous<S: BaseFloat> {
    fn to_inv_h(&self) -> Matrix4<S>;
}

impl<S: BaseFloat> InvertHomogeneous<S> for Matrix4<S> {
    fn to_inv_h(&self) -> cgmath::Matrix4<S> {
        use cgmath::Matrix;
        let inv_r = cgmath::Matrix3::<S>::from_cols(
            self.x.truncate(),
            self.y.truncate(),
            self.z.truncate(),
        )
        .transpose();
        let v_inv = -inv_r * self.w.truncate();
        cgmath::Matrix4::<S>::from_cols(
            inv_r.x.extend(S::zero()),
            inv_r.y.extend(S::zero()),
            inv_r.z.extend(S::zero()),
            v_inv.extend(S::one()),
        )
    }
}

pub trait ToTranslation<S: BaseFloat> {
    fn to_translation(&self) -> cgmath::Vector3<S>;
}

impl<S: BaseFloat> ToTranslation<S> for Matrix4<S> {
    fn to_translation(&self) -> cgmath::Vector3<S> {
        self.w.truncate()
    }
}

pub trait EuclideanNorm<S: BaseFloat> {
    fn euclid_norm(&self) -> S;
}

impl<S: BaseFloat> EuclideanNorm<S> for cgmath::Vector3<S> {
    fn euclid_norm(&self) -> S {
        use cgmath::MetricSpace;
        self.distance2(cgmath::Vector3::<S>::new(S::zero(), S::zero(), S::zero()))
            .sqrt()
    }
}

pub trait ToCross<S: BaseFloat> {
    fn to_cross(&self) -> cgmath::Matrix3<S>;
}

impl<S: BaseFloat> ToCross<S> for cgmath::Vector3<S> {
    fn to_cross(&self) -> cgmath::Matrix3<S> {
        cgmath::Matrix3::<S>::from_cols(
            cgmath::Vector3::<S>::new(S::zero(), self.z, -self.y),
            cgmath::Vector3::<S>::new(-self.z, S::zero(), self.x),
            cgmath::Vector3::<S>::new(self.y, -self.x, S::zero()),
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Twist<S: BaseFloat> {
    pub v: cgmath::Vector3<S>,
    pub w: cgmath::Vector3<S>,
}

impl<S: BaseFloat> Twist<S> {
    pub fn new(v: cgmath::Vector3<S>, w: cgmath::Vector3<S>) -> Self {
        Self { v, w }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Adjoint<S: BaseFloat> {
    pub r: cgmath::Matrix3<S>,
    pub p_r: cgmath::Matrix3<S>,
}

impl<S: BaseFloat> std::ops::Mul<Twist<S>> for Adjoint<S> {
    type Output = Twist<S>;

    fn mul(self, other: Twist<S>) -> Self::Output {
        Twist {
            w: self.r * other.w,
            v: self.p_r * other.w + self.r * other.v,
        }
    }
}
impl<S: BaseFloat> std::ops::Add<Twist<S>> for Twist<S> {
    type Output = Twist<S>;

    fn add(self, other: Twist<S>) -> Self::Output {
        Twist {
            w: self.w + other.w,
            v: self.v + other.v,
        }
    }
}

pub trait ToAdjoint<S: BaseFloat> {
    fn to_adjoint(&self) -> Adjoint<S>;
}

impl<S: BaseFloat> ToAdjoint<S> for cgmath::Matrix4<S> {
    fn to_adjoint(&self) -> Adjoint<S> {
        Adjoint {
            r: self.to_rotation(),
            p_r: self.w.truncate().to_cross() * self.to_rotation(),
        }
    }
}

pub trait ToRollPitchYaw<S: BaseFloat> {
    fn to_rpy(&self) -> cgmath::Vector3<S>;
}

impl<S: BaseFloat> ToRollPitchYaw<S> for cgmath::Matrix4<S> {
    fn to_rpy(&self) -> cgmath::Vector3<S> {
        self.to_rotation().to_rpy()
    }
}
impl<S: BaseFloat> ToRollPitchYaw<S> for cgmath::Matrix3<S> {
    fn to_rpy(&self) -> cgmath::Vector3<S> {
        let roll = self.y.z.atan2(self.z.z);
        let pitch = (-self.x.z).atan2((self.y.z.powi(2) + self.z.z.powi(2)).sqrt());
        let yaw = self.x.y.atan2(self.x.x);
        cgmath::Vector3::<S>::new(roll, pitch, yaw)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cgmath::vec3;
    #[test]
    fn test_rpy() {
        use ToRollPitchYaw;

        // Test only single component
        let transform = Matrix4::from_angle_z(cgmath::Rad(0.3));
        let rpy = transform.to_rpy();
        assert_eq!(rpy, vec3(0.0, 0.0, 0.3));

        let transform = Matrix4::from_angle_x(cgmath::Rad(0.3));
        let rpy = transform.to_rpy();
        assert_eq!(rpy, vec3(0.3, 0.0, 0.0));

        let transform = Matrix4::from_angle_y(cgmath::Rad(0.3));
        let rpy = transform.to_rpy();
        assert_eq!(rpy, vec3(0.0, 0.3, 0.0));

        // https://math.stackexchange.com/a/3242503
        // Make rpy rotation matrix with Rz * Ry * Rx

        let transform =
            Matrix4::from_angle_z(cgmath::Rad(0.3)) * Matrix4::from_angle_y(cgmath::Rad(0.3));
        let rpy = transform.to_rpy();
        assert_eq!(rpy, vec3(0.0, 0.3, 0.3));

        let transform =
            Matrix4::from_angle_z(cgmath::Rad(0.3)) * Matrix4::from_angle_x(cgmath::Rad(0.3));
        let rpy = transform.to_rpy();
        assert_eq!(rpy, vec3(0.3, 0.0, 0.3));
    }
}
