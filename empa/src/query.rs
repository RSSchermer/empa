use web_sys::{GpuQuerySet, GpuQuerySetDescriptor, GpuQueryType};

use crate::device::Device;

pub struct OcclusionQuerySet {
    inner: GpuQuerySet,
    len: u32,
}

impl OcclusionQuerySet {
    pub(crate) fn new(device: &Device, len: u32) -> Self {
        assert!(len < 8192, "query set len must be less than `8192`");

        let desc = GpuQuerySetDescriptor::new(len, GpuQueryType::Occlusion);
        let inner = device.inner.create_query_set(&desc);

        OcclusionQuerySet { inner, len }
    }

    pub(crate) fn as_web_sys(&self) -> &GpuQuerySet {
        &self.inner
    }

    pub fn len(&self) -> u32 {
        self.len
    }
}
