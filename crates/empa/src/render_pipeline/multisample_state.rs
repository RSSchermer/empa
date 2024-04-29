use web_sys::GpuMultisampleState;

pub struct MultisampleState<const SAMPLES: u8> {
    pub(crate) inner: GpuMultisampleState,
}

impl<const SAMPLES: u8> MultisampleState<SAMPLES> {
    pub fn new() -> Self {
        let mut inner = GpuMultisampleState::new();

        if SAMPLES <= 1 {
            panic!("sample count must be more than `1`");
        }

        inner.count(SAMPLES as u32);

        MultisampleState { inner }
    }

    pub fn mask(mut self, mask: u32) -> Self {
        self.inner.mask(mask);

        self
    }

    pub fn enable_alpha_to_coverage(mut self) -> Self {
        self.inner.alpha_to_coverage_enabled(true);

        self
    }
}
