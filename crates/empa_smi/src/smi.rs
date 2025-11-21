use std::borrow::Cow;
use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum OverridableConstantType {
    Float,
    Bool,
    SignedInteger,
    UnsignedInteger,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum TexelType {
    Float,
    UnfilterableFloat,
    Integer,
    UnsignedInteger,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub enum StorageTextureFormat {
    rgba8unorm,
    rgba8snorm,
    rgba8uint,
    rgba8sint,
    rgba16uint,
    rgba16sint,
    rgba16float,
    r32uint,
    r32sint,
    r32float,
    rg32uint,
    rg32sint,
    rg32float,
    rgba32uint,
    rgba32sint,
    rgba32float,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum IoBindingType {
    SignedInteger,
    SignedIntegerVector2,
    SignedIntegerVector3,
    SignedIntegerVector4,
    UnsignedInteger,
    UnsignedIntegerVector2,
    UnsignedIntegerVector3,
    UnsignedIntegerVector4,
    Float,
    FloatVector2,
    FloatVector3,
    FloatVector4,
    HalfFloat,
    HalfFloatVector2,
    HalfFloatVector3,
    HalfFloatVector4,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum InterpolationType {
    Perspective,
    Linear,
    Flat,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum Sampling {
    Center,
    Centroid,
    Sample,
    First,
    Either,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct Interpolate {
    pub interpolation_type: InterpolationType,
    pub sampling: Option<Sampling>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct IoBinding {
    pub location: u32,
    pub binding_type: IoBindingType,
    pub interpolate: Option<Interpolate>,
}

impl PartialOrd for IoBinding {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.location.partial_cmp(&other.location)
    }
}

impl Ord for IoBinding {
    fn cmp(&self, other: &Self) -> Ordering {
        self.location.cmp(&other.location)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct OverridableConstant {
    pub name: Cow<'static, str>,
    pub id: Option<u16>,
    pub constant_type: OverridableConstantType,
    pub required: bool,
}

impl PartialOrd for OverridableConstant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id).and_then(|ord| match ord {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Equal => self.name.partial_cmp(&other.name),
            Ordering::Greater => Some(Ordering::Greater),
        })
    }
}

impl Ord for OverridableConstant {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.id.cmp(&other.id) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.name.cmp(&other.name),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ArrayLayout {
    pub element_layout: Cow<'static, [MemoryUnit]>,
    pub stride: u64,
    pub len: u64,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct MemoryUnit {
    pub offset: u64,
    pub layout: MemoryUnitLayout,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct SizedBufferLayout {
    pub memory_units: Cow<'static, [MemoryUnit]>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct UnsizedTailLayout {
    pub offset: u64,
    pub element_layout: Cow<'static, [MemoryUnit]>,
    pub stride: u64,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct UnsizedBufferLayout {
    pub sized_head: Cow<'static, [MemoryUnit]>,
    pub unsized_tail: Option<UnsizedTailLayout>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ResourceBinding {
    pub group: u32,
    pub binding: u32,
    pub resource_type: ResourceType,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct EntryPoint {
    pub name: Cow<'static, str>,
    pub stage: ShaderStage,
    pub input_bindings: Cow<'static, [IoBinding]>,
    pub output_bindings: Cow<'static, [IoBinding]>,
    pub overridable_constants: Cow<'static, [usize]>,
    pub resource_bindings: Cow<'static, [usize]>,
}

impl PartialOrd for EntryPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for EntryPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ShaderModuleInterface {
    pub overridable_constants: Cow<'static, [OverridableConstant]>,
    pub resource_bindings: Cow<'static, [ResourceBinding]>,
    pub entry_points: Cow<'static, [EntryPoint]>,
}
