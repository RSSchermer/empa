#![feature(new_uninit)]

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

#[cfg(feature = "arwa")]
pub mod arwa;

#[doc(hidden)]
pub struct Untyped {}

#[doc(hidden)]
pub use memoffset::offset_of;
