use std::marker;

use flagset::FlagSet;

use crate::abi;
use crate::access_mode::{Read, ReadWrite};
use crate::driver::ShaderStage;
use crate::resource_binding::bind_group_layout::{
    BindGroupLayoutEntry, BindingType, SizedBufferLayout, TexelType, UnsizedBufferLayout,
};
use crate::texture::format::Storable;
use crate::type_flag::{TypeFlag, O, X};

mod visibility_seal {
    use flagset::FlagSet;

    use crate::driver::ShaderStage;

    pub trait Seal {
        #[doc(hidden)]
        const FLAG_SET: FlagSet<ShaderStage>;
    }
}

pub trait Visibility: visibility_seal::Seal {}

// TODO: currently the type tags are ordered in the same way you would view them in a bit-flag
// number; decide whether or not I like that, because though that does make some sense, part of my
// brain seems to find the reverse order more intuitive.
pub struct ShaderStages<Compute: TypeFlag, Fragment: TypeFlag, Vertex: TypeFlag> {
    _marker: marker::PhantomData<(Compute, Fragment, Vertex)>,
}

impl<Compute: TypeFlag, Fragment: TypeFlag, Vertex: TypeFlag> visibility_seal::Seal
    for ShaderStages<Compute, Fragment, Vertex>
{
    const FLAG_SET: FlagSet<ShaderStage> = {
        let mut bits = 0;

        if Compute::IS_ENABLED {
            bits |= 0x0004;
        }

        if Fragment::IS_ENABLED {
            bits |= 0x0002;
        }

        if Vertex::IS_ENABLED {
            bits |= 0x0001;
        }

        unsafe { FlagSet::new_unchecked(bits) }
    };

    // TODO when const traits
    // const FLAG_SET: FlagSet<ShaderStage> = {
    //     let mut flag_set = FlagSet::from(ShaderStage::None);
    //
    //     if Compute::IS_ENABLED {
    //         flag_set |= ShaderStage::Compute;
    //     }
    //
    //     if Fragment::IS_ENABLED {
    //         flag_set |= ShaderStage::Fragment;
    //     }
    //
    //     if Vertex::IS_ENABLED {
    //         flag_set |= ShaderStage::Vertex;
    //     }
    //
    //     flag_set
    // };
}

impl<Compute: TypeFlag, Fragment: TypeFlag, Vertex: TypeFlag> Visibility
    for ShaderStages<Compute, Fragment, Vertex>
{
}

mod vertex_visible_seal {
    pub trait Seal {}
}

pub trait VertexVisible: vertex_visible_seal::Seal {}

impl<Compute: TypeFlag, Fragment: TypeFlag> vertex_visible_seal::Seal
    for ShaderStages<Compute, Fragment, X>
{
}
impl<Compute: TypeFlag, Fragment: TypeFlag> VertexVisible for ShaderStages<Compute, Fragment, X> {}

mod fragment_visible_seal {
    pub trait Seal {}
}

pub trait FragmentVisible: fragment_visible_seal::Seal {}

impl<Compute: TypeFlag, Vertex: TypeFlag> fragment_visible_seal::Seal
    for ShaderStages<Compute, X, Vertex>
{
}
impl<Compute: TypeFlag, Vertex: TypeFlag> FragmentVisible for ShaderStages<Compute, X, Vertex> {}

mod compute_visible_seal {
    pub trait Seal {}
}

pub trait ComputeVisible: compute_visible_seal::Seal {}

impl<Fragment: TypeFlag, Vertex: TypeFlag> compute_visible_seal::Seal
    for ShaderStages<X, Fragment, Vertex>
{
}
impl<Fragment: TypeFlag, Vertex: TypeFlag> ComputeVisible for ShaderStages<X, Fragment, Vertex> {}

mod typed_slot_binding_seal {
    pub trait Seal {}
}

pub trait TypedSlotBinding: typed_slot_binding_seal::Seal {
    const ENTRY: Option<BindGroupLayoutEntry>;

    /// Helper for the `Resources` derive macro.
    #[doc(hidden)]
    type WithVisibility<T: Visibility>;
}

impl typed_slot_binding_seal::Seal for () {}
impl TypedSlotBinding for () {
    const ENTRY: Option<BindGroupLayoutEntry> = None;

    type WithVisibility<T: Visibility> = ();
}

#[allow(non_camel_case_types)]
pub struct f32_unfiltered {}

pub struct Texture1D<T, Visibility> {
    _marker: marker::PhantomData<(*const T, Visibility)>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture1D<f32, V> {}
impl<V: Visibility> TypedSlotBinding for Texture1D<f32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture1D(TexelType::Float),
    });

    type WithVisibility<T: Visibility> = Texture1D<f32, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture1D<f32_unfiltered, V> {}
