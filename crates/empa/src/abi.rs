use std::mem;

pub use empa_macros::Sized;

pub unsafe trait Sized {
    const LAYOUT: &'static [MemoryUnit];
}

pub unsafe trait Unsized {
    const SIZED_HEAD_LAYOUT: &'static [MemoryUnit];

    const UNSIZED_TAIL_LAYOUT: Option<&'static [MemoryUnit]>;
}

unsafe impl<T> Unsized for T
where
    T: Sized,
{
    const SIZED_HEAD_LAYOUT: &'static [MemoryUnit] = T::LAYOUT;
    const UNSIZED_TAIL_LAYOUT: Option<&'static [MemoryUnit]> = None;
}

unsafe impl<T> Unsized for [T]
where
    T: Sized,
{
    const SIZED_HEAD_LAYOUT: &'static [MemoryUnit] = &[];
    const UNSIZED_TAIL_LAYOUT: Option<&'static [MemoryUnit]> = Some(T::LAYOUT);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MemoryUnit {
    pub offset: usize,
    pub layout: MemoryUnitLayout,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MemoryUnitLayout {
    Float,
    FloatVector2,
    FloatVector3,
    FloatVector4,
    Integer,
    IntegerVector2,
    IntegerVector3,
    IntegerVector4,
    UnsignedInteger,
    UnsignedIntegerVector2,
    UnsignedIntegerVector3,
    UnsignedIntegerVector4,
    Matrix2x2,
    Matrix2x3,
    Matrix2x4,
    Matrix3x2,
    Matrix3x3,
    Matrix3x4,
    Matrix4x2,
    Matrix4x3,
    Matrix4x4,
    Array {
        units: &'static [MemoryUnit],
        stride: usize,
        len: usize,
    },
}

unsafe impl<T, const N: usize> Sized for [T; N]
where
    T: Sized,
{
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Array {
            units: T::LAYOUT,
            stride: mem::size_of::<T>(),
            len: N,
        },
    }];
}

unsafe impl Sized for f32 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Float,
    }];
}

unsafe impl Sized for i32 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Integer,
    }];
}

unsafe impl Sized for u32 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::UnsignedInteger,
    }];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(C, align(8))]
pub struct Vec2<T>(pub T, pub T);

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Vec2<u32> {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Vec2<i32> {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Vec2<f32> {}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for Vec2<u32> {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for Vec2<i32> {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for Vec2<f32> {}

unsafe impl Sized for Vec2<f32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::FloatVector2,
    }];
}

unsafe impl Sized for Vec2<i32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::IntegerVector2,
    }];
}

unsafe impl Sized for Vec2<u32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::UnsignedIntegerVector2,
    }];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(C, align(16))]
pub struct Vec3<T>(pub T, pub T, pub T);

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Vec3<u32> {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Vec3<i32> {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Vec3<f32> {}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for Vec3<u32> {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for Vec3<i32> {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for Vec3<f32> {}

unsafe impl Sized for Vec3<f32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::FloatVector3,
    }];
}

unsafe impl Sized for Vec3<i32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::IntegerVector3,
    }];
}

unsafe impl Sized for Vec3<u32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::UnsignedIntegerVector3,
    }];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(C, align(16))]
pub struct Vec4<T>(pub T, pub T, pub T, pub T);

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Vec4<u32> {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Vec4<i32> {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Vec4<f32> {}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for Vec4<u32> {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for Vec4<i32> {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Pod for Vec4<f32> {}

unsafe impl Sized for Vec4<f32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::FloatVector4,
    }];
}

unsafe impl Sized for Vec4<i32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::IntegerVector4,
    }];
}

unsafe impl Sized for Vec4<u32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::UnsignedIntegerVector4,
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Mat2x2(pub Vec2<f32>, pub Vec2<f32>);

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Mat2x2 {}

unsafe impl Sized for Mat2x2 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix2x2,
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Mat2x3(pub Vec3<f32>, pub Vec3<f32>);

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Mat2x3 {}

unsafe impl Sized for Mat2x3 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix2x3,
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Mat2x4(pub Vec4<f32>, pub Vec4<f32>);

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Mat2x4 {}

unsafe impl Sized for Mat2x4 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix2x4,
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Mat3x2(pub Vec2<f32>, pub Vec2<f32>, pub Vec2<f32>);

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Mat3x2 {}

unsafe impl Sized for Mat3x2 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix3x2,
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Mat3x3(pub Vec3<f32>, pub Vec3<f32>, pub Vec3<f32>);

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Mat3x3 {}

unsafe impl Sized for Mat3x3 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix3x3,
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Mat3x4(pub Vec4<f32>, pub Vec4<f32>, pub Vec4<f32>);

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Mat3x4 {}

unsafe impl Sized for Mat3x4 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix3x4,
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Mat4x2(pub Vec2<f32>, pub Vec2<f32>, pub Vec2<f32>, pub Vec2<f32>);

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Mat4x2 {}

unsafe impl Sized for Mat4x2 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix4x2,
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Mat4x3(pub Vec3<f32>, pub Vec3<f32>, pub Vec3<f32>, pub Vec3<f32>);

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Mat4x3 {}

unsafe impl Sized for Mat4x3 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix4x3,
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Mat4x4(pub Vec4<f32>, pub Vec4<f32>, pub Vec4<f32>, pub Vec4<f32>);

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for Mat4x4 {}

unsafe impl Sized for Mat4x4 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix4x4,
    }];
}
