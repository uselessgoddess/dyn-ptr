#![feature(trivial_bounds, generic_const_exprs, ptr_metadata)]

use dyn_ptr::{Dyn, PointerLike, PtrRepr};
use std::{fmt, future::Future, io, mem, ptr::Pointee};

#[test]
fn basic() {
    trait A {}
    trait B: A {}

    // but also use `impl Trait`. `Dyn`
    fn display_eq(a: impl fmt::Display, b: Dyn<dyn fmt::Display>) {
        // need to use manual deref
        assert_eq!(format!("{a}"), format!("{}", &*b))
    }

    display_eq(2, Dyn::new(2usize));
    display_eq(2, 2usize.do_dyn());
}

#[test]
fn as_impl() {
    fn impl_(_: impl fmt::Display) {}
    fn dyn_(x: Dyn<dyn fmt::Display>) {
        assert_eq!(
            mem::size_of_val(&x),
            mem::size_of::<PtrRepr>() + mem::size_of::<<dyn fmt::Display as Pointee>::Metadata>()
        );
        assert_eq!(
            mem::size_of_val(&x),
            mem::size_of::<PtrRepr>() + mem::size_of::<<dyn fmt::Display as Pointee>::Metadata>()
        );
    }

    impl_(12);
    impl_(12_usize);
    impl_(Box::new(12));

    dyn_((&12).do_dyn()); // ref to `i32` has `PtrRepr` repr
    dyn_(12_usize.do_dyn()); // `usize` has `PtrRepr` repr
    dyn_(Box::new(12).do_dyn()); // `usize` has `PtrRepr` repr
}

#[test]
fn generic() {
    fn from_impl<T: fmt::Display + 'static>(t: T) -> Dyn<dyn fmt::Display>
    where
        T: PointerLike<dyn fmt::Display + 'static>,
    {
        t.do_dyn()
    }
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
