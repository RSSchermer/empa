use empa_smi::dynamic::{
    ArrayLayout, EntryPoint as SmiEntryPoint, MemoryUnit, MemoryUnitLayout, OverridableConstant,
    ResourceBinding, ResourceType, ShaderModuleInterface, SizedBufferLayout, UnsizedBufferLayout,
    UnsizedTailLayout,
};
use empa_smi::{
    Interpolate, InterpolationType, IoBinding, IoBindingType, OverridableConstantType,
    Sampling as SmiSampling, ShaderStage as SmiShaderStage, StorageTextureFormat, TexelType,
};
use indexmap::IndexMap;
use naga::front::wgsl;
use naga::proc::IndexableLength;
use naga::valid::{Capabilities, ModuleInfo, ValidationError, ValidationFlags, Validator};
use naga::{
    AddressSpace, Binding, EntryPoint, Handle, ImageClass, ImageDimension, Interpolation, Module,
    Override, Sampling, Scalar, ScalarKind, ShaderStage, StorageFormat, Type, TypeInner,
    VectorSize, WithSpan,
};
pub use wgsl::ParseError;

type ResourceBindingMap = IndexMap<BindingKey, ResourceBinding>;
type OverridableConstantMap = IndexMap<OverridableConstantKey, OverridableConstant>;

