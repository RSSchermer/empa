pub mod constant;
pub mod dynamic;

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct OverridableConstant {
    pub id: u32,
    pub constant_type: OverridableConstantType,
    pub required: bool,
}

impl PartialOrd for OverridableConstant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for OverridableConstant {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
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

pub enum ArrayLayout<'a> {
    Dynamic(&'a dynamic::ArrayLayout),
    Constant(&'static constant::ArrayLayout),
}

impl<'a> ArrayLayout<'a> {
    pub fn element_layout(&self) -> MemoryUnits<'a> {
        match self {
            ArrayLayout::Dynamic(layout) => MemoryUnits::Dynamic(&layout.element_layout),
            ArrayLayout::Constant(layout) => MemoryUnits::Constant(layout.element_layout),
        }
    }

    pub fn stride(&self) -> u64 {
        match self {
            ArrayLayout::Dynamic(layout) => layout.stride,
            ArrayLayout::Constant(layout) => layout.stride,
        }
    }

    pub fn len(&self) -> u64 {
        match self {
            ArrayLayout::Dynamic(layout) => layout.len,
            ArrayLayout::Constant(layout) => layout.len,
        }
    }
}

pub enum MemoryUnitLayout<'a> {
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
    Array(ArrayLayout<'a>),
}

impl<'a> From<&'a dynamic::MemoryUnitLayout> for MemoryUnitLayout<'a> {
    fn from(value: &'a dynamic::MemoryUnitLayout) -> Self {
        match value {
            dynamic::MemoryUnitLayout::Float => MemoryUnitLayout::Float,
            dynamic::MemoryUnitLayout::FloatVector2 => MemoryUnitLayout::FloatVector2,
            dynamic::MemoryUnitLayout::FloatVector3 => MemoryUnitLayout::FloatVector3,
            dynamic::MemoryUnitLayout::FloatVector4 => MemoryUnitLayout::FloatVector4,
            dynamic::MemoryUnitLayout::Integer => MemoryUnitLayout::Integer,
            dynamic::MemoryUnitLayout::IntegerVector2 => MemoryUnitLayout::IntegerVector2,
            dynamic::MemoryUnitLayout::IntegerVector3 => MemoryUnitLayout::IntegerVector3,
            dynamic::MemoryUnitLayout::IntegerVector4 => MemoryUnitLayout::IntegerVector4,
            dynamic::MemoryUnitLayout::UnsignedInteger => MemoryUnitLayout::UnsignedInteger,
            dynamic::MemoryUnitLayout::UnsignedIntegerVector2 => {
                MemoryUnitLayout::UnsignedIntegerVector2
            }
            dynamic::MemoryUnitLayout::UnsignedIntegerVector3 => {
                MemoryUnitLayout::UnsignedIntegerVector3
            }
            dynamic::MemoryUnitLayout::UnsignedIntegerVector4 => {
                MemoryUnitLayout::UnsignedIntegerVector4
            }
            dynamic::MemoryUnitLayout::Matrix2x2 => MemoryUnitLayout::Matrix2x2,
            dynamic::MemoryUnitLayout::Matrix2x3 => MemoryUnitLayout::Matrix2x3,
            dynamic::MemoryUnitLayout::Matrix2x4 => MemoryUnitLayout::Matrix2x4,
            dynamic::MemoryUnitLayout::Matrix3x2 => MemoryUnitLayout::Matrix3x2,
            dynamic::MemoryUnitLayout::Matrix3x3 => MemoryUnitLayout::Matrix3x3,
            dynamic::MemoryUnitLayout::Matrix3x4 => MemoryUnitLayout::Matrix3x4,
            dynamic::MemoryUnitLayout::Matrix4x2 => MemoryUnitLayout::Matrix4x2,
            dynamic::MemoryUnitLayout::Matrix4x3 => MemoryUnitLayout::Matrix4x3,
            dynamic::MemoryUnitLayout::Matrix4x4 => MemoryUnitLayout::Matrix4x4,
            dynamic::MemoryUnitLayout::Array(layout) => {
                MemoryUnitLayout::Array(ArrayLayout::Dynamic(layout))
            }
        }
    }
}

