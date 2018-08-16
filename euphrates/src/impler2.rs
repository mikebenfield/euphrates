
macro_rules! closure_fn {
    ($name: ident; typ: $ty) => {
        fn $name<F, T>(&mut self) -> T
        where
            F: FnOnce(&mut $ty) -> T;

    }
}
