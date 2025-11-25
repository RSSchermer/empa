use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

pub use empa_macros::shader_source;
use empa_smi::wgsl::BuildSmiError;
use empa_smi::{EntryPoint, ResourceBinding, ShaderModuleInterface};

use crate::device::Device;
use crate::driver::{Device as _, Driver, Dvr};
use crate::pipeline_constants::{PipelineConstantIdentifier, PipelineConstants};

pub struct ParseError {
    inner: BuildSmiError,
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            BuildSmiError::Parse(e) => fmt::Debug::fmt(e, f),
            BuildSmiError::Validation(e) => fmt::Debug::fmt(e, f),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            BuildSmiError::Parse(e) => fmt::Display::fmt(e, f),
            BuildSmiError::Validation(e) => fmt::Display::fmt(e, f),
        }
    }
}

pub(crate) enum SmiRef {
    Static(&'static ShaderModuleInterface),
    Dynamic(Arc<ShaderModuleInterface>),
}

impl From<&'static ShaderModuleInterface> for SmiRef {
    fn from(value: &'static ShaderModuleInterface) -> Self {
        SmiRef::Static(value)
    }
}

impl From<ShaderModuleInterface> for SmiRef {
    fn from(value: ShaderModuleInterface) -> Self {
        SmiRef::Dynamic(Arc::new(value))
    }
}

impl Deref for SmiRef {
    type Target = ShaderModuleInterface;

    fn deref(&self) -> &Self::Target {
        match self {
            SmiRef::Static(v) => v,
            SmiRef::Dynamic(v) => v,
        }
    }
}

pub struct ShaderSource {
    source: Cow<'static, str>,
    smi: SmiRef,
}

impl ShaderSource {
    /// Internal function for `shader_source` macro.
    #[doc(hidden)]
    pub const fn from_static_unchecked(
        source: &'static str,
        smi: &'static ShaderModuleInterface,
    ) -> Self {
        ShaderSource {
            source: Cow::Borrowed(source),
            smi: SmiRef::Static(smi),
        }
    }

    pub fn parse(source: String) -> Result<Self, ParseError> {
        let smi = empa_smi::wgsl::build_smi(&source).map_err(|e| ParseError { inner: e })?;

        Ok(ShaderSource {
            source: source.into(),
            smi: smi.into(),
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct EntryPointExt<'a> {
    smi: &'a ShaderModuleInterface,
    entry_point: &'a EntryPoint,
}

impl EntryPointExt<'_> {
    pub(crate) fn has_required_constants(&self) -> bool {
        self.entry_point
            .overridable_constants
            .iter()
            .any(|constant_index| self.smi.overridable_constants[*constant_index].required)
    }

    pub(crate) fn resource_bindings(&self) -> impl Iterator<Item = &ResourceBinding> + '_ {
        self.entry_point
            .resource_bindings
            .iter()
            .map(|i| &self.smi.resource_bindings[*i])
    }

    pub(crate) fn build_constants<C: PipelineConstants>(
        &self,
        pipeline_constants: &C,
    ) -> HashMap<String, f64> {
        let mut map = HashMap::new();

        for constant_index in self.entry_point.overridable_constants.iter().copied() {
            let constant = &self.smi.overridable_constants[constant_index];

            let identifier = if let Some(id) = constant.id {
                PipelineConstantIdentifier::Number(id)
            } else {
                PipelineConstantIdentifier::Name(constant.name.as_ref())
            };

            if let Some(supplied_value) = pipeline_constants.lookup(identifier) {
                if supplied_value.constant_type() != constant.constant_type {
                    panic!(
                        "supplied value for pipeline constant `{}` does not match the type \
                        expected by the shader",
                        identifier
                    )
                }

                // Pipelines with multiple entry points (e.g. render pipelines) may reference
                // the same overridable constant multiple times, but since the identifier will
                // be identical in each case, this does not result in duplicate entries.
                map.insert(identifier.to_string(), supplied_value.to_f64());
            } else {
                if constant.required {
                    panic!(
                        "could not find a value for the required constant `{}`",
                        identifier
                    );
                }
            }
        }

        map
    }
}

impl Deref for EntryPointExt<'_> {
    type Target = EntryPoint;

    fn deref(&self) -> &Self::Target {
        self.entry_point
    }
}

#[derive(Clone)]
pub(crate) struct ShaderModuleData {
    pub(crate) handle: <Dvr as Driver>::ShaderModuleHandle,
    pub(crate) smi: ShaderModuleInterface,
}

impl ShaderModuleData {
    pub(crate) fn resolve_entry_point_index(&self, name: &str) -> Option<usize> {
        self.smi
            .entry_points
            .iter()
            .enumerate()
            .find(|(_, e)| e.name.as_ref() == name)
            .map(|(index, _)| index)
    }

    pub(crate) fn entry_point_ext(&self, entry_point_index: usize) -> EntryPointExt<'_> {
        let entry_point = &self.smi.entry_points[entry_point_index];

        EntryPointExt {
            smi: &self.smi,
            entry_point,
        }
    }
}

pub struct ShaderModule {
    pub(crate) data: ShaderModuleData,
}

impl ShaderModule {
    pub(crate) fn new(device: &Device, source: &ShaderSource) -> Self {
        let handle = device
            .device_handle
            .create_shader_module(source.source.as_ref());
        let data = ShaderModuleData {
            handle,
            smi: source.smi.clone(),
        };

        ShaderModule { data }
    }
}