impl From<&'static constant::MemoryUnitLayout> for MemoryUnitLayout<'_> {
    fn from(value: &'static constant::MemoryUnitLayout) -> Self {
        match value {
            constant::MemoryUnitLayout::Float => MemoryUnitLayout::Float,
            constant::MemoryUnitLayout::FloatVector2 => MemoryUnitLayout::FloatVector2,
            constant::MemoryUnitLayout::FloatVector3 => MemoryUnitLayout::FloatVector3,
            constant::MemoryUnitLayout::FloatVector4 => MemoryUnitLayout::FloatVector4,
            constant::MemoryUnitLayout::Integer => MemoryUnitLayout::Integer,
            constant::MemoryUnitLayout::IntegerVector2 => MemoryUnitLayout::IntegerVector2,
            constant::MemoryUnitLayout::IntegerVector3 => MemoryUnitLayout::IntegerVector3,
            constant::MemoryUnitLayout::IntegerVector4 => MemoryUnitLayout::IntegerVector4,
            constant::MemoryUnitLayout::UnsignedInteger => MemoryUnitLayout::UnsignedInteger,
            constant::MemoryUnitLayout::UnsignedIntegerVector2 => {
                MemoryUnitLayout::UnsignedIntegerVector2
            }
            constant::MemoryUnitLayout::UnsignedIntegerVector3 => {
                MemoryUnitLayout::UnsignedIntegerVector3
            }
            constant::MemoryUnitLayout::UnsignedIntegerVector4 => {
                MemoryUnitLayout::UnsignedIntegerVector4
            }
            constant::MemoryUnitLayout::Matrix2x2 => MemoryUnitLayout::Matrix2x2,
            constant::MemoryUnitLayout::Matrix2x3 => MemoryUnitLayout::Matrix2x3,
            constant::MemoryUnitLayout::Matrix2x4 => MemoryUnitLayout::Matrix2x4,
            constant::MemoryUnitLayout::Matrix3x2 => MemoryUnitLayout::Matrix3x2,
            constant::MemoryUnitLayout::Matrix3x3 => MemoryUnitLayout::Matrix3x3,
            constant::MemoryUnitLayout::Matrix3x4 => MemoryUnitLayout::Matrix3x4,
            constant::MemoryUnitLayout::Matrix4x2 => MemoryUnitLayout::Matrix4x2,
            constant::MemoryUnitLayout::Matrix4x3 => MemoryUnitLayout::Matrix4x3,
            constant::MemoryUnitLayout::Matrix4x4 => MemoryUnitLayout::Matrix4x4,
            constant::MemoryUnitLayout::Array(layout) => {
                MemoryUnitLayout::Array(ArrayLayout::Constant(layout))
            }
        }
    }
}

