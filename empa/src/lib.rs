#![feature(const_mut_refs, generic_associated_types, new_uninit)]

pub use empa_reflect::abi;

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

mod compare_function;
pub use self::compare_function::CompareFunction;

#[doc(hidden)]
pub struct Untyped {}