impl<V: Visibility> TypedSlotBinding for Texture1D<f32_unfiltered, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture1D(TexelType::UnfilterableFloat),
    });

    type WithVisibility<T: Visibility> = Texture1D<f32_unfiltered, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture1D<i32, V> {}
impl<V: Visibility> TypedSlotBinding for Texture1D<i32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture1D(TexelType::SignedInteger),
    });

    type WithVisibility<T: Visibility> = Texture1D<i32, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture1D<u32, V> {}
impl<V: Visibility> TypedSlotBinding for Texture1D<u32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture1D(TexelType::UnsignedInteger),
    });

    type WithVisibility<T: Visibility> = Texture1D<u32, T>;
}

pub struct Texture2D<T, Visibility> {
    _marker: marker::PhantomData<(*const T, Visibility)>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture2D<f32, V> {}
impl<V: Visibility> TypedSlotBinding for Texture2D<f32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture2D(TexelType::Float),
    });

    type WithVisibility<T: Visibility> = Texture2D<f32, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture2D<f32_unfiltered, V> {}
impl<V: Visibility> TypedSlotBinding for Texture2D<f32_unfiltered, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture2D(TexelType::UnfilterableFloat),
    });

    type WithVisibility<T: Visibility> = Texture2D<f32_unfiltered, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture2D<i32, V> {}
impl<V: Visibility> TypedSlotBinding for Texture2D<i32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture2D(TexelType::SignedInteger),
    });

    type WithVisibility<T: Visibility> = Texture2D<i32, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture2D<u32, V> {}
impl<V: Visibility> TypedSlotBinding for Texture2D<u32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture2D(TexelType::UnsignedInteger),
    });

    type WithVisibility<T: Visibility> = Texture2D<u32, T>;
}

pub struct Texture3D<T, Visibility> {
    _marker: marker::PhantomData<(*const T, Visibility)>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture3D<f32, V> {}
impl<V: Visibility> TypedSlotBinding for Texture3D<f32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture3D(TexelType::Float),
    });

    type WithVisibility<T: Visibility> = Texture3D<f32, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture3D<f32_unfiltered, V> {}
impl<V: Visibility> TypedSlotBinding for Texture3D<f32_unfiltered, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture3D(TexelType::UnfilterableFloat),
    });

    type WithVisibility<T: Visibility> = Texture3D<f32_unfiltered, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture3D<i32, V> {}
impl<V: Visibility> TypedSlotBinding for Texture3D<i32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture3D(TexelType::SignedInteger),
    });

    type WithVisibility<T: Visibility> = Texture3D<i32, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture3D<u32, V> {}
impl<V: Visibility> TypedSlotBinding for Texture3D<u32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture3D(TexelType::UnsignedInteger),
    });

    type WithVisibility<T: Visibility> = Texture3D<u32, T>;
}

pub struct Texture2DArray<T, Visibility> {
    _marker: marker::PhantomData<(*const T, Visibility)>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture2DArray<f32, V> {}
impl<V: Visibility> TypedSlotBinding for Texture2DArray<f32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture2DArray(TexelType::Float),
    });

    type WithVisibility<T: Visibility> = Texture2DArray<f32, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture2DArray<f32_unfiltered, V> {}
impl<V: Visibility> TypedSlotBinding for Texture2DArray<f32_unfiltered, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture2DArray(TexelType::UnfilterableFloat),
    });

    type WithVisibility<T: Visibility> = Texture2DArray<f32_unfiltered, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture2DArray<i32, V> {}
impl<V: Visibility> TypedSlotBinding for Texture2DArray<i32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture2DArray(TexelType::SignedInteger),
    });

    type WithVisibility<T: Visibility> = Texture2DArray<i32, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for Texture2DArray<u32, V> {}
impl<V: Visibility> TypedSlotBinding for Texture2DArray<u32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Texture2DArray(TexelType::UnsignedInteger),
    });

    type WithVisibility<T: Visibility> = Texture2DArray<u32, T>;
}

pub struct TextureCube<T, Visibility> {
    _marker: marker::PhantomData<(*const T, Visibility)>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureCube<f32, V> {}
impl<V: Visibility> TypedSlotBinding for TextureCube<f32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureCube(TexelType::Float),
    });

    type WithVisibility<T: Visibility> = TextureCube<f32, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureCube<f32_unfiltered, V> {}
