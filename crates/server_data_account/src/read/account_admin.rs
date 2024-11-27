
pub mod news;

pub struct ReadCommandsAccountAdmin<C>(C);

impl<C> ReadCommandsAccountAdmin<C> {
    pub fn new(c: C) -> Self {
        Self(c)
    }

    pub fn news(self) -> news::ReadCommandsAccountNewsAdmin<C> {
        news::ReadCommandsAccountNewsAdmin::new(self.0)
    }
}
