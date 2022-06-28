use cgmath::{Matrix2, Matrix3, Matrix4, Vector1, Vector2, Vector3, Vector4};
use empa::abi;

pub trait ToAbi {
    type Abi: abi::Sized;

    fn to_abi(&self) -> Self::Abi;
}

impl ToAbi for Vector1<f32> {
    type Abi = f32;

    fn to_abi(&self) -> Self::Abi {
        self.x
    }
}

impl ToAbi for Vector2<f32> {
    type Abi = abi::Vec2<f32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec2(self.x, self.y)
    }
}

impl ToAbi for Vector3<f32> {
    type Abi = abi::Vec3<f32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec3(self.x, self.y, self.z)
    }
}

impl ToAbi for Vector4<f32> {
    type Abi = abi::Vec4<f32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec4(self.x, self.y, self.z, self.w)
    }
}

impl ToAbi for Vector1<i32> {
    type Abi = i32;

    fn to_abi(&self) -> Self::Abi {
        self.x
    }
}

impl ToAbi for Vector2<i32> {
    type Abi = abi::Vec2<i32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec2(self.x, self.y)
    }
}

impl ToAbi for Vector3<i32> {
    type Abi = abi::Vec3<i32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec3(self.x, self.y, self.z)
    }
}

impl ToAbi for Vector4<i32> {
    type Abi = abi::Vec4<i32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec4(self.x, self.y, self.z, self.w)
    }
}

impl ToAbi for Vector1<u32> {
    type Abi = u32;

    fn to_abi(&self) -> Self::Abi {
        self.x
    }
}

impl ToAbi for Vector2<u32> {
    type Abi = abi::Vec2<u32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec2(self.x, self.y)
    }
}

impl ToAbi for Vector3<u32> {
    type Abi = abi::Vec3<u32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec3(self.x, self.y, self.z)
    }
}

impl ToAbi for Vector4<u32> {
    type Abi = abi::Vec4<u32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec4(self.x, self.y, self.z, self.w)
    }
}

impl ToAbi for Matrix2<f32> {
    type Abi = abi::Mat2x2;

    fn to_abi(&self) -> Self::Abi {
        abi::Mat2x2(self.x.to_abi(), self.y.to_abi())
    }
}

impl ToAbi for Matrix3<f32> {
    type Abi = abi::Mat3x3;

    fn to_abi(&self) -> Self::Abi {
        abi::Mat3x3(self.x.to_abi(), self.y.to_abi(), self.z.to_abi())
    }
}

impl ToAbi for Matrix4<f32> {
    type Abi = abi::Mat4x4;

    fn to_abi(&self) -> Self::Abi {
        abi::Mat4x4(
            self.x.to_abi(),
            self.y.to_abi(),
            self.z.to_abi(),
            self.w.to_abi(),
        )
    }
}
