use zeroable::Zeroable;

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

// This doesn't currently work, but I'm hopeful it will one day:
// https://github.com/rust-lang/rust/issues/76560
//
// unsafe impl<T, const N: usize> FixedSized for [T; N] where T: FixedSized {
//     default const MEMORY_UNITS: &'static [MemoryUnit] = &{
//         const LEN: usize = T::MEMORY_UNITS.len() * N;
//
//         // Initialize with temp values
//         let mut memory_units = [MemoryUnit {
//             offset: 0,
//             layout: MemoryUnitLayout::Float
//         }; LEN];
//
//         let stride = mem::size_of::<T>();
//         let mut index = 0;
//
//         for i in 0..N {
//             for mut unit in T::MEMORY_UNITS.iter().copied() {
//                 unit.offset += i * stride;
//
//                 memory_units[index] = unit;
//
//                 index += 1;
//             }
//         }
//
//         memory_units
//     };
// }
//
// This concept for representing arrays of complex types is taken from how OpenGL represents such
// type in its shader program reflection API. The downside is that if a user decides to declare a
// long fixed size array of complex types, this could result in a significant size increase in the
// compiled binary (though I think that for arrays of any significant size, a runtime-sized array is
// probably the more appropriate tool). The upside is that checking if two memory layouts match
// becomes very straightforward (simple pair-wise equality of the two memory unit lists).

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MemoryUnit {
    pub offset: usize,
    pub layout: MemoryUnitLayout,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MemoryUnitLayout {
    Float,
    FloatArray(usize),
    FloatVector2,
    FloatVector2Array(usize),
    FloatVector3,
    FloatVector3Array(usize),
    FloatVector4,
    FloatVector4Array(usize),
    Integer,
    IntegerArray(usize),
    IntegerVector2,
    IntegerVector2Array(usize),
    IntegerVector3,
    IntegerVector3Array(usize),
    IntegerVector4,
    IntegerVector4Array(usize),
    UnsignedInteger,
    UnsignedIntegerArray(usize),
    UnsignedIntegerVector2,
    UnsignedIntegerVector2Array(usize),
    UnsignedIntegerVector3,
    UnsignedIntegerVector3Array(usize),
    UnsignedIntegerVector4,
    UnsignedIntegerVector4Array(usize),
    Matrix2x2,
    Matrix2x2Array(usize),
    Matrix2x3,
    Matrix2x3Array(usize),
    Matrix2x4,
    Matrix2x4Array(usize),
    Matrix3x2,
    Matrix3x2Array(usize),
    Matrix3x3,
    Matrix3x3Array(usize),
    Matrix3x4,
    Matrix3x4Array(usize),
    Matrix4x2,
    Matrix4x2Array(usize),
    Matrix4x3,
    Matrix4x3Array(usize),
    Matrix4x4,
    Matrix4x4Array(usize),
}

unsafe impl Sized for f32 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Float,
    }];
}

unsafe impl<const N: usize> Sized for [f32; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::FloatArray(N),
    }];
}

unsafe impl Sized for i32 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Integer,
    }];
}

unsafe impl<const N: usize> Sized for [i32; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::IntegerArray(N),
    }];
}

unsafe impl Sized for u32 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::UnsignedInteger,
    }];
}

unsafe impl<const N: usize> Sized for [u32; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::UnsignedIntegerArray(N),
    }];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Zeroable)]
#[repr(C, align(8))]
pub struct Vec2<T>(pub T, pub T);

unsafe impl Sized for Vec2<f32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::FloatVector2,
    }];
}

unsafe impl<const N: usize> Sized for [Vec2<f32>; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::FloatVector2Array(N),
    }];
}

unsafe impl Sized for Vec2<i32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::IntegerVector2,
    }];
}

unsafe impl<const N: usize> Sized for [Vec2<i32>; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::IntegerVector2Array(N),
    }];
}

unsafe impl Sized for Vec2<u32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::UnsignedIntegerVector2,
    }];
}

unsafe impl<const N: usize> Sized for [Vec2<u32>; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::UnsignedIntegerVector2Array(N),
    }];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Zeroable)]
#[repr(C, align(16))]
pub struct Vec3<T>(pub T, pub T, pub T);

unsafe impl Sized for Vec3<f32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::FloatVector3,
    }];
}

unsafe impl<const N: usize> Sized for [Vec3<f32>; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::FloatVector3Array(N),
    }];
}