impl<V: Visibility> TypedSlotBinding for TextureCube<f32_unfiltered, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureCube(TexelType::UnfilterableFloat),
    });

    type WithVisibility<T: Visibility> = TextureCube<f32_unfiltered, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureCube<i32, V> {}
impl<V: Visibility> TypedSlotBinding for TextureCube<i32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureCube(TexelType::SignedInteger),
    });

    type WithVisibility<T: Visibility> = TextureCube<i32, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureCube<u32, V> {}
impl<V: Visibility> TypedSlotBinding for TextureCube<u32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureCube(TexelType::UnsignedInteger),
    });

    type WithVisibility<T: Visibility> = TextureCube<u32, T>;
}

pub struct TextureCubeArray<T, Visibility> {
    _marker: marker::PhantomData<(*const T, Visibility)>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureCubeArray<f32, V> {}
impl<V: Visibility> TypedSlotBinding for TextureCubeArray<f32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureCubeArray(TexelType::Float),
    });

    type WithVisibility<T: Visibility> = TextureCubeArray<f32, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureCubeArray<f32_unfiltered, V> {}
impl<V: Visibility> TypedSlotBinding for TextureCubeArray<f32_unfiltered, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureCubeArray(TexelType::UnfilterableFloat),
    });

    type WithVisibility<T: Visibility> = TextureCubeArray<f32_unfiltered, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureCubeArray<i32, V> {}
impl<V: Visibility> TypedSlotBinding for TextureCubeArray<i32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureCubeArray(TexelType::SignedInteger),
    });

    type WithVisibility<T: Visibility> = TextureCubeArray<i32, T>;
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureCubeArray<u32, V> {}
impl<V: Visibility> TypedSlotBinding for TextureCubeArray<u32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureCubeArray(TexelType::UnsignedInteger),
    });

    type WithVisibility<T: Visibility> = TextureCubeArray<u32, T>;
}

pub struct TextureMultisampled2D<T, Visibility> {
    _marker: marker::PhantomData<(*const T, Visibility)>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureMultisampled2D<f32, V> {}
impl<V: Visibility> TypedSlotBinding for TextureMultisampled2D<f32, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureMultisampled2D(TexelType::Float),
    });

    type WithVisibility<T: Visibility> = TextureMultisampled2D<f32, T>;
}

pub struct TextureDepth2D<Visibility> {
    _marker: marker::PhantomData<Visibility>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureDepth2D<V> {}
impl<V: Visibility> TypedSlotBinding for TextureDepth2D<V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureDepth2D,
    });

    type WithVisibility<T: Visibility> = TextureDepth2D<T>;
}

pub struct TextureDepth2DArray<Visibility> {
    _marker: marker::PhantomData<Visibility>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureDepth2DArray<V> {}
impl<V: Visibility> TypedSlotBinding for TextureDepth2DArray<V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureDepth2DArray,
    });

    type WithVisibility<T: Visibility> = TextureDepth2DArray<T>;
}

pub struct TextureDepthCube<Visibility> {
    _marker: marker::PhantomData<Visibility>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureDepthCube<V> {}
impl<V: Visibility> TypedSlotBinding for TextureDepthCube<V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureDepthCube,
    });

    type WithVisibility<T: Visibility> = TextureDepthCube<T>;
}

pub struct TextureDepthCubeArray<Visibility> {
    _marker: marker::PhantomData<Visibility>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureDepthCubeArray<V> {}
impl<V: Visibility> TypedSlotBinding for TextureDepthCubeArray<V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureDepthCubeArray,
    });

    type WithVisibility<T: Visibility> = TextureDepthCubeArray<T>;
}

pub struct TextureDepthMultisampled2D<Visibility> {
    _marker: marker::PhantomData<Visibility>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for TextureDepthMultisampled2D<V> {}
impl<V: Visibility> TypedSlotBinding for TextureDepthMultisampled2D<V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::TextureDepthMultisampled2D,
    });

    type WithVisibility<T: Visibility> = TextureDepthMultisampled2D<T>;
}

pub struct FilteringSampler<Visibility> {
    _marker: marker::PhantomData<Visibility>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for FilteringSampler<V> {}
impl<V: Visibility> TypedSlotBinding for FilteringSampler<V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::FilteringSampler,
    });

    type WithVisibility<T: Visibility> = FilteringSampler<T>;
}

pub struct NonFilteringSampler<Visibility> {
    _marker: marker::PhantomData<Visibility>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for NonFilteringSampler<V> {}
impl<V: Visibility> TypedSlotBinding for NonFilteringSampler<V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::NonFilteringSampler,
    });

    type WithVisibility<T: Visibility> = NonFilteringSampler<T>;
}