pub enum MemoryUnit<'a> {
    Dynamic(&'a dynamic::MemoryUnit),
    Constant(&'static constant::MemoryUnit),
}

impl<'a> MemoryUnit<'a> {
    pub fn offset(&self) -> u64 {
        match self {
            MemoryUnit::Dynamic(unit) => unit.offset,
            MemoryUnit::Constant(unit) => unit.offset,
        }
    }

    pub fn layout(&self) -> MemoryUnitLayout<'a> {
        match self {
            MemoryUnit::Dynamic(unit) => MemoryUnitLayout::from(&unit.layout),
            MemoryUnit::Constant(unit) => MemoryUnitLayout::from(&unit.layout),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SizedBufferLayout<'a> {
    Dynamic(&'a dynamic::SizedBufferLayout),
    Constant(&'static constant::SizedBufferLayout),
}

impl<'a> SizedBufferLayout<'a> {
    pub fn memory_units(&self) -> MemoryUnits<'a> {
        match self {
            SizedBufferLayout::Dynamic(layout) => MemoryUnits::Dynamic(&layout.memory_units),
            SizedBufferLayout::Constant(layout) => MemoryUnits::Constant(&layout.memory_units),
        }
    }
}

pub enum UnsizedTailLayout<'a> {
    Dynamic(&'a dynamic::UnsizedTailLayout),
    Constant(&'static constant::UnsizedTailLayout),
}

impl<'a> UnsizedTailLayout<'a> {
    pub fn offset(&self) -> u64 {
        match self {
            UnsizedTailLayout::Dynamic(layout) => layout.offset,
            UnsizedTailLayout::Constant(layout) => layout.offset,
        }
    }

    pub fn element_layout(&self) -> MemoryUnits<'a> {
        match self {
            UnsizedTailLayout::Dynamic(layout) => MemoryUnits::Dynamic(&layout.element_layout),
            UnsizedTailLayout::Constant(layout) => MemoryUnits::Constant(&layout.element_layout),
        }
    }

    pub fn stride(&self) -> u64 {
        match self {
            UnsizedTailLayout::Dynamic(layout) => layout.stride,
            UnsizedTailLayout::Constant(layout) => layout.stride,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnsizedBufferLayout<'a> {
    Dynamic(&'a dynamic::UnsizedBufferLayout),
    Constant(&'static constant::UnsizedBufferLayout),
}

impl<'a> UnsizedBufferLayout<'a> {
    pub fn sized_head(&self) -> MemoryUnits<'a> {
        match self {
            UnsizedBufferLayout::Dynamic(layout) => MemoryUnits::Dynamic(&layout.sized_head),
            UnsizedBufferLayout::Constant(layout) => MemoryUnits::Constant(&layout.sized_head),
        }
    }

    pub fn unsized_tail(&self) -> Option<UnsizedTailLayout<'a>> {
        match self {
            UnsizedBufferLayout::Dynamic(layout) => layout
                .unsized_tail
                .as_ref()
                .map(|layout| UnsizedTailLayout::Dynamic(layout)),
            UnsizedBufferLayout::Constant(layout) => layout
                .unsized_tail
                .as_ref()
                .map(|layout| UnsizedTailLayout::Constant(layout)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceType<'a> {
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
    Uniform(SizedBufferLayout<'a>),
    StorageRead(UnsizedBufferLayout<'a>),
    StorageReadWrite(UnsizedBufferLayout<'a>),
}

impl<'a> From<&'a dynamic::ResourceType> for ResourceType<'a> {
    fn from(value: &'a dynamic::ResourceType) -> Self {
        match value {
            dynamic::ResourceType::Texture1D(ty) => ResourceType::Texture1D(*ty),
            dynamic::ResourceType::Texture2D(ty) => ResourceType::Texture2D(*ty),
            dynamic::ResourceType::Texture3D(ty) => ResourceType::Texture3D(*ty),
            dynamic::ResourceType::Texture2DArray(ty) => ResourceType::Texture2DArray(*ty),
            dynamic::ResourceType::TextureCube(ty) => ResourceType::TextureCube(*ty),
            dynamic::ResourceType::TextureCubeArray(ty) => ResourceType::TextureCubeArray(*ty),
            dynamic::ResourceType::TextureMultisampled2D(ty) => {
                ResourceType::TextureMultisampled2D(*ty)
            }
            dynamic::ResourceType::TextureDepth2D => ResourceType::TextureDepth2D,
            dynamic::ResourceType::TextureDepth2DArray => ResourceType::TextureDepth2DArray,
            dynamic::ResourceType::TextureDepthCube => ResourceType::TextureDepthCube,
            dynamic::ResourceType::TextureDepthCubeArray => ResourceType::TextureDepthCubeArray,
            dynamic::ResourceType::TextureDepthMultisampled2D => {
                ResourceType::TextureDepthMultisampled2D
            }
            dynamic::ResourceType::StorageTexture1D(format) => {
                ResourceType::StorageTexture1D(*format)
            }
            dynamic::ResourceType::StorageTexture2D(format) => {
                ResourceType::StorageTexture2D(*format)
            }
            dynamic::ResourceType::StorageTexture2DArray(format) => {
                ResourceType::StorageTexture2DArray(*format)
            }
            dynamic::ResourceType::StorageTexture3D(format) => {
                ResourceType::StorageTexture3D(*format)
            }
            dynamic::ResourceType::FilteringSampler => ResourceType::FilteringSampler,
            dynamic::ResourceType::NonFilteringSampler => ResourceType::NonFilteringSampler,
            dynamic::ResourceType::ComparisonSampler => ResourceType::ComparisonSampler,
            dynamic::ResourceType::Uniform(layout) => {
                ResourceType::Uniform(SizedBufferLayout::Dynamic(layout))
            }
            dynamic::ResourceType::StorageRead(layout) => {
                ResourceType::StorageRead(UnsizedBufferLayout::Dynamic(layout))
            }
            dynamic::ResourceType::StorageReadWrite(layout) => {
                ResourceType::StorageReadWrite(UnsizedBufferLayout::Dynamic(layout))
            }
        }
    }
}

impl From<&'static constant::ResourceType> for ResourceType<'_> {
    fn from(value: &'static constant::ResourceType) -> Self {
        match value {
            constant::ResourceType::Texture1D(ty) => ResourceType::Texture1D(*ty),
            constant::ResourceType::Texture2D(ty) => ResourceType::Texture2D(*ty),
            constant::ResourceType::Texture3D(ty) => ResourceType::Texture3D(*ty),
            constant::ResourceType::Texture2DArray(ty) => ResourceType::Texture2DArray(*ty),
            constant::ResourceType::TextureCube(ty) => ResourceType::TextureCube(*ty),
            constant::ResourceType::TextureCubeArray(ty) => ResourceType::TextureCubeArray(*ty),
            constant::ResourceType::TextureMultisampled2D(ty) => {
                ResourceType::TextureMultisampled2D(*ty)
            }
            constant::ResourceType::TextureDepth2D => ResourceType::TextureDepth2D,
            constant::ResourceType::TextureDepth2DArray => ResourceType::TextureDepth2DArray,
            constant::ResourceType::TextureDepthCube => ResourceType::TextureDepthCube,
            constant::ResourceType::TextureDepthCubeArray => ResourceType::TextureDepthCubeArray,
            constant::ResourceType::TextureDepthMultisampled2D => {
                ResourceType::TextureDepthMultisampled2D
            }
            constant::ResourceType::StorageTexture1D(format) => {
                ResourceType::StorageTexture1D(*format)
            }
            constant::ResourceType::StorageTexture2D(format) => {
                ResourceType::StorageTexture2D(*format)
            }
            constant::ResourceType::StorageTexture2DArray(format) => {
                ResourceType::StorageTexture2DArray(*format)
            }
            constant::ResourceType::StorageTexture3D(format) => {
                ResourceType::StorageTexture3D(*format)
            }
            constant::ResourceType::FilteringSampler => ResourceType::FilteringSampler,
            constant::ResourceType::NonFilteringSampler => ResourceType::NonFilteringSampler,
            constant::ResourceType::ComparisonSampler => ResourceType::ComparisonSampler,
            constant::ResourceType::Uniform(layout) => {
                ResourceType::Uniform(SizedBufferLayout::Constant(layout))
            }
            constant::ResourceType::StorageRead(layout) => {
                ResourceType::StorageRead(UnsizedBufferLayout::Constant(layout))
            }
            constant::ResourceType::StorageReadWrite(layout) => {
                ResourceType::StorageReadWrite(UnsizedBufferLayout::Constant(layout))
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceBinding<'a> {
    Dynamic(&'a dynamic::ResourceBinding),
    Constant(&'static constant::ResourceBinding),
}

impl<'a> ResourceBinding<'a> {
    pub fn group(&self) -> u32 {
        match self {
            ResourceBinding::Dynamic(binding) => binding.group,
            ResourceBinding::Constant(binding) => binding.group,
        }
    }

    pub fn binding(&self) -> u32 {
        match self {
            ResourceBinding::Dynamic(binding) => binding.binding,
            ResourceBinding::Constant(binding) => binding.binding,
        }
    }

    pub fn resource_type(&self) -> ResourceType<'a> {
        match self {
            ResourceBinding::Dynamic(binding) => ResourceType::from(&binding.resource_type),
            ResourceBinding::Constant(binding) => ResourceType::from(&binding.resource_type),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntryPoint<'a> {
    Dynamic(&'a dynamic::EntryPoint),
    Constant(&'static constant::EntryPoint),
}

impl<'a> EntryPoint<'a> {
    pub fn stage(&self) -> ShaderStage {
        match self {
            EntryPoint::Dynamic(entry_point) => entry_point.stage,
            EntryPoint::Constant(entry_point) => entry_point.stage,
        }
    }

    pub fn name(&self) -> &'a str {
        match self {
            EntryPoint::Dynamic(entry_point) => &entry_point.name,
            EntryPoint::Constant(entry_point) => entry_point.name,
        }
    }

    pub fn input_bindings(&self) -> &[IoBinding] {
        match self {
            EntryPoint::Dynamic(entry_point) => &entry_point.input_bindings,
            EntryPoint::Constant(entry_point) => entry_point.input_bindings,
        }
    }

    pub fn output_bindings(&self) -> &[IoBinding] {
        match self {
            EntryPoint::Dynamic(entry_point) => &entry_point.output_bindings,
            EntryPoint::Constant(entry_point) => entry_point.output_bindings,
        }
    }

    pub fn overridable_constants(&self) -> &[usize] {
        match self {
            EntryPoint::Dynamic(entry_point) => &entry_point.overridable_constants,
            EntryPoint::Constant(entry_point) => entry_point.overridable_constants,
        }
    }

    pub fn resource_bindings(&self) -> &[usize] {
        match self {
            EntryPoint::Dynamic(entry_point) => &entry_point.resource_bindings,
            EntryPoint::Constant(entry_point) => entry_point.resource_bindings,
        }
    }
}

macro_rules! gen_slice_wrapper {
    ($wrapper:ident, $element:ident, $iter:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum $wrapper<'a> {
            Dynamic(&'a [dynamic::$element]),
            Constant(&'static [constant::$element]),
        }

        impl<'a> $wrapper<'a> {
            pub fn get(&self, index: usize) -> Option<$element<'a>> {
                match self {
                    $wrapper::Dynamic(bindings) => bindings
                        .get(index)
                        .map(|binding| $element::Dynamic(binding)),
                    $wrapper::Constant(bindings) => bindings
                        .get(index)
                        .map(|binding| $element::Constant(binding)),
                }
            }

            pub fn len(&self) -> usize {
                match self {
                    $wrapper::Dynamic(bindings) => bindings.len(),
                    $wrapper::Constant(bindings) => bindings.len(),
                }
            }

            pub fn iter(&self) -> $iter<'a> {
                self.into_iter()
            }
        }

        pub struct $iter<'a> {
            bindings: $wrapper<'a>,
            index: usize,
        }

        impl<'a> Iterator for $iter<'a> {
            type Item = $element<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                let index = self.index;

                self.index += 1;

                self.bindings.get(index)
            }
        }

        impl<'a> IntoIterator for $wrapper<'a> {
            type Item = $element<'a>;
            type IntoIter = $iter<'a>;

            fn into_iter(self) -> Self::IntoIter {
                $iter {
                    bindings: self,
                    index: 0,
                }
            }
        }
    };
}

gen_slice_wrapper!(MemoryUnits, MemoryUnit, MemoryUnitsIter);
gen_slice_wrapper!(ResourceBindings, ResourceBinding, ResourceBindingsIter);
gen_slice_wrapper!(EntryPoints, EntryPoint, EntryPointsIter);

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ShaderModuleInterface {
    Dynamic(dynamic::ShaderModuleInterface),
    Constant(constant::ShaderModuleInterface),
}

impl ShaderModuleInterface {
    pub fn overridable_constants(&self) -> &[OverridableConstant] {
        match self {
            ShaderModuleInterface::Dynamic(dynamic) => &dynamic.overridable_constants,
            ShaderModuleInterface::Constant(constant) => constant.overridable_constants,
        }
    }

    pub fn resource_bindings(&self) -> ResourceBindings<'_> {
        match self {
            ShaderModuleInterface::Dynamic(smi) => {
                ResourceBindings::Dynamic(&smi.resource_bindings)
            }
            ShaderModuleInterface::Constant(smi) => {
                ResourceBindings::Constant(smi.resource_bindings)
            }
        }
    }

    pub fn entry_points(&self) -> EntryPoints<'_> {
        match self {
            ShaderModuleInterface::Dynamic(smi) => EntryPoints::Dynamic(&smi.entry_points),
            ShaderModuleInterface::Constant(smi) => EntryPoints::Constant(smi.entry_points),
        }
    }
}