#[derive(Clone, Debug)]
pub enum Error {
    Parse(ParseError),
    Validation(WithSpan<ValidationError>),
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<WithSpan<ValidationError>> for Error {
    fn from(e: WithSpan<ValidationError>) -> Self {
        Self::Validation(e)
    }
}

pub fn build_smi(source: String) -> Result<ShaderModuleInterface, Error> {
    let module = wgsl::parse_str(&source)?;

    let mut validator = Validator::new(ValidationFlags::default(), Capabilities::default());

    let info = validator.validate(&module)?;

    let overridable_constants = collect_overridable_constants(&module);
    let resource_bindings = collect_resource_bindings(&module);

    let mut entry_points = module
        .entry_points
        .iter()
        .enumerate()
        .map(|ep| {
            entry_point_to_smi(
                &module,
                &info,
                ep,
                &overridable_constants,
                &resource_bindings,
            )
        })
        .collect::<Vec<_>>();

    entry_points.sort();

    Ok(ShaderModuleInterface {
        overridable_constants: overridable_constants.into_values().collect(),
        resource_bindings: resource_bindings.into_values().collect(),
        entry_points,
    })
}

fn override_to_smi(module: &Module, c: &Override) -> OverridableConstant {
    let ty = module.types.get_handle(c.ty).unwrap();

    let TypeInner::Scalar(scalar) = ty.inner else {
        panic!("overridable constants must of of scalar type");
    };

    let constant_type = match scalar.kind {
        ScalarKind::Sint => OverridableConstantType::SignedInteger,
        ScalarKind::Uint => OverridableConstantType::UnsignedInteger,
        ScalarKind::Float => OverridableConstantType::Float,
        ScalarKind::Bool => OverridableConstantType::Bool,
        ScalarKind::AbstractInt | ScalarKind::AbstractFloat => {
            panic!("abstract types cannot be part of a module's public interface")
        }
    };

    let required = c.init.is_none();

    OverridableConstant {
        name: c.name.clone().unwrap_or_default(),
        id: c.id,
        constant_type,
        required,
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct OverridableConstantKey {
    id: Option<u16>,
    name: Option<String>,
}

fn collect_overridable_constants(
    module: &Module,
) -> IndexMap<OverridableConstantKey, OverridableConstant> {
    let mut map = IndexMap::default();

    for (_, c) in module.overrides.iter() {
        map.insert(
            OverridableConstantKey {
                id: c.id,
                name: c.name.clone(),
            },
            override_to_smi(&module, c),
        );
    }

    map.sort_keys();

    map
}

fn scalar_layout(scalar_kind: ScalarKind) -> MemoryUnitLayout {
    use ScalarKind::*;

    match scalar_kind {
        Sint => MemoryUnitLayout::Integer,
        Uint => MemoryUnitLayout::UnsignedInteger,
        Float => MemoryUnitLayout::Float,
        Bool | AbstractInt | AbstractFloat => panic!("not a host-sharable scalar type"),
    }
}

fn vector_layout(scalar_kind: ScalarKind, size: VectorSize) -> MemoryUnitLayout {
    use ScalarKind::*;
    use VectorSize::*;

    match (scalar_kind, size) {
        (Sint, Bi) => MemoryUnitLayout::IntegerVector2,
        (Sint, Tri) => MemoryUnitLayout::IntegerVector3,
        (Sint, Quad) => MemoryUnitLayout::IntegerVector4,
        (Uint, Bi) => MemoryUnitLayout::UnsignedIntegerVector2,
        (Uint, Tri) => MemoryUnitLayout::UnsignedIntegerVector3,
        (Uint, Quad) => MemoryUnitLayout::UnsignedIntegerVector4,
        (Float, Bi) => MemoryUnitLayout::FloatVector2,
        (Float, Tri) => MemoryUnitLayout::FloatVector3,
        (Float, Quad) => MemoryUnitLayout::FloatVector4,
        _ => panic!("not a host-sharable vector type"),
    }
}

fn matrix_layout(
    scalar_kind: ScalarKind,
    columns: VectorSize,
    rows: VectorSize,
) -> MemoryUnitLayout {
    use ScalarKind::*;
    use VectorSize::*;

    match (scalar_kind, columns, rows) {
        (Float, Bi, Bi) => MemoryUnitLayout::Matrix2x2,
        (Float, Bi, Tri) => MemoryUnitLayout::Matrix2x3,
        (Float, Bi, Quad) => MemoryUnitLayout::Matrix2x4,
        (Float, Tri, Bi) => MemoryUnitLayout::Matrix3x2,
        (Float, Tri, Tri) => MemoryUnitLayout::Matrix3x3,
        (Float, Tri, Quad) => MemoryUnitLayout::Matrix3x4,
        (Float, Quad, Bi) => MemoryUnitLayout::Matrix4x2,
        (Float, Quad, Tri) => MemoryUnitLayout::Matrix4x3,
        (Float, Quad, Quad) => MemoryUnitLayout::Matrix4x4,
        _ => panic!("not a host-sharable matrix type"),
    }
}

fn atomic_layout(scalar_kind: ScalarKind) -> MemoryUnitLayout {
    use ScalarKind::*;

    match scalar_kind {
        Sint => MemoryUnitLayout::Integer,
        Uint => MemoryUnitLayout::UnsignedInteger,
        Float | Bool | AbstractInt | AbstractFloat => panic!("not a host-sharable atomic type"),
    }
}

fn array_layout(
    module: &Module,
    element_ty: Handle<Type>,
    stride: u32,
    len: u32,
) -> MemoryUnitLayout {
    let mut head = Vec::new();
    let mut tail = None;

    collect_layout(0, module, element_ty, &mut head, &mut tail);

    if tail.is_some() {
        panic!("the layout of an array element must be sized");
    }

    MemoryUnitLayout::Array(ArrayLayout {
        element_layout: head,
        stride: stride as u64,
        len: len as u64,
    })
}

fn unsized_tail_layout(
    module: &Module,
    element_ty: Handle<Type>,
    offset: u64,
    stride: u32,
) -> UnsizedTailLayout {
    let mut head = Vec::new();
    let mut tail = None;

    collect_layout(0, module, element_ty, &mut head, &mut tail);

    if tail.is_some() {
        panic!("the layout of an array element must be sized");
    }

    UnsizedTailLayout {
        offset,
        element_layout: head,
        stride: stride as u64,
    }
}

fn collect_layout(
    offset: u64,
    module: &Module,
    type_handle: Handle<Type>,
    head: &mut Vec<MemoryUnit>,
    tail: &mut Option<UnsizedTailLayout>,
) {
    if tail.is_some() {
        panic!("cannot add more units after encountering a dynamically sized array");
    }

    let ty = module.types.get_handle(type_handle).unwrap();

    match &ty.inner {
        TypeInner::Scalar(Scalar { kind, .. }) => head.push(MemoryUnit {
            offset,
            layout: scalar_layout(*kind),
        }),
        TypeInner::Vector {
            scalar: Scalar { kind, .. },
            size,
            ..
        } => head.push(MemoryUnit {
            offset,
            layout: vector_layout(*kind, *size),
        }),
        TypeInner::Matrix {
            scalar: Scalar { kind, .. },
            columns,
            rows,
        } => head.push(MemoryUnit {
            offset,
            layout: matrix_layout(*kind, *columns, *rows),
        }),

        TypeInner::Atomic(Scalar { kind, .. }) => head.push(MemoryUnit {
            offset,
            layout: atomic_layout(*kind),
        }),
        TypeInner::Array { base, size, stride } => {
            match size.to_indexable_length(module).unwrap() {
                IndexableLength::Known(size) => {
                    head.push(MemoryUnit {
                        offset,
                        layout: array_layout(module, *base, *stride, size),
                    });
                }
                IndexableLength::Dynamic => {
                    *tail = Some(unsized_tail_layout(module, *base, offset, *stride));
                }
            }
        }
        TypeInner::Struct { members, .. } => {
            for member in members {
                collect_layout(offset + member.offset as u64, module, member.ty, head, tail);
            }
        }
        _ => panic!("not a host-sharable type"),
    };
}

fn type_to_smi_sized_buffer_layout(module: &Module, ty: Handle<Type>) -> SizedBufferLayout {
    let mut head_units = Vec::new();
    let mut tail_units = None;

    collect_layout(0, module, ty, &mut head_units, &mut tail_units);

    if tail_units.is_some() {
        panic!("a sized buffer should not have an unsized tail")
    }

    SizedBufferLayout {
        memory_units: head_units,
    }
}

fn type_to_smi_unsized_buffer_layout(module: &Module, ty: Handle<Type>) -> UnsizedBufferLayout {
    let mut sized_head = Vec::new();
    let mut unsized_tail = None;

    collect_layout(0, module, ty, &mut sized_head, &mut unsized_tail);

    UnsizedBufferLayout {
        sized_head,
        unsized_tail,
    }
}

fn scalar_kind_to_smi_texel_type(kind: ScalarKind) -> TexelType {
    // TODO: unfiltered float
    match kind {
        ScalarKind::Sint => TexelType::Integer,
        ScalarKind::Uint => TexelType::UnsignedInteger,
        ScalarKind::Float => TexelType::Float,
        ScalarKind::Bool | ScalarKind::AbstractInt | ScalarKind::AbstractFloat => {
            panic!("cannot be a texel kind")
        }
    }
}

fn storage_format_to_smi(format: StorageFormat) -> StorageTextureFormat {
    match format {
        StorageFormat::R32Uint => StorageTextureFormat::r32uint,
        StorageFormat::R32Sint => StorageTextureFormat::r32sint,
        StorageFormat::R32Float => StorageTextureFormat::r32float,
        StorageFormat::Rgba8Unorm => StorageTextureFormat::rgba8unorm,
        StorageFormat::Rgba8Snorm => StorageTextureFormat::rgba8snorm,
        StorageFormat::Rgba8Uint => StorageTextureFormat::rgba8uint,
        StorageFormat::Rgba8Sint => StorageTextureFormat::rgba8sint,
        StorageFormat::Rg32Uint => StorageTextureFormat::rg32uint,
        StorageFormat::Rg32Sint => StorageTextureFormat::rg32sint,
        StorageFormat::Rg32Float => StorageTextureFormat::rg32float,
        StorageFormat::Rgba16Uint => StorageTextureFormat::rgba16uint,
        StorageFormat::Rgba16Sint => StorageTextureFormat::rgba16sint,
        StorageFormat::Rgba16Float => StorageTextureFormat::rgba16float,
        StorageFormat::Rgba32Uint => StorageTextureFormat::rgba32uint,
        StorageFormat::Rgba32Sint => StorageTextureFormat::rgba32sint,
        StorageFormat::Rgba32Float => StorageTextureFormat::rgba32float,
        _ => panic!("format not supported in storage textures"),
    }
}

fn global_to_resource_type(
    module: &Module,
    space: &AddressSpace,
    ty: Handle<Type>,
) -> ResourceType {
    match space {
        AddressSpace::Uniform => {
            let layout = type_to_smi_sized_buffer_layout(module, ty);

            ResourceType::Uniform(layout)
        }
        AddressSpace::Storage { access } => {
            if *access == naga::StorageAccess::all() {
                let layout = type_to_smi_unsized_buffer_layout(module, ty);

                ResourceType::StorageReadWrite(layout)
            } else if *access == naga::StorageAccess::LOAD {
                let layout = type_to_smi_unsized_buffer_layout(module, ty);

                ResourceType::StorageRead(layout)
            } else {
                panic!("storage buffer must be read-only or read-write");
            }
        }
        AddressSpace::Handle => {
            let ty = module.types.get_handle(ty).unwrap();

            match &ty.inner {
                TypeInner::Image {
                    dim,
                    arrayed,
                    class,
                } => match (dim, arrayed, class) {
                    (ImageDimension::D1, false, ImageClass::Sampled { kind, multi: false }) => {
                        let texel_type = scalar_kind_to_smi_texel_type(*kind);

                        ResourceType::Texture1D(texel_type)
                    }
                    (ImageDimension::D2, false, ImageClass::Sampled { kind, multi: false }) => {
                        let texel_type = scalar_kind_to_smi_texel_type(*kind);

                        ResourceType::Texture2D(texel_type)
                    }
                    (ImageDimension::D3, false, ImageClass::Sampled { kind, multi: false }) => {
                        let texel_type = scalar_kind_to_smi_texel_type(*kind);

                        ResourceType::Texture3D(texel_type)
                    }
                    (ImageDimension::D2, true, ImageClass::Sampled { kind, multi: false }) => {
                        let texel_type = scalar_kind_to_smi_texel_type(*kind);

                        ResourceType::Texture2DArray(texel_type)
                    }
                    (ImageDimension::Cube, false, ImageClass::Sampled { kind, multi: false }) => {
                        let texel_type = scalar_kind_to_smi_texel_type(*kind);

                        ResourceType::TextureCube(texel_type)
                    }
                    (ImageDimension::Cube, true, ImageClass::Sampled { kind, multi: false }) => {
                        let texel_type = scalar_kind_to_smi_texel_type(*kind);

                        ResourceType::TextureCubeArray(texel_type)
                    }
                    (ImageDimension::D2, false, ImageClass::Sampled { kind, multi: true }) => {
                        let texel_type = scalar_kind_to_smi_texel_type(*kind);

                        ResourceType::TextureMultisampled2D(texel_type)
                    }
                    (ImageDimension::D2, false, ImageClass::Depth { .. }) => {
                        ResourceType::TextureDepth2D
                    }
                    (ImageDimension::D2, true, ImageClass::Depth { .. }) => {
                        ResourceType::TextureDepth2DArray
                    }
                    (ImageDimension::Cube, false, ImageClass::Depth { .. }) => {
                        ResourceType::TextureDepthCube
                    }
                    (ImageDimension::Cube, true, ImageClass::Depth { .. }) => {
                        ResourceType::TextureDepthCubeArray
                    }
                    (ImageDimension::D1, false, ImageClass::Storage { format, .. }) => {
                        let format = storage_format_to_smi(*format);

                        ResourceType::StorageTexture1D(format)
                    }
                    (ImageDimension::D2, false, ImageClass::Storage { format, .. }) => {
                        let format = storage_format_to_smi(*format);

                        ResourceType::StorageTexture2D(format)
                    }
                    (ImageDimension::D2, true, ImageClass::Storage { format, .. }) => {
                        let format = storage_format_to_smi(*format);

                        ResourceType::StorageTexture2DArray(format)
                    }
                    (ImageDimension::D3, false, ImageClass::Storage { format, .. }) => {
                        let format = storage_format_to_smi(*format);

                        ResourceType::StorageTexture3D(format)
                    }
                    _ => panic!("not a valid image type"),
                },
                // TODO: non-filtering sampler
                TypeInner::Sampler { comparison: true } => ResourceType::ComparisonSampler,
                TypeInner::Sampler { comparison: false } => ResourceType::FilteringSampler,
                _ => panic!(
                    "in the handle address-space, only image and sampler types can be resources"
                ),
            }
        }
        _ => panic!(
            "only globals in the uniform, storage and handle address-spaces can be resources"
        ),
    }
}

/// Used as a key to look up a specific resource binding.
///
/// The WGSL spec mandates that the combination of `group` and `binding` for a resource variable is
/// globally unique within a module, so assuming a valid WGSL module, this will always uniquely
/// identify a single resource.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct BindingKey {
    group: u32,
    binding: u32,
}

fn collect_resource_bindings(module: &Module) -> IndexMap<BindingKey, ResourceBinding> {
    let mut map: IndexMap<BindingKey, ResourceBinding> = Default::default();

    for (_, global) in module.global_variables.iter() {
        if let Some(b) = &global.binding {
            map.insert(
                BindingKey {
                    group: b.group,
                    binding: b.binding,
                },
                ResourceBinding {
                    group: b.group,
                    binding: b.binding,
                    resource_type: global_to_resource_type(&module, &global.space, global.ty),
                },
            );
        }
    }

    map.sort_keys();

    map
}

fn shader_stage_to_smi(stage: &ShaderStage) -> SmiShaderStage {
    match stage {
        ShaderStage::Vertex => SmiShaderStage::Vertex,
        ShaderStage::Fragment => SmiShaderStage::Fragment,
        ShaderStage::Compute => SmiShaderStage::Compute,
    }
}

fn io_binding_ty_to_smi(ty: &Type) -> IoBindingType {
    use ScalarKind::*;
    use VectorSize::*;

    match &ty.inner {
        TypeInner::Scalar(Scalar { kind: Float, .. }) => IoBindingType::Float,
        TypeInner::Scalar(Scalar { kind: Sint, .. }) => IoBindingType::SignedInteger,
        TypeInner::Scalar(Scalar { kind: Uint, .. }) => IoBindingType::UnsignedInteger,
        TypeInner::Vector {
            scalar: Scalar { kind: Float, .. },
            size: Bi,
            ..
        } => IoBindingType::FloatVector2,
        TypeInner::Vector {
            scalar: Scalar { kind: Float, .. },
            size: Tri,
            ..
        } => IoBindingType::FloatVector3,
        TypeInner::Vector {
            scalar: Scalar { kind: Float, .. },
            size: Quad,
            ..
        } => IoBindingType::FloatVector4,
        TypeInner::Vector {
            scalar: Scalar { kind: Sint, .. },
            size: Bi,
            ..
        } => IoBindingType::SignedIntegerVector2,
        TypeInner::Vector {
            scalar: Scalar { kind: Sint, .. },
            size: Tri,
            ..
        } => IoBindingType::SignedIntegerVector3,
        TypeInner::Vector {
            scalar: Scalar { kind: Sint, .. },
            size: Quad,
            ..
        } => IoBindingType::SignedIntegerVector4,
        TypeInner::Vector {
            scalar: Scalar { kind: Uint, .. },
            size: Bi,
            ..
        } => IoBindingType::UnsignedIntegerVector2,
        TypeInner::Vector {
            scalar: Scalar { kind: Uint, .. },
            size: Tri,
            ..
        } => IoBindingType::UnsignedIntegerVector3,
        TypeInner::Vector {
            scalar: Scalar { kind: Uint, .. },
            size: Quad,
            ..
        } => IoBindingType::UnsignedIntegerVector4,
        _ => panic!("not a valid type for an IO-binding"),
    }
}

fn interpolation_to_smi(interpolation: Interpolation) -> InterpolationType {
    match interpolation {
        Interpolation::Perspective => InterpolationType::Perspective,
        Interpolation::Linear => InterpolationType::Linear,
        Interpolation::Flat => InterpolationType::Flat,
    }
}

fn sampling_to_smi(sampling: Sampling) -> SmiSampling {
    match sampling {
        Sampling::Center => SmiSampling::Center,
        Sampling::Centroid => SmiSampling::Centroid,
        Sampling::Sample => SmiSampling::Sample,
    }
}

fn interpolate_to_smi(
    interpolation: Option<Interpolation>,
    sampling: Option<Sampling>,
) -> Option<Interpolate> {
    interpolation.map(|interpolation| {
        let interpolation_type = interpolation_to_smi(interpolation);
        let sampling = sampling.map(sampling_to_smi);

        Interpolate {
            interpolation_type,
            sampling,
        }
    })
}

fn io_binding_to_smi(
    ty: &Type,
    location: u32,
    interpolation: Option<Interpolation>,
    sampling: Option<Sampling>,
) -> IoBinding {
    IoBinding {
        location,
        binding_type: io_binding_ty_to_smi(ty),
        interpolate: interpolate_to_smi(interpolation, sampling),
    }
}

fn collect_bindings(
    module: &Module,
    binding: Option<&Binding>,
    type_handle: Handle<Type>,
    sink: &mut Vec<IoBinding>,
) {
    let ty = module.types.get_handle(type_handle).unwrap();

    if let Some(Binding::Location {
        location,
        interpolation,
        sampling,
        ..
    }) = binding
    {
        sink.push(io_binding_to_smi(ty, *location, *interpolation, *sampling));
    }

    if let TypeInner::Struct { members, .. } = &ty.inner {
        for member in members {
            let binding = member.binding.as_ref();
            let ty = module.types.get_handle(member.ty).unwrap();

            if let Some(Binding::Location {
                location,
                interpolation,
                sampling,
                ..
            }) = binding
            {
                sink.push(io_binding_to_smi(ty, *location, *interpolation, *sampling));
            }
        }
    }
}

fn entry_point_to_smi(
    module: &Module,
    info: &ModuleInfo,
    (index, entry_point): (usize, &EntryPoint),
    overridable_constant_map: &OverridableConstantMap,
    resource_binding_map: &ResourceBindingMap,
) -> SmiEntryPoint {
    let mut input_bindings = Vec::new();

    for argument in entry_point.function.arguments.iter() {
        collect_bindings(
            module,
            argument.binding.as_ref(),
            argument.ty,
            &mut input_bindings,
        );
    }

    let mut output_bindings = Vec::new();

    if let Some(result) = &entry_point.function.result {
        collect_bindings(
            module,
            result.binding.as_ref(),
            result.ty,
            &mut output_bindings,
        );
    }

    input_bindings.sort();
    output_bindings.sort();

    let info = info.get_entry_point(index);

    let mut resource_bindings = Vec::new();

    for (handle, global) in module.global_variables.iter() {
        let usage = info[handle];

        if !usage.is_empty()
            && let Some(binding) = &global.binding
        {
            let resource_index = resource_binding_map
                .get_index_of(&BindingKey {
                    group: binding.group,
                    binding: binding.binding,
                })
                .expect(
                    "an entry point that references a binding does not exist should not have \
                    passed validation",
                );

            resource_bindings.push(resource_index);
        }
    }

    resource_bindings.sort();

    SmiEntryPoint {
        name: entry_point.name.clone(),
        stage: shader_stage_to_smi(&entry_point.stage),
        input_bindings,
        output_bindings,
        overridable_constants: todo!(),
        resource_bindings,
    }
}
