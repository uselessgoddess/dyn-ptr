use dyn_ptr::{Dyn, PointerLike};
use std::{fmt, future::Future, io};

#[test]
fn basic() {
    trait A {}
    trait B: A {}

    fn display_eq(a: impl fmt::Display, b: Dyn<dyn fmt::Display>) {
        // need to use manual deref
        assert_eq!(format!("{a}"), format!("{}", &*b))
    }

    display_eq(2, Dyn::new(2));
    display_eq(2, 2.do_dyn());
}

#[test]
fn futures() {
    pub trait DynAsync {
        // fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
        fn read(&mut self, buf: &mut [u8]) -> Dyn<dyn Future<Output = io::Result<usize>>>;
    }

    impl DynAsync for () {
        fn read(&mut self, _: &mut [u8]) -> Dyn<dyn Future<Output = io::Result<usize>>> {
            // Box::pin(...)
            // OtherBox::new(...)
            // SmallFuture::new(...)
            // PreAllocatedFuture::from(..)
            todo!()
        }
    }
}
