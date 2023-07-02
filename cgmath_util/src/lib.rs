use cgmath::{BaseFloat, Matrix3, Matrix4};
pub type Mat4 = cgmath::Matrix4<f32>;
pub type Vec3 = cgmath::Vector3<f32>;
pub use cgmath::vec3;
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use super::Adjoint;
    pub use super::EuclideanNorm;
    pub use super::HomogenousTruncate;
    pub use super::InvertHomogeneous;
    pub use super::RollPitchYawToHomogenous;
    pub use super::RotationFrom;
    pub use super::ToAdjoint;
    pub use super::ToCross; // ToSkew?
    pub use super::ToHomogenous;
    pub use super::ToQuaternion;
    pub use super::ToRollPitchYaw;
    pub use super::ToRotation;
    pub use super::ToRotationH;
    pub use super::ToTranslation;
    pub use super::Twist;
    pub use cgmath::Matrix;
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

impl<S: BaseFloat> ToQuaternion<S> for Matrix3<S> {
    fn to_quaternion(&self) -> cgmath::Quaternion<S> {
        let m = *self;
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

impl<S: BaseFloat> ToRotation<S> for cgmath::Quaternion<S> {
    fn to_rotation(&self) -> cgmath::Matrix3<S> {
        cgmath::Matrix3::<S>::from(*self)
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
impl<S: BaseFloat> ToHomogenous<S> for cgmath::Matrix3<S> {
    fn to_h(&self) -> cgmath::Matrix4<S> {
        cgmath::Matrix4::<S>::from_cols(
            self.x.extend(S::zero()),
            self.y.extend(S::zero()),
            self.z.extend(S::zero()),
            cgmath::Vector4::<S>::new(S::zero(), S::zero(), S::zero(), S::one()),
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

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub struct Twist<S: BaseFloat> {
    pub v: cgmath::Vector3<S>,
    pub w: cgmath::Vector3<S>,
}

impl<S: BaseFloat> Twist<S> {
    pub fn new(v: cgmath::Vector3<S>, w: cgmath::Vector3<S>) -> Self {
        Self { v, w }
    }

    pub fn to_cross(&self) -> Matrix4<S> {
        cgmath::Matrix4::<S>::from_cols(
            cgmath::Vector4::<S>::new(S::zero(), self.w.z, -self.w.y, S::zero()),
            cgmath::Vector4::<S>::new(-self.w.z, S::zero(), self.w.x, S::zero()),
            cgmath::Vector4::<S>::new(self.w.y, -self.w.x, S::zero(), S::zero()),
            self.v.extend(S::zero()),
        )
    }

    pub fn exp(&self) -> Matrix4<S> {
        use cgmath::SquareMatrix;
        let wnorm = self.w.euclid_norm();
        let r = (self.w / wnorm).rotation_about(cgmath::Rad::<S>(self.w.euclid_norm()));
        let s = S::one() / (wnorm * wnorm);
        let wv = self.w.to_cross() * self.v;
        let wvw = self.w * self.w.transposed_mul(self.v);
        let v = ((cgmath::Matrix3::<S>::identity() - r) * wv + wvw) * s;
        let mut r = r.to_h();
        r.w = v.extend(S::one());
        r
    }
}

// Scaling of a twist, expected it was a unit twist, but not validated.
impl<S: BaseFloat> std::ops::Mul<S> for Twist<S> {
    type Output = Twist<S>;

    fn mul(self, other: S) -> Self::Output {
        Twist {
            w: self.w * other,
            v: self.v * other,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq)]
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

pub trait RollPitchYawToHomogenous<S: BaseFloat> {
    fn rpy_to_h(&self) -> cgmath::Matrix4<S>;
}
impl<S: BaseFloat> RollPitchYawToHomogenous<S> for cgmath::Vector3<S> {
    fn rpy_to_h(&self) -> cgmath::Matrix4<S> {
        Matrix4::from_angle_z(cgmath::Rad(self.z))
            * Matrix4::from_angle_y(cgmath::Rad(self.y))
            * Matrix4::from_angle_x(cgmath::Rad(self.x))
    }
}

pub trait RotationFrom<S: BaseFloat> {
    fn rotation_from(&self, v: cgmath::Vector3<S>) -> cgmath::Matrix3<S>;

    fn rotation_about<A: Into<cgmath::Rad<S>>>(&self, angle: A) -> cgmath::Matrix3<S>
    where
        Self: ToCross<S> + Sized,
    {
        use cgmath::Angle;
        use cgmath::SquareMatrix;
        let k = self.to_cross();
        let a: cgmath::Rad<S> = angle.into();
        cgmath::Matrix3::<S>::identity() + (k * a.sin()) + k * k * (S::one() - a.cos())
    }
}

// https://math.stackexchange.com/a/3262315
impl<S: BaseFloat> RotationFrom<S> for cgmath::Vector3<S> {
    fn rotation_from(&self, v: cgmath::Vector3<S>) -> cgmath::Matrix3<S> {
        use cgmath::SquareMatrix;
        let u = *self;
        let w = u.cross(v);
        let k = w.to_cross();
        let d = w.euclid_norm();
        let vu = v.euclid_norm() * u.euclid_norm();
        let a = (d / vu).asin();
        cgmath::Matrix3::<S>::identity() + (k * a.sin()) + k * k * (S::one() - a.cos())
    }
}

pub trait MulTranspose<S: BaseFloat> {
    fn transposed_mul(&self, other: cgmath::Vector3<S>) -> S;
}

impl<S: BaseFloat> MulTranspose<S> for cgmath::Vector3<S> {
    fn transposed_mul(&self, other: cgmath::Vector3<S>) -> S {
        self[0] * other[0] + self[1] * other[1] + self[2] * other[2]
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

    #[test]
    fn test_rotation_about() {
        use cgmath::Rad;
        let v = vec3(1.0f32, 0.0, 0.0);
        let r = v.rotation_about(Rad(0.5));
        assert_eq!(r.x.x, 1.0);
        assert!((r.y.y - 0.87758).abs() < 0.001);
        assert!((r.y.z - 0.47943).abs() < 0.001);
        assert!((r.z.y - -0.47943).abs() < 0.001);
        assert!((r.z.z - 0.87758).abs() < 0.001);
    }

    #[test]
    fn test_exponential_twist() {
        let t = Twist::new(vec3(0.0f32, 0.0, 0.0), vec3(0.0, 0.0, 1.0));
        let exp_t = t.exp();
        assert!((exp_t.x.x - 0.54030).abs() < 0.001);
        assert!((exp_t.x.y - 0.84147).abs() < 0.001);
        assert!((exp_t.y.x - -0.84147).abs() < 0.001);
        assert!((exp_t.y.y - 0.54030).abs() < 0.001);
        assert!((exp_t.z.z - 1.0).abs() < 0.001);
        assert!((exp_t.w.w - 1.0).abs() < 0.001);

        let t = Twist::new(vec3(4.0f32, 5.0, 6.0), vec3(1.0, 2.0, 3.0));
        let exp_t = t.exp();
        println!("{exp_t:?}");
        /*
        The answer here is:
        -0.69492   0.71352   0.08929   1.63586
        -0.19201  -0.30379   0.93319   5.28902
        0.69298   0.63135   0.34811   6.59537
        0.00000   0.00000   0.00000   1.00000
        */
        assert!((exp_t.w.x - 1.63586).abs() < 0.001);
        assert!((exp_t.w.y - 5.28902).abs() < 0.001);
        assert!((exp_t.w.z - 6.59537).abs() < 0.001);
        assert!((exp_t.w.w - 1.0).abs() < 0.001);
    }
}
