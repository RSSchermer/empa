use crate::driver;

pub struct MultisampleState<const SAMPLES: u8> {
    pub(crate) inner: driver::MultisampleState,
}

impl<const SAMPLES: u8> MultisampleState<SAMPLES> {
    pub fn new() -> Self {
        if SAMPLES <= 1 {
            panic!("sample count must be more than `1`");
        }

        MultisampleState {
            inner: driver::MultisampleState {
                count: SAMPLES as u32,
                mask: 0xFFFFFFF,
                alpha_to_coverage_enabled: false,
            },
        }
    }

    pub fn mask(mut self, mask: u32) -> Self {
        self.inner.mask = mask;

        self
    }

    pub fn enable_alpha_to_coverage(mut self) -> Self {
        self.inner.alpha_to_coverage_enabled = true;

        self
    }
}
