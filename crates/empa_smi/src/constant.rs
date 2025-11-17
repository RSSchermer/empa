//! A compile-time constant description of a shader module interface.
//!
//! This is primarily meant to be constructed by procedural macros at compile-time.

use crate::{IoBinding, OverridableConstant, ShaderStage, StorageTextureFormat, TexelType};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ArrayLayout {
    pub element_layout: &'static [MemoryUnit],
    pub stride: u64,
    pub len: u64,
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
    Array(ArrayLayout),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MemoryUnit {
    pub offset: u64,
    pub layout: MemoryUnitLayout,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SizedBufferLayout {
    pub memory_units: &'static [MemoryUnit],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UnsizedTailLayout {
    pub offset: u64,
    pub element_layout: &'static [MemoryUnit],
    pub stride: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UnsizedBufferLayout {
    pub sized_head: &'static [MemoryUnit],
    pub unsized_tail: Option<UnsizedTailLayout>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceType {
    Texture1D(TexelType),
    Texture2D(TexelType),
    Texture3D(TexelType),
    Texture2DArray(TexelType),
    TextureCube(TexelType),
    TextureCubeArray(TexelType),
    TextureMultisampled2D(TexelType),
    TextureDepth2D,
    TextureDepth2DArray,
    TextureDepthCube,
    TextureDepthCubeArray,
    TextureDepthMultisampled2D,
    StorageTexture1D(StorageTextureFormat),
    StorageTexture2D(StorageTextureFormat),
    StorageTexture2DArray(StorageTextureFormat),
    StorageTexture3D(StorageTextureFormat),
    FilteringSampler,
    NonFilteringSampler,
    ComparisonSampler,
    Uniform(SizedBufferLayout),
    StorageRead(UnsizedBufferLayout),
    StorageReadWrite(UnsizedBufferLayout),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ResourceBinding {
    pub group: u32,
    pub binding: u32,
    pub resource_type: ResourceType,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EntryPoint {
    pub name: &'static str,
    pub stage: ShaderStage,
    pub input_bindings: &'static [IoBinding],
    pub output_bindings: &'static [IoBinding],
    pub overridable_constants: &'static [usize],
    pub resource_bindings: &'static [usize],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ShaderModuleInterface {
    pub overridable_constants: &'static [OverridableConstant],
    pub resource_bindings: &'static [ResourceBinding],
    pub entry_points: &'static [EntryPoint],
}
