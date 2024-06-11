use std::collections::HashMap;
use std::sync::Arc;
use std::{fmt, slice};

pub use empa_macros::shader_source;
use empa_reflect::{
    ConstantIdentifier, ConstantType, EntryPointBinding as DynamicEntryPointBinding,
    EntryPointBindingType, ParseError as DynamicParseError, ShaderSource as DynamicShaderSource,
    ShaderStage,
};

use crate::device::Device;
use crate::driver::{Device as _, Driver, Dvr};
use crate::pipeline_constants::{PipelineConstantIdentifier, PipelineConstants};
use crate::resource_binding::BindingType;

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StaticConstantDescriptor {
    pub identifier: PipelineConstantIdentifier<'static>,
    pub constant_type: StaticConstantType,
    pub required: bool,
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
pub type StaticConstantType = ConstantType;

/// Internal type for `shader_source` macro.
#[doc(hidden)]
pub type StaticShaderStage = ShaderStage;

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StaticResourceBinding {
    pub group: u32,
    pub binding: u32,
    pub binding_type: BindingType,
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StaticEntryPointBinding {
    pub location: u32,
    pub binding_type: StaticEntryPointBindingType,
    pub interpolation: Option<StaticInterpolation>,
    pub sampling: Option<StaticSampling>,
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StaticEntryPointBindingType {
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

impl StaticEntryPointBindingType {
    pub fn to_entry_point_binding_type(&self) -> EntryPointBindingType {
        match self {
            StaticEntryPointBindingType::SignedInteger => EntryPointBindingType::SignedInteger,
            StaticEntryPointBindingType::SignedIntegerVector2 => {
                EntryPointBindingType::SignedIntegerVector2
            }
            StaticEntryPointBindingType::SignedIntegerVector3 => {
                EntryPointBindingType::SignedIntegerVector3
            }
            StaticEntryPointBindingType::SignedIntegerVector4 => {
                EntryPointBindingType::SignedIntegerVector4
            }
            StaticEntryPointBindingType::UnsignedInteger => EntryPointBindingType::UnsignedInteger,
            StaticEntryPointBindingType::UnsignedIntegerVector2 => {
                EntryPointBindingType::UnsignedIntegerVector2
            }
            StaticEntryPointBindingType::UnsignedIntegerVector3 => {
                EntryPointBindingType::UnsignedIntegerVector3
            }
            StaticEntryPointBindingType::UnsignedIntegerVector4 => {
                EntryPointBindingType::UnsignedIntegerVector4
            }
            StaticEntryPointBindingType::Float => EntryPointBindingType::Float,
            StaticEntryPointBindingType::FloatVector2 => EntryPointBindingType::FloatVector2,
            StaticEntryPointBindingType::FloatVector3 => EntryPointBindingType::FloatVector3,
            StaticEntryPointBindingType::FloatVector4 => EntryPointBindingType::FloatVector4,
            StaticEntryPointBindingType::HalfFloat => EntryPointBindingType::HalfFloat,
            StaticEntryPointBindingType::HalfFloatVector2 => {
                EntryPointBindingType::HalfFloatVector2
            }
            StaticEntryPointBindingType::HalfFloatVector3 => {
                EntryPointBindingType::HalfFloatVector3
            }
            StaticEntryPointBindingType::HalfFloatVector4 => {
                EntryPointBindingType::HalfFloatVector4
            }
        }
    }
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StaticInterpolation {
    Perspective,
    Linear,
    Flat,
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StaticSampling {
    Center,
    Centroid,
    Sample,
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StaticShaderSource {
    pub source: &'static str,
    pub resource_bindings: &'static [StaticResourceBinding],
    pub constants: &'static [StaticConstantDescriptor],
    pub entry_points: &'static [StaticEntryPoint],
}

/// Internal type for `shader_source` macro.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StaticEntryPoint {
    pub name: &'static str,
    pub stage: StaticShaderStage,
    pub input_bindings: &'static [StaticEntryPointBinding],
    pub output_bindings: &'static [StaticEntryPointBinding],
}

#[derive(Clone)]
pub(crate) enum ShaderSourceInternal {
    Static(StaticShaderSource),
    Dynamic(Arc<DynamicShaderSource>),
}

impl ShaderSourceInternal {
    pub(crate) fn source(&self) -> &str {
        match self {
            ShaderSourceInternal::Static(source) => source.source,
            ShaderSourceInternal::Dynamic(source) => source.raw_str(),
        }
    }

    pub(crate) fn resource_bindings(&self) -> &[StaticResourceBinding] {
        match self {
            ShaderSourceInternal::Static(source) => source.resource_bindings,
            ShaderSourceInternal::Dynamic(_) => todo!(),
        }
    }

    pub(crate) fn has_required_constants(&self) -> bool {
        match self {
            ShaderSourceInternal::Static(s) => s.constants.iter().any(|c| c.required),
            ShaderSourceInternal::Dynamic(s) => s.constants().iter().any(|c| c.required()),
        }
    }

    pub(crate) fn resolve_entry_point_index(&self, name: &str) -> Option<usize> {
        match self {
            ShaderSourceInternal::Static(source) => source
                .entry_points
                .iter()
                .enumerate()
                .find(|(_, e)| e.name == name)
                .map(|(index, _)| index),
            ShaderSourceInternal::Dynamic(source) => source
                .entry_points()
                .iter()
                .enumerate()
                .find(|(_, e)| e.name() == name)
                .map(|(index, _)| index),
        }
    }

    pub(crate) fn entry_point_stage(&self, index: usize) -> Option<ShaderStage> {
        match self {
            ShaderSourceInternal::Static(source) => source.entry_points.get(index).map(|e| e.stage),
            ShaderSourceInternal::Dynamic(source) => {
                source.entry_points().get(index).map(|e| e.stage())
            }
        }
    }

    pub(crate) fn entry_point_input_bindings(&self, index: usize) -> Option<EntryPointBindings> {
        match self {
            ShaderSourceInternal::Static(source) => source
                .entry_points
                .get(index)
                .map(|e| EntryPointBindings::Static(e.input_bindings.iter())),
            ShaderSourceInternal::Dynamic(source) => source
                .entry_points()
                .get(index)
                .map(|e| EntryPointBindings::Dynamic(e.input_bindings().iter())),
        }
    }

    pub(crate) fn entry_point_output_bindings(&self, index: usize) -> Option<EntryPointBindings> {
        match self {
            ShaderSourceInternal::Static(source) => source
                .entry_points
                .get(index)
                .map(|e| EntryPointBindings::Static(e.output_bindings.iter())),
            ShaderSourceInternal::Dynamic(source) => source
                .entry_points()
                .get(index)
                .map(|e| EntryPointBindings::Dynamic(e.output_bindings().iter())),
        }
    }

    pub(crate) fn build_constants<C: PipelineConstants>(
        &self,
        pipeline_constants: &C,
    ) -> HashMap<String, f64> {
        let mut map = HashMap::new();

        let mut add_constant = |identifier: PipelineConstantIdentifier,
                                tpe: ConstantType,
                                required: bool| {
            if let Some(supplied_value) = pipeline_constants.lookup(identifier) {
                if supplied_value.constant_type() != tpe {
                    panic!("supplied value for pipeline constant `{}` does not match the type expected by the shader", identifier)
                }

                map.insert(identifier.to_string(), supplied_value.to_f64());
            } else {
                if required {
                    panic!(
                        "could not find a value for the required constant `{}`",
                        identifier
                    );
                }
            }
        };

        match self {
            ShaderSourceInternal::Static(s) => {
                for constant in s.constants {
                    add_constant(
                        constant.identifier,
                        constant.constant_type,
                        constant.required,
                    );
                }
            }
            ShaderSourceInternal::Dynamic(s) => {
                for constant in s.constants() {
                    let identifier = match constant.identifier() {
                        ConstantIdentifier::Number(n) => PipelineConstantIdentifier::Number(*n),
                        ConstantIdentifier::Name(n) => PipelineConstantIdentifier::Name(n),
                    };

                    add_constant(identifier, constant.constant_type(), constant.required());
                }
            }
        }

        map
    }
}

pub(crate) enum EntryPointBindings<'a> {
    Static(slice::Iter<'a, StaticEntryPointBinding>),
    Dynamic(slice::Iter<'a, DynamicEntryPointBinding>),
}

impl<'a> Iterator for EntryPointBindings<'a> {
    type Item = EntryPointBinding<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EntryPointBindings::Static(s) => s.next().map(|e| EntryPointBinding::Static(e)),
            EntryPointBindings::Dynamic(s) => s.next().map(|e| EntryPointBinding::Dynamic(e)),
        }
    }
}

pub(crate) enum EntryPointBinding<'a> {
    Static(&'a StaticEntryPointBinding),
    Dynamic(&'a DynamicEntryPointBinding),
}

impl EntryPointBinding<'_> {
    pub(crate) fn location(&self) -> u32 {
        match self {
            EntryPointBinding::Static(b) => b.location,
            EntryPointBinding::Dynamic(b) => b.location(),
        }
    }

    pub(crate) fn binding_type(&self) -> EntryPointBindingType {
        match self {
            EntryPointBinding::Static(b) => b.binding_type.to_entry_point_binding_type(),
            EntryPointBinding::Dynamic(b) => b.binding_type(),
        }
    }
}

pub struct ParseError {
    inner: DynamicParseError,
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <DynamicParseError as fmt::Debug>::fmt(&self.inner, f)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <DynamicParseError as fmt::Display>::fmt(&self.inner, f)
    }
}

pub struct ShaderSource {
    inner: ShaderSourceInternal,
}

impl ShaderSource {
    /// Internal function for `shader_source` macro.
    #[doc(hidden)]
    pub const fn from_static(shader_source: StaticShaderSource) -> Self {
        ShaderSource {
            inner: ShaderSourceInternal::Static(shader_source),
        }
    }

    pub fn parse(raw: String) -> Result<Self, ParseError> {
        DynamicShaderSource::parse(raw)
            .map(|ok| ShaderSource {
                inner: ShaderSourceInternal::Dynamic(Arc::new(ok)),
            })
            .map_err(|inner| ParseError { inner })
    }
}

pub struct ShaderModule {
    pub(crate) handle: <Dvr as Driver>::ShaderModuleHandle,
    pub(crate) meta: ShaderSourceInternal,
}

impl ShaderModule {
    pub(crate) fn new(device: &Device, source: &ShaderSource) -> Self {
        let handle = device
            .device_handle
            .create_shader_module(source.inner.source());

        ShaderModule {
            handle,
            meta: source.inner.clone(),
        }
    }
}
