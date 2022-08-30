use empa::abi;

pub trait ToAbi {
    type Abi: abi::Sized;

    fn to_abi(&self) -> Self::Abi;
}

impl ToAbi for glam::f32::Vec2 {
    type Abi = abi::Vec2<f32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec2(self.x, self.y)
    }
}

impl ToAbi for glam::f32::Vec3 {
    type Abi = abi::Vec3<f32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec3(self.x, self.y, self.z)
    }
}

impl ToAbi for glam::f32::Vec4 {
    type Abi = abi::Vec4<f32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec4(self.x, self.y, self.z, self.w)
    }
}

impl ToAbi for glam::i32::IVec2 {
    type Abi = abi::Vec2<i32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec2(self.x, self.y)
    }
}

impl ToAbi for glam::i32::IVec3 {
    type Abi = abi::Vec3<i32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec3(self.x, self.y, self.z)
    }
}

impl ToAbi for glam::i32::IVec4 {
    type Abi = abi::Vec4<i32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec4(self.x, self.y, self.z, self.w)
    }
}

impl ToAbi for glam::u32::UVec2 {
    type Abi = abi::Vec2<u32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec2(self.x, self.y)
    }
}

impl ToAbi for glam::u32::UVec3 {
    type Abi = abi::Vec3<u32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec3(self.x, self.y, self.z)
    }
}

impl ToAbi for glam::u32::UVec4 {
    type Abi = abi::Vec4<u32>;

    fn to_abi(&self) -> Self::Abi {
        abi::Vec4(self.x, self.y, self.z, self.w)
    }
}

impl ToAbi for glam::f32::Mat2 {
    type Abi = abi::Mat2x2;

    fn to_abi(&self) -> Self::Abi {
        abi::Mat2x2(self.col(0).to_abi(), self.col(1).to_abi())
    }
}

impl ToAbi for glam::f32::Mat3 {
    type Abi = abi::Mat3x3;

    fn to_abi(&self) -> Self::Abi {
        abi::Mat3x3(self.col(0).to_abi(), self.col(1).to_abi(), self.col(2).to_abi())
    }
}

impl ToAbi for glam::f32::Mat4 {
    type Abi = abi::Mat4x4;

    fn to_abi(&self) -> Self::Abi {
        abi::Mat4x4(
            self.col(0).to_abi(),
            self.col(1).to_abi(),
            self.col(2).to_abi(),
            self.col(3).to_abi(),
        )
    }
}
