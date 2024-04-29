use std::sync::Arc;

use web_sys::{GpuQuerySet, GpuQuerySetDescriptor, GpuQueryType};

use crate::device::Device;

pub(crate) struct QuerySetHandle {
    query_set: GpuQuerySet,
}

impl From<GpuQuerySet> for QuerySetHandle {
    fn from(query_set: GpuQuerySet) -> Self {
        QuerySetHandle { query_set }
    }
}

impl Drop for QuerySetHandle {
    fn drop(&mut self) {
        self.query_set.destroy();
    }
}

pub struct OcclusionQuerySet {
    pub(crate) inner: Arc<QuerySetHandle>,
    len: u32,
}

impl OcclusionQuerySet {
    pub(crate) fn new(device: &Device, len: u32) -> Self {
        assert!(len < 8192, "query set len must be less than `8192`");

        let desc = GpuQuerySetDescriptor::new(len, GpuQueryType::Occlusion);
        let query_set = device.inner.create_query_set(&desc);

        OcclusionQuerySet {
            inner: Arc::new(query_set.into()),
            len,
        }
    }

    pub(crate) fn as_web_sys(&self) -> &GpuQuerySet {
        &self.inner.query_set
    }

    pub fn len(&self) -> u32 {
        self.len
    }
}

pub struct TimestampQuerySet {
    pub(crate) inner: Arc<QuerySetHandle>,
    len: u32,
}

impl TimestampQuerySet {
    pub(crate) fn new(device: &Device, len: u32) -> Self {
        assert!(len < 8192, "query set len must be less than `8192`");

        let desc = GpuQuerySetDescriptor::new(len, GpuQueryType::Timestamp);
        let query_set = device.inner.create_query_set(&desc);

        TimestampQuerySet {
            inner: Arc::new(query_set.into()),
            len,
        }
    }

    pub(crate) fn as_web_sys(&self) -> &GpuQuerySet {
        &self.inner.query_set
    }

    pub fn len(&self) -> u32 {
        self.len
    }
}
