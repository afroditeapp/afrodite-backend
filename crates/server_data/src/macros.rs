
#[macro_export]
macro_rules! define_cmd_wrapper {
    ($struct_name:ident) => {
        pub struct $struct_name<C>(C);

        impl<C> $struct_name<C> {
            pub fn new(c: C) -> Self {
                Self(c)
            }
        }

        impl <C> core::ops::Deref for $struct_name<C> {
            type Target = C;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    }
}
