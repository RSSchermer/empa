use std::convert::TryFrom;
use std::ops::Deref;

use naga::front::wgsl;
use naga::proc::IndexableLength;
use naga::{AddressSpace, Module, Override, ScalarKind};
pub use wgsl::ParseError;

#[derive(Clone, Debug)]
pub struct ShaderSource {
    source: String,
    module: Module,
    resource_bindings: Vec<ShaderResourceBinding>,
    constants: Vec<Constant>,
    entry_points: Vec<EntryPoint>,
}

impl ShaderSource {
    pub fn parse(source: String) -> Result<ShaderSource, ParseError> {
        let module = wgsl::parse_str(&source)?;

        let mut resource_bindings = Vec::new();

        for (_, global) in module.global_variables.iter() {
            if let Some(naga::ResourceBinding { group, binding }) = global.binding {
                resource_bindings.push(ShaderResourceBinding {
                    group,
                    binding,
                    binding_type: BindingType::try_from_naga(&module, &global.space, global.ty)
                        .unwrap(),
                });
            }
        }

        let constants = module.overrides.iter().map(|(_, c)| {
            let ty = module.types.get_handle(c.ty).unwrap();

            Constant {
                identifier: ConstantIdentifier::from_naga(c),
                constant_type: ConstantType::from_naga(ty),
                required: c.init.is_none(),
            }
        }).collect();

        let mut entry_points = Vec::new();

        for entry_point in module.entry_points.iter() {
            entry_points.push(EntryPoint::try_from_naga(&module, entry_point).unwrap());
        }

        Ok(ShaderSource {
            source,
            module,
            resource_bindings,
            constants,
            entry_points,
        })
    }

    pub fn raw_str(&self) -> &str {
        &self.source
    }

    pub fn module(&self) -> &Module {
        &self.module
    }