pub struct ComparisonSampler<Visibility> {
    _marker: marker::PhantomData<Visibility>,
}

impl<V: Visibility> typed_slot_binding_seal::Seal for ComparisonSampler<V> {}
impl<V: Visibility> TypedSlotBinding for ComparisonSampler<V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::ComparisonSampler,
    });

    type WithVisibility<T: Visibility> = ComparisonSampler<T>;
}

pub struct Uniform<T, Visibility> {
    _marker: marker::PhantomData<(*const T, Visibility)>,
}

impl<T: abi::Sized, V: Visibility> typed_slot_binding_seal::Seal for Uniform<T, V> {}
impl<T: abi::Sized, V: Visibility> TypedSlotBinding for Uniform<T, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Uniform(SizedBufferLayout(T::LAYOUT)),
    });

    type WithVisibility<N: Visibility> = Uniform<T, N>;
}

pub trait ValidStorageVisibility: Visibility {}

impl<Compute: TypeFlag, Fragment: TypeFlag> ValidStorageVisibility
    for ShaderStages<Compute, Fragment, O>
{
}

pub struct Storage<T, A, Visibility>
where
    T: ?Sized,
{
    _marker: marker::PhantomData<(*const T, A, Visibility)>,
}

impl<T: abi::Unsized + ?Sized, V: ValidStorageVisibility> typed_slot_binding_seal::Seal
    for Storage<T, ReadWrite, V>
{
}
impl<T: abi::Unsized + ?Sized, V: ValidStorageVisibility> TypedSlotBinding
    for Storage<T, ReadWrite, V>
{
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::Storage(UnsizedBufferLayout {
            sized_head: T::SIZED_HEAD_LAYOUT,
            unsized_tail: T::UNSIZED_TAIL_LAYOUT,
        }),
    });

    type WithVisibility<N: Visibility> = Storage<T, ReadWrite, N>;
}

impl<T: abi::Unsized + ?Sized, V: Visibility> typed_slot_binding_seal::Seal
    for Storage<T, Read, V>
{
}
impl<T: abi::Unsized + ?Sized, V: Visibility> TypedSlotBinding for Storage<T, Read, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::ReadOnlyStorage(UnsizedBufferLayout {
            sized_head: T::SIZED_HEAD_LAYOUT,
            unsized_tail: T::UNSIZED_TAIL_LAYOUT,
        }),
    });

    type WithVisibility<N: Visibility> = Storage<T, Read, N>;
}

pub struct StorageTexture1D<F, Visibility> {
    _marker: marker::PhantomData<(*const F, Visibility)>,
}

impl<F: Storable, V: Visibility> typed_slot_binding_seal::Seal for StorageTexture1D<F, V> {}
impl<F: Storable, V: Visibility> TypedSlotBinding for StorageTexture1D<F, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::StorageTexture1D(F::FORMAT_ID),
    });

    type WithVisibility<T: Visibility> = StorageTexture1D<F, T>;
}

pub struct StorageTexture2D<F, Visibility> {
    _marker: marker::PhantomData<(*const F, Visibility)>,
}

impl<F: Storable, V: Visibility> typed_slot_binding_seal::Seal for StorageTexture2D<F, V> {}
impl<F: Storable, V: Visibility> TypedSlotBinding for StorageTexture2D<F, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::StorageTexture2D(F::FORMAT_ID),
    });

    type WithVisibility<T: Visibility> = StorageTexture2D<F, T>;
}

pub struct StorageTexture2DArray<F: Storable, Visibility> {
    _marker: marker::PhantomData<(*const F, Visibility)>,
}

impl<F: Storable, V: Visibility> typed_slot_binding_seal::Seal for StorageTexture2DArray<F, V> {}
impl<F: Storable, V: Visibility> TypedSlotBinding for StorageTexture2DArray<F, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::StorageTexture2DArray(F::FORMAT_ID),
    });

    type WithVisibility<T: Visibility> = StorageTexture2DArray<F, T>;
}

pub struct StorageTexture3D<F, Visibility> {
    _marker: marker::PhantomData<(*const F, Visibility)>,
}

impl<F: Storable, V: Visibility> typed_slot_binding_seal::Seal for StorageTexture3D<F, V> {}
impl<F: Storable, V: Visibility> TypedSlotBinding for StorageTexture3D<F, V> {
    const ENTRY: Option<BindGroupLayoutEntry> = Some(BindGroupLayoutEntry {
        visibility: V::FLAG_SET,
        binding_type: BindingType::StorageTexture3D(F::FORMAT_ID),
    });

    type WithVisibility<T: Visibility> = StorageTexture3D<F, T>;
}
