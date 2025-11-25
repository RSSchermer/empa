mod driver;

mod compare_function;
pub use compare_function::CompareFunction;

pub mod abi;
pub mod access_mode;
pub mod adapter;
pub mod buffer;
pub mod command;
pub mod compute_pipeline;
pub mod device;
pub mod pipeline_constants;
pub mod query;
pub mod render_pipeline;
pub mod render_target;
pub mod resource_binding;
pub mod sampler;
pub mod shader_module;
pub mod texture;
pub mod type_flag;

pub mod smi {
    use std::borrow::Cow;

    pub use empa_smi::{
        ArrayLayout, EntryPoint, Interpolate, InterpolationType, IoBinding, IoBindingType,
        MemoryUnit, MemoryUnitLayout, OverridableConstant, OverridableConstantType,
        ResourceBinding, ResourceType, Sampling, ShaderModuleInterface, ShaderStage,
        SizedBufferLayout, StorageTextureFormat, TexelType, UnsizedBufferLayout, UnsizedTailLayout,
    };

    /// This is a helper function for making a clone in a constant context; used by empa-macros
    #[doc(hidden)]
    pub const fn clone_memory_unit_layout(
        memory_unit_layout: &MemoryUnitLayout,
    ) -> MemoryUnitLayout {
        match memory_unit_layout {
            MemoryUnitLayout::Float => MemoryUnitLayout::Float,
            MemoryUnitLayout::FloatVector2 => MemoryUnitLayout::FloatVector2,
            MemoryUnitLayout::FloatVector3 => MemoryUnitLayout::FloatVector3,
            MemoryUnitLayout::FloatVector4 => MemoryUnitLayout::FloatVector4,
            MemoryUnitLayout::Integer => MemoryUnitLayout::Integer,
            MemoryUnitLayout::IntegerVector2 => MemoryUnitLayout::IntegerVector2,
            MemoryUnitLayout::IntegerVector3 => MemoryUnitLayout::IntegerVector3,
            MemoryUnitLayout::IntegerVector4 => MemoryUnitLayout::IntegerVector4,
            MemoryUnitLayout::UnsignedInteger => MemoryUnitLayout::UnsignedInteger,
            MemoryUnitLayout::UnsignedIntegerVector2 => MemoryUnitLayout::UnsignedIntegerVector2,
            MemoryUnitLayout::UnsignedIntegerVector3 => MemoryUnitLayout::UnsignedIntegerVector3,
            MemoryUnitLayout::UnsignedIntegerVector4 => MemoryUnitLayout::UnsignedIntegerVector4,
            MemoryUnitLayout::Matrix2x2 => MemoryUnitLayout::Matrix2x2,
            MemoryUnitLayout::Matrix2x3 => MemoryUnitLayout::Matrix2x3,
            MemoryUnitLayout::Matrix2x4 => MemoryUnitLayout::Matrix2x4,
            MemoryUnitLayout::Matrix3x2 => MemoryUnitLayout::Matrix3x2,
            MemoryUnitLayout::Matrix3x3 => MemoryUnitLayout::Matrix3x3,
            MemoryUnitLayout::Matrix3x4 => MemoryUnitLayout::Matrix3x4,
            MemoryUnitLayout::Matrix4x2 => MemoryUnitLayout::Matrix4x2,
            MemoryUnitLayout::Matrix4x3 => MemoryUnitLayout::Matrix4x3,
            MemoryUnitLayout::Matrix4x4 => MemoryUnitLayout::Matrix4x4,
            MemoryUnitLayout::Array(array_layout) => {
                let element_layout = match array_layout.element_layout {
                    Cow::Borrowed(b) => b,
                    Cow::Owned(_) => unreachable!(),
                };

                MemoryUnitLayout::Array(ArrayLayout {
                    element_layout: Cow::Borrowed(element_layout),
                    stride: array_layout.stride,
                    len: array_layout.len,
                })
            }
        }
    }
}

#[cfg(all(feature = "web", feature = "arwa"))]
pub mod arwa;

#[cfg(not(feature = "web"))]
pub mod native;

#[doc(hidden)]
pub struct Untyped {}

#[doc(hidden)]
pub use memoffset::offset_of;
