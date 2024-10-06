
#[derive(Debug, Clone, Copy, Default)]
pub struct DbIteratorState {
    pub page: u64,
}

impl DbIteratorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next(self) -> Self {
        Self {
            page: self.page.saturating_add(1),
        }
    }
}