    pub fn resource_bindings(&self) -> &[ShaderResourceBinding] {
        &self.resource_bindings
    }

    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }

    pub fn entry_points(&self) -> &[EntryPoint] {
        &self.entry_points
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConstantIdentifier {
    Number(u32),
    Name(String),
}

impl ConstantIdentifier {
    fn from_naga(value: &Override) -> Self {
        if let Some(id) = value.id {
            ConstantIdentifier::Number(id as u32)
        } else {
            ConstantIdentifier::Name(value.name.clone().expect("override constant should have name or ID"))
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Constant {
    identifier: ConstantIdentifier,
    constant_type: ConstantType,
    required: bool,
}

impl Constant {
    pub fn identifier(&self) -> &ConstantIdentifier {
        &self.identifier
    }

    pub fn constant_type(&self) -> ConstantType {
        self.constant_type
    }

    pub fn required(&self) -> bool {
        self.required
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConstantType {
    Float,
    Bool,
    SignedInteger,
    UnsignedInteger,
}

impl ConstantType {
    fn from_naga(value: &naga::Type) -> Self {
        if let naga::TypeInner::Scalar(scalar) = &value.inner {
            match &scalar.kind {
                ScalarKind::Sint => ConstantType::SignedInteger,
                ScalarKind::Uint => ConstantType::UnsignedInteger,
                ScalarKind::Float => ConstantType::Float,
                ScalarKind::Bool => ConstantType::Bool,
                _ => unreachable!("constant type must be concrete")
            }
        } else {
            unreachable!("constant type must be scalar");
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

impl From<&'_ naga::ShaderStage> for ShaderStage {
    fn from(value: &naga::ShaderStage) -> Self {
        match value {
            naga::ShaderStage::Vertex => ShaderStage::Vertex,
            naga::ShaderStage::Fragment => ShaderStage::Fragment,
            naga::ShaderStage::Compute => ShaderStage::Compute,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ShaderResourceBinding {
    group: u32,
    binding: u32,
    binding_type: BindingType,
}

impl ShaderResourceBinding {
    pub fn group(&self) -> u32 {
        self.group
    }

    pub fn binding(&self) -> u32 {
        self.binding
    }

    pub fn binding_type(&self) -> &BindingType {
        &self.binding_type
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum BindingType {
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
    Storage(UnsizedBufferLayout),
    ReadOnlyStorage(UnsizedBufferLayout),
}

impl BindingType {
    fn try_from_naga(
        module: &naga::Module,
        space: &naga::AddressSpace,
        type_handle: naga::Handle<naga::Type>,
    ) -> Result<Self, ()> {
        match space {
            AddressSpace::Uniform => {
                let layout = SizedBufferLayout::try_from_naga(module, type_handle)?;

                Ok(BindingType::Uniform(layout))
            }
            AddressSpace::Storage { access } => {
                if *access == naga::StorageAccess::all() {
                    let layout = UnsizedBufferLayout::try_from_naga(module, type_handle)?;

                    Ok(BindingType::Storage(layout))
                } else if *access == naga::StorageAccess::LOAD {
                    let layout = UnsizedBufferLayout::try_from_naga(module, type_handle)?;

                    Ok(BindingType::ReadOnlyStorage(layout))
                } else {
                    Err(())
                }
            }
            AddressSpace::Handle => {
                let ty = module.types.get_handle(type_handle).unwrap();

                match &ty.inner {
                    naga::TypeInner::Image {
                        dim,
                        arrayed,
                        class,
                    } => match (dim, arrayed, class) {
                        (
                            naga::ImageDimension::D1,
                            false,
                            naga::ImageClass::Sampled { kind, multi: false },
                        ) => {
                            let texel_type = TexelType::try_from(*kind)?;

                            Ok(BindingType::Texture1D(texel_type))
                        }
                        (
                            naga::ImageDimension::D2,
                            false,
                            naga::ImageClass::Sampled { kind, multi: false },
                        ) => {
                            let texel_type = TexelType::try_from(*kind)?;

                            Ok(BindingType::Texture2D(texel_type))
                        }
                        (
                            naga::ImageDimension::D3,
                            false,
                            naga::ImageClass::Sampled { kind, multi: false },
                        ) => {
                            let texel_type = TexelType::try_from(*kind)?;

                            Ok(BindingType::Texture3D(texel_type))
                        }
                        (
                            naga::ImageDimension::D2,
                            true,
                            naga::ImageClass::Sampled { kind, multi: false },
                        ) => {
                            let texel_type = TexelType::try_from(*kind)?;

                            Ok(BindingType::Texture2DArray(texel_type))
                        }
                        (
                            naga::ImageDimension::Cube,
                            false,
                            naga::ImageClass::Sampled { kind, multi: false },
                        ) => {
                            let texel_type = TexelType::try_from(*kind)?;

                            Ok(BindingType::TextureCube(texel_type))
                        }
                        (
                            naga::ImageDimension::Cube,
                            true,
                            naga::ImageClass::Sampled { kind, multi: false },
                        ) => {
                            let texel_type = TexelType::try_from(*kind)?;

                            Ok(BindingType::TextureCubeArray(texel_type))
                        }
                        (
                            naga::ImageDimension::D2,
                            false,
                            naga::ImageClass::Sampled { kind, multi: true },
                        ) => {
                            let texel_type = TexelType::try_from(*kind)?;

                            Ok(BindingType::TextureMultisampled2D(texel_type))
                        }
                        (naga::ImageDimension::D2, false, naga::ImageClass::Depth { .. }) => {
                            Ok(BindingType::TextureDepth2D)
                        }
                        (naga::ImageDimension::D2, true, naga::ImageClass::Depth { .. }) => {
                            Ok(BindingType::TextureDepth2DArray)
                        }
                        (naga::ImageDimension::Cube, false, naga::ImageClass::Depth { .. }) => {
                            Ok(BindingType::TextureDepthCube)
                        }
                        (naga::ImageDimension::Cube, true, naga::ImageClass::Depth { .. }) => {
                            Ok(BindingType::TextureDepthCubeArray)
                        }
                        (
                            naga::ImageDimension::D1,
                            false,
                            naga::ImageClass::Storage { format, .. },
                        ) => {
                            let format = StorageTextureFormat::try_from(*format)?;

                            Ok(BindingType::StorageTexture1D(format))
                        }
                        (
                            naga::ImageDimension::D2,
                            false,
                            naga::ImageClass::Storage { format, .. },
                        ) => {
                            let format = StorageTextureFormat::try_from(*format)?;

                            Ok(BindingType::StorageTexture2D(format))
                        }
                        (
                            naga::ImageDimension::D2,
                            true,
                            naga::ImageClass::Storage { format, .. },
                        ) => {
                            let format = StorageTextureFormat::try_from(*format)?;

                            Ok(BindingType::StorageTexture2DArray(format))
                        }
                        (
                            naga::ImageDimension::D3,
                            false,
                            naga::ImageClass::Storage { format, .. },
                        ) => {
                            let format = StorageTextureFormat::try_from(*format)?;

                            Ok(BindingType::StorageTexture3D(format))
                        }
                        _ => Err(()),
                    },
                    // TODO: non-filtering sampler
                    naga::TypeInner::Sampler { comparison: true } => {
                        Ok(BindingType::ComparisonSampler)
                    }
                    naga::TypeInner::Sampler { comparison: false } => {
                        Ok(BindingType::FilteringSampler)
                    }
                    _ => Err(()),
                }
            }
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TexelType {
    Float,
    UnfilterableFloat,
    Integer,
    UnsignedInteger,
}

impl TryFrom<naga::ScalarKind> for TexelType {
    type Error = ();

    fn try_from(value: naga::ScalarKind) -> Result<Self, Self::Error> {
        // TODO: unfiltered float
        match value {
            naga::ScalarKind::Sint => Ok(TexelType::Integer),
            naga::ScalarKind::Uint => Ok(TexelType::UnsignedInteger),
            naga::ScalarKind::Float => Ok(TexelType::Float),
            naga::ScalarKind::Bool
            | naga::ScalarKind::AbstractInt
            | naga::ScalarKind::AbstractFloat => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

impl TryFrom<naga::StorageFormat> for StorageTextureFormat {
    type Error = ();

    fn try_from(format: naga::StorageFormat) -> Result<Self, Self::Error> {
        match format {
            naga::StorageFormat::R32Uint => Ok(StorageTextureFormat::r32uint),
            naga::StorageFormat::R32Sint => Ok(StorageTextureFormat::r32sint),
            naga::StorageFormat::R32Float => Ok(StorageTextureFormat::r32float),
            naga::StorageFormat::Rgba8Unorm => Ok(StorageTextureFormat::rgba8unorm),
            naga::StorageFormat::Rgba8Snorm => Ok(StorageTextureFormat::rgba8snorm),
            naga::StorageFormat::Rgba8Uint => Ok(StorageTextureFormat::rgba8uint),
            naga::StorageFormat::Rgba8Sint => Ok(StorageTextureFormat::rgba8sint),
            naga::StorageFormat::Rg32Uint => Ok(StorageTextureFormat::rg32uint),
            naga::StorageFormat::Rg32Sint => Ok(StorageTextureFormat::rg32sint),
            naga::StorageFormat::Rg32Float => Ok(StorageTextureFormat::rg32float),
            naga::StorageFormat::Rgba16Uint => Ok(StorageTextureFormat::rgba16uint),
            naga::StorageFormat::Rgba16Sint => Ok(StorageTextureFormat::rgba16sint),
            naga::StorageFormat::Rgba16Float => Ok(StorageTextureFormat::rgba16float),
            naga::StorageFormat::Rgba32Uint => Ok(StorageTextureFormat::rgba32uint),
            naga::StorageFormat::Rgba32Sint => Ok(StorageTextureFormat::rgba32sint),
            naga::StorageFormat::Rgba32Float => Ok(StorageTextureFormat::rgba32float),
            _ => Err(()),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SizedBufferLayout(Vec<MemoryUnit>);

impl SizedBufferLayout {
    fn try_from_naga(
        module: &naga::Module,
        type_handle: naga::Handle<naga::Type>,
    ) -> Result<Self, ()> {
        let mut head_units = Vec::new();
        let mut tail_units = None;

        collect_units(0, module, type_handle, &mut head_units, &mut tail_units)?;

        if tail_units.is_some() {
            return Err(());
        }

        Ok(SizedBufferLayout(head_units))
    }

    pub fn memory_units(&self) -> &[MemoryUnit] {
        &self.0
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct UnsizedBufferLayout {
    sized_head: Vec<MemoryUnit>,
    unsized_tail: Option<Vec<MemoryUnit>>,
}

impl UnsizedBufferLayout {
    fn try_from_naga(
        module: &naga::Module,
        type_handle: naga::Handle<naga::Type>,
    ) -> Result<Self, ()> {
        let mut head_units = Vec::new();
        let mut tail_units = None;

        collect_units(0, module, type_handle, &mut head_units, &mut tail_units)?;

        Ok(UnsizedBufferLayout {
            sized_head: head_units,
            unsized_tail: tail_units,
        })
    }

    pub fn sized_head(&self) -> &[MemoryUnit] {
        &self.sized_head
    }

    pub fn unsized_tail(&self) -> Option<&[MemoryUnit]> {
        self.unsized_tail.as_ref().map(|t| t.deref())
    }
}

#[derive(Clone, Debug)]
pub struct EntryPoint {
    name: String,
    stage: ShaderStage,
    input_bindings: Vec<EntryPointBinding>,
    output_bindings: Vec<EntryPointBinding>,
}

impl EntryPoint {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn stage(&self) -> ShaderStage {
        self.stage
    }

    pub fn input_bindings(&self) -> &[EntryPointBinding] {
        &self.input_bindings
    }

    pub fn output_bindings(&self) -> &[EntryPointBinding] {
        &self.output_bindings
    }
}

impl EntryPoint {
    fn try_from_naga(module: &naga::Module, entry_point: &naga::EntryPoint) -> Result<Self, ()> {
        fn collect_bindings(
            module: &naga::Module,
            binding: Option<&naga::Binding>,
            type_handle: naga::Handle<naga::Type>,
            sink: &mut Vec<EntryPointBinding>,
        ) -> Result<(), ()> {
            let ty = module.types.get_handle(type_handle).unwrap();

            if let Some(naga::Binding::Location {
                location,
                interpolation,
                sampling,
                ..
            }) = binding
            {
                sink.push(EntryPointBinding::try_from_naga(
                    *location,
                    *interpolation,
                    *sampling,
                    &ty.inner,
                )?);
            }

            if let naga::TypeInner::Struct { members, .. } = &ty.inner {
                for member in members {
                    let binding = member.binding.as_ref().ok_or(())?;
                    let ty = module.types.get_handle(member.ty).unwrap();

                    if let naga::Binding::Location {
                        location,
                        interpolation,
                        sampling,
                        ..
                    } = binding
                    {
                        sink.push(EntryPointBinding::try_from_naga(
                            *location,
                            *interpolation,
                            *sampling,
                            &ty.inner,
                        )?);
                    }
                }
            }

            Ok(())
        }

        let mut input_bindings = Vec::new();

        for argument in entry_point.function.arguments.iter() {
            collect_bindings(
                module,
                argument.binding.as_ref(),
                argument.ty,
                &mut input_bindings,
            )?;
        }

        let mut output_bindings = Vec::new();

        if let Some(result) = &entry_point.function.result {
            collect_bindings(
                module,
                result.binding.as_ref(),
                result.ty,
                &mut output_bindings,
            )?;
        }

        Ok(EntryPoint {
            name: entry_point.name.to_string(),
            stage: ShaderStage::from(&entry_point.stage),
            input_bindings,
            output_bindings,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct EntryPointBinding {
    location: u32,
    binding_type: EntryPointBindingType,
    interpolation: Option<Interpolation>,
    sampling: Option<Sampling>,
}

impl EntryPointBinding {
    fn try_from_naga(
        location: u32,
        interpolation: Option<naga::Interpolation>,
        sampling: Option<naga::Sampling>,
        ty: &naga::TypeInner,
    ) -> Result<Self, ()> {
        let binding_type = EntryPointBindingType::try_from(ty)?;

        Ok(EntryPointBinding {
            location,
            binding_type,
            interpolation: interpolation.map(|i| i.into()),
            sampling: sampling.map(|s| s.into()),
        })
    }

    pub fn location(&self) -> u32 {
        self.location
    }

    pub fn binding_type(&self) -> EntryPointBindingType {
        self.binding_type
    }

    pub fn interpolation(&self) -> Option<Interpolation> {
        self.interpolation
    }

    pub fn sampling(&self) -> Option<Sampling> {
        self.sampling
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EntryPointBindingType {
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

impl EntryPointBindingType {
    pub fn is_float(&self) -> bool {
        match self {
            EntryPointBindingType::Float
            | EntryPointBindingType::FloatVector2
            | EntryPointBindingType::FloatVector3
            | EntryPointBindingType::FloatVector4 => true,
            _ => false,
        }
    }

    pub fn is_half_float(&self) -> bool {
        match self {
            EntryPointBindingType::HalfFloat
            | EntryPointBindingType::HalfFloatVector2
            | EntryPointBindingType::HalfFloatVector3
            | EntryPointBindingType::HalfFloatVector4 => true,
            _ => false,
        }
    }

    pub fn is_signed_integer(&self) -> bool {
        match self {
            EntryPointBindingType::SignedInteger
            | EntryPointBindingType::SignedIntegerVector2
            | EntryPointBindingType::SignedIntegerVector3
            | EntryPointBindingType::SignedIntegerVector4 => true,
            _ => false,
        }
    }

    pub fn is_unsigned_integer(&self) -> bool {
        match self {
            EntryPointBindingType::UnsignedInteger
            | EntryPointBindingType::UnsignedIntegerVector2
            | EntryPointBindingType::UnsignedIntegerVector3
            | EntryPointBindingType::UnsignedIntegerVector4 => true,
            _ => false,
        }
    }
}

impl TryFrom<&'_ naga::TypeInner> for EntryPointBindingType {
    type Error = ();

    fn try_from(value: &naga::TypeInner) -> Result<Self, Self::Error> {
        // TODO: half-float not currently in naga
        match value {
            naga::TypeInner::Scalar(naga::Scalar {
                kind: naga::ScalarKind::Float,
                ..
            }) => Ok(EntryPointBindingType::Float),
            naga::TypeInner::Scalar(naga::Scalar {
                kind: naga::ScalarKind::Sint,
                ..
            }) => Ok(EntryPointBindingType::SignedInteger),
            naga::TypeInner::Scalar(naga::Scalar {
                kind: naga::ScalarKind::Uint,
                ..
            }) => Ok(EntryPointBindingType::UnsignedInteger),
            naga::TypeInner::Vector {
                scalar:
                    naga::Scalar {
                        kind: naga::ScalarKind::Float,
                        ..
                    },
                size: naga::VectorSize::Bi,
                ..
            } => Ok(EntryPointBindingType::FloatVector2),
            naga::TypeInner::Vector {
                scalar:
                    naga::Scalar {
                        kind: naga::ScalarKind::Float,
                        ..
                    },
                size: naga::VectorSize::Tri,
                ..
            } => Ok(EntryPointBindingType::FloatVector3),
            naga::TypeInner::Vector {
                scalar:
                    naga::Scalar {
                        kind: naga::ScalarKind::Float,
                        ..
                    },
                size: naga::VectorSize::Quad,
                ..
            } => Ok(EntryPointBindingType::FloatVector4),
            naga::TypeInner::Vector {
                scalar:
                    naga::Scalar {
                        kind: naga::ScalarKind::Sint,
                        ..
                    },
                size: naga::VectorSize::Bi,
                ..
            } => Ok(EntryPointBindingType::SignedIntegerVector2),
            naga::TypeInner::Vector {
                scalar:
                    naga::Scalar {
                        kind: naga::ScalarKind::Sint,
                        ..
                    },
                size: naga::VectorSize::Tri,
                ..
            } => Ok(EntryPointBindingType::SignedIntegerVector3),
            naga::TypeInner::Vector {
                scalar:
                    naga::Scalar {
                        kind: naga::ScalarKind::Sint,
                        ..
                    },
                size: naga::VectorSize::Quad,
                ..
            } => Ok(EntryPointBindingType::SignedIntegerVector4),
            naga::TypeInner::Vector {
                scalar:
                    naga::Scalar {
                        kind: naga::ScalarKind::Uint,
                        ..
                    },
                size: naga::VectorSize::Bi,
                ..
            } => Ok(EntryPointBindingType::UnsignedIntegerVector2),
            naga::TypeInner::Vector {
                scalar:
                    naga::Scalar {
                        kind: naga::ScalarKind::Uint,
                        ..
                    },
                size: naga::VectorSize::Tri,
                ..
            } => Ok(EntryPointBindingType::UnsignedIntegerVector3),
            naga::TypeInner::Vector {
                scalar:
                    naga::Scalar {
                        kind: naga::ScalarKind::Uint,
                        ..
                    },
                size: naga::VectorSize::Quad,
                ..
            } => Ok(EntryPointBindingType::UnsignedIntegerVector4),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Interpolation {
    Perspective,
    Linear,
    Flat,
}

impl From<naga::Interpolation> for Interpolation {
    fn from(interpolation: naga::Interpolation) -> Self {
        match interpolation {
            naga::Interpolation::Perspective => Interpolation::Perspective,
            naga::Interpolation::Linear => Interpolation::Linear,
            naga::Interpolation::Flat => Interpolation::Flat,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sampling {
    Center,
    Centroid,
    Sample,
}

impl From<naga::Sampling> for Sampling {
    fn from(sampling: naga::Sampling) -> Self {
        match sampling {
            naga::Sampling::Center => Sampling::Center,
            naga::Sampling::Centroid => Sampling::Centroid,
            naga::Sampling::Sample => Sampling::Sample,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MemoryUnit {
    pub offset: usize,
    pub layout: MemoryUnitLayout,
}

#[derive(Clone, PartialEq, Eq, Debug)]
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
    ComplexArray {
        units: Vec<MemoryUnit>,
        stride: usize,
        len: usize,
    },
}

fn collect_units(
    offset: usize,
    module: &naga::Module,
    type_handle: naga::Handle<naga::Type>,
    head: &mut Vec<MemoryUnit>,
    tail: &mut Option<Vec<MemoryUnit>>,
) -> Result<(), ()> {
    if tail.is_some() {
        // Cannot add more units after dynamically sized array
        return Err(());
    }

    let ty = module.types.get_handle(type_handle).unwrap();

    match &ty.inner {
        naga::TypeInner::Scalar(naga::Scalar {
            kind: naga::ScalarKind::Float,
            ..
        }) => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Float,
        }),
        naga::TypeInner::Scalar(naga::Scalar {
            kind: naga::ScalarKind::Sint,
            ..
        }) => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Integer,
        }),
        naga::TypeInner::Scalar(naga::Scalar {
            kind: naga::ScalarKind::Uint,
            ..
        }) => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::UnsignedInteger,
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Float,
                    ..
                },
            size: naga::VectorSize::Bi,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::FloatVector2,
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Float,
                    ..
                },
            size: naga::VectorSize::Tri,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::FloatVector3,
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Float,
                    ..
                },
            size: naga::VectorSize::Quad,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::FloatVector4,
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Sint,
                    ..
                },
            size: naga::VectorSize::Bi,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::IntegerVector2,
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Sint,
                    ..
                },
            size: naga::VectorSize::Tri,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::IntegerVector3,
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Sint,
                    ..
                },
            size: naga::VectorSize::Quad,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::IntegerVector4,
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Uint,
                    ..
                },
            size: naga::VectorSize::Bi,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::UnsignedIntegerVector2,
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Uint,
                    ..
                },
            size: naga::VectorSize::Tri,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::UnsignedIntegerVector3,
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Uint,
                    ..
                },
            size: naga::VectorSize::Quad,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::UnsignedIntegerVector4,
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Bi,
            rows: naga::VectorSize::Bi,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix2x2,
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Bi,
            rows: naga::VectorSize::Tri,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix2x3,
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Bi,
            rows: naga::VectorSize::Quad,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix2x4,
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Tri,
            rows: naga::VectorSize::Bi,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix3x2,
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Tri,
            rows: naga::VectorSize::Tri,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix3x3,
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Tri,
            rows: naga::VectorSize::Quad,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix3x4,
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Quad,
            rows: naga::VectorSize::Bi,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix4x2,
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Quad,
            rows: naga::VectorSize::Tri,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix4x3,
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Quad,
            rows: naga::VectorSize::Quad,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix4x4,
        }),
        naga::TypeInner::Atomic(naga::Scalar {
            kind: naga::ScalarKind::Sint,
            ..
        }) => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Integer,
        }),
        naga::TypeInner::Atomic(naga::Scalar {
            kind: naga::ScalarKind::Uint,
            ..
        }) => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::UnsignedInteger,
        }),
        naga::TypeInner::Array { base, size, stride } => {
            match size.to_indexable_length(module).map_err(|_| ())? {
                IndexableLength::Known(size) => {
                    collect_array_units(offset, module, *base, size, *stride, head)?;
                }
                IndexableLength::Dynamic => {
                    let mut units = Vec::new();
                    let mut nested_tail = None;

                    collect_units(offset, module, *base, &mut units, &mut nested_tail)?;

                    if nested_tail.is_some() {
                        // Cannot have a dynamically sized array inside of a dynamically size array
                        return Err(());
                    }

                    *tail = Some(units);
                }
            }
        }
        naga::TypeInner::Struct { members, .. } => {
            collect_struct_units(offset, module, members, head, tail)?;
        }
        _ => return Err(()),
    };

    Ok(())
}

fn collect_array_units(
    offset: usize,
    module: &naga::Module,
    type_handle: naga::Handle<naga::Type>,
    len: u32,
    stride: u32,
    head: &mut Vec<MemoryUnit>,
) -> Result<(), ()> {
    let len = len as usize;
    let ty = module.types.get_handle(type_handle).unwrap();

    match &ty.inner {
        naga::TypeInner::Scalar(naga::Scalar {
            kind: naga::ScalarKind::Float,
            ..
        }) => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::FloatArray(len),
        }),
        naga::TypeInner::Scalar(naga::Scalar {
            kind: naga::ScalarKind::Sint,
            ..
        }) => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::IntegerArray(len),
        }),
        naga::TypeInner::Scalar(naga::Scalar {
            kind: naga::ScalarKind::Uint,
            ..
        }) => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::UnsignedIntegerArray(len),
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Float,
                    ..
                },
            size: naga::VectorSize::Bi,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::FloatVector2Array(len),
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Float,
                    ..
                },
            size: naga::VectorSize::Tri,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::FloatVector3Array(len),
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Float,
                    ..
                },
            size: naga::VectorSize::Quad,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::FloatVector4Array(len),
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Sint,
                    ..
                },
            size: naga::VectorSize::Bi,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::IntegerVector2Array(len),
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Sint,
                    ..
                },
            size: naga::VectorSize::Tri,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::IntegerVector3Array(len),
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Sint,
                    ..
                },
            size: naga::VectorSize::Quad,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::IntegerVector4Array(len),
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Uint,
                    ..
                },
            size: naga::VectorSize::Bi,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::UnsignedIntegerVector2Array(len),
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Uint,
                    ..
                },
            size: naga::VectorSize::Tri,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::UnsignedIntegerVector3Array(len),
        }),
        naga::TypeInner::Vector {
            scalar:
                naga::Scalar {
                    kind: naga::ScalarKind::Uint,
                    ..
                },
            size: naga::VectorSize::Quad,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::UnsignedIntegerVector4Array(len),
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Bi,
            rows: naga::VectorSize::Bi,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix2x2Array(len),
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Bi,
            rows: naga::VectorSize::Tri,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix2x3Array(len),
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Bi,
            rows: naga::VectorSize::Quad,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix2x4Array(len),
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Tri,
            rows: naga::VectorSize::Bi,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix3x2Array(len),
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Tri,
            rows: naga::VectorSize::Tri,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix3x3Array(len),
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Tri,
            rows: naga::VectorSize::Quad,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix3x4Array(len),
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Quad,
            rows: naga::VectorSize::Bi,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix4x2Array(len),
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Quad,
            rows: naga::VectorSize::Tri,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix4x3Array(len),
        }),
        naga::TypeInner::Matrix {
            columns: naga::VectorSize::Quad,
            rows: naga::VectorSize::Quad,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::Matrix4x4Array(len),
        }),
        naga::TypeInner::Atomic(naga::Scalar {
            kind: naga::ScalarKind::Sint,
            ..
        }) => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::IntegerArray(len),
        }),
        naga::TypeInner::Atomic(naga::Scalar {
            kind: naga::ScalarKind::Uint,
            ..
        }) => head.push(MemoryUnit {
            offset,
            layout: MemoryUnitLayout::UnsignedIntegerArray(len),
        }),
        naga::TypeInner::Array {
            base,
            size,
            stride: stride_inner,
        } => {
            let len_inner =
                if let Ok(IndexableLength::Known(len)) = size.to_indexable_length(module) {
                    len
                } else {
                    return Err(());
                };

            let mut units = Vec::new();

            collect_array_units(0, module, *base, len_inner, *stride_inner, &mut units)?;

            head.push(MemoryUnit {
                offset,
                layout: MemoryUnitLayout::ComplexArray {
                    units,
                    stride: stride as usize,
                    len,
                },
            })
        }
        naga::TypeInner::Struct { members, .. } => {
            let mut units = Vec::new();
            let mut nested_tail = None;

            collect_struct_units(0, module, members, &mut units, &mut nested_tail)?;

            if nested_tail.is_some() {
                // Cannot have a dynamically sized array inside of an array
                return Err(());
            }

            head.push(MemoryUnit {
                offset,
                layout: MemoryUnitLayout::ComplexArray {
                    units,
                    stride: stride as usize,
                    len,
                },
            })
        }
        _ => return Err(()),
    };

    Ok(())
}

fn collect_struct_units(
    offset: usize,
    module: &naga::Module,
    members: &[naga::StructMember],
    head: &mut Vec<MemoryUnit>,
    tail: &mut Option<Vec<MemoryUnit>>,
) -> Result<(), ()> {
    for member in members {
        collect_units(
            offset + member.offset as usize,
            module,
            member.ty,
            head,
            tail,
        )?;
    }

    Ok(())
}
