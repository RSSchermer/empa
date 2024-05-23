use crate::device::Device;
use crate::driver::{Device as _, Driver, Dvr, QuerySetDescriptor, QueryType};

pub struct OcclusionQuerySet {
    pub(crate) handle: <Dvr as Driver>::QuerySetHandle,
    len: usize,
}

impl OcclusionQuerySet {
    pub(crate) fn new(device: &Device, len: usize) -> Self {
        assert!(len < 8192, "query set len must be less than `8192`");

        let handle = device.handle.create_query_set(&QuerySetDescriptor {
            query_type: QueryType::Occlusion,
            len,
        });

        OcclusionQuerySet { handle, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

pub struct TimestampQuerySet {
    pub(crate) handle: <Dvr as Driver>::QuerySetHandle,
    len: usize,
}

impl TimestampQuerySet {
    pub(crate) fn new(device: &Device, len: usize) -> Self {
        assert!(len < 8192, "query set len must be less than `8192`");

        let handle = device.handle.create_query_set(&QuerySetDescriptor {
            query_type: QueryType::Timestamp,
            len,
        });

        TimestampQuerySet { handle, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