unsafe impl Sized for Vec3<i32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::IntegerVector3,
    }];
}

unsafe impl<const N: usize> Sized for [Vec3<i32>; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::IntegerVector3Array(N),
    }];
}

unsafe impl Sized for Vec3<u32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::UnsignedIntegerVector3,
    }];
}

unsafe impl<const N: usize> Sized for [Vec3<u32>; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::UnsignedIntegerVector3Array(N),
    }];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Zeroable)]
#[repr(C, align(16))]
pub struct Vec4<T>(pub T, pub T, pub T, pub T);

unsafe impl Sized for Vec4<f32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::FloatVector4,
    }];
}

unsafe impl<const N: usize> Sized for [Vec4<f32>; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::FloatVector4Array(N),
    }];
}

unsafe impl Sized for Vec4<i32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::IntegerVector4,
    }];
}

unsafe impl<const N: usize> Sized for [Vec4<i32>; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::IntegerVector4Array(N),
    }];
}

unsafe impl Sized for Vec4<u32> {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::UnsignedIntegerVector4,
    }];
}

unsafe impl<const N: usize> Sized for [Vec4<u32>; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::UnsignedIntegerVector4Array(N),
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Zeroable)]
#[repr(C)]
pub struct Mat2x2(pub Vec2<f32>, pub Vec2<f32>);

unsafe impl Sized for Mat2x2 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix2x2,
    }];
}

unsafe impl<const N: usize> Sized for [Mat2x2; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix2x2Array(N),
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Zeroable)]
#[repr(C)]
pub struct Mat2x3(pub Vec3<f32>, pub Vec3<f32>);

unsafe impl Sized for Mat2x3 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix2x3,
    }];
}

unsafe impl<const N: usize> Sized for [Mat2x3; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix2x3Array(N),
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Zeroable)]
#[repr(C)]
pub struct Mat2x4(pub Vec4<f32>, pub Vec4<f32>);

unsafe impl Sized for Mat2x4 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix2x4,
    }];
}

unsafe impl<const N: usize> Sized for [Mat2x4; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix2x4Array(N),
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Zeroable)]
#[repr(C)]
pub struct Mat3x2(pub Vec2<f32>, pub Vec2<f32>, pub Vec2<f32>);

unsafe impl Sized for Mat3x2 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix3x2,
    }];
}

unsafe impl<const N: usize> Sized for [Mat3x2; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix3x2Array(N),
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Zeroable)]
#[repr(C)]
pub struct Mat3x3(pub Vec3<f32>, pub Vec3<f32>, pub Vec3<f32>);

unsafe impl Sized for Mat3x3 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix3x3,
    }];
}

unsafe impl<const N: usize> Sized for [Mat3x3; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix3x3Array(N),
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Zeroable)]
#[repr(C)]
pub struct Mat3x4(pub Vec4<f32>, pub Vec4<f32>, pub Vec4<f32>);

unsafe impl Sized for Mat3x4 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix3x4,
    }];
}

unsafe impl<const N: usize> Sized for [Mat3x4; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix3x4Array(N),
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Zeroable)]
#[repr(C)]
pub struct Mat4x2(pub Vec2<f32>, pub Vec2<f32>, pub Vec2<f32>, pub Vec2<f32>);

unsafe impl Sized for Mat4x2 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix4x2,
    }];
}

unsafe impl<const N: usize> Sized for [Mat4x2; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix4x2Array(N),
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Zeroable)]
#[repr(C)]
pub struct Mat4x3(pub Vec3<f32>, pub Vec3<f32>, pub Vec3<f32>, pub Vec3<f32>);

unsafe impl Sized for Mat4x3 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix4x3,
    }];
}

unsafe impl<const N: usize> Sized for [Mat4x3; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix4x3Array(N),
    }];
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Zeroable)]
#[repr(C)]
pub struct Mat4x4(pub Vec4<f32>, pub Vec4<f32>, pub Vec4<f32>, pub Vec4<f32>);

unsafe impl Sized for Mat4x4 {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix4x4,
    }];
}

unsafe impl<const N: usize> Sized for [Mat4x4; N] {
    const LAYOUT: &'static [MemoryUnit] = &[MemoryUnit {
        offset: 0,
        layout: MemoryUnitLayout::Matrix4x4Array(N),
    }];
}
