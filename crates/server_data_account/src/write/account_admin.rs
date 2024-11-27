mod news;

pub struct WriteCommandsAccountAdmin<C>(C);

impl<C> WriteCommandsAccountAdmin<C> {
    pub fn new(c: C) -> Self {
        Self(c)
    }

    pub fn news(self) -> news::WriteCommandsAccountNewsAdmin<C> {
        news::WriteCommandsAccountNewsAdmin::new(self.0)
    }
}
