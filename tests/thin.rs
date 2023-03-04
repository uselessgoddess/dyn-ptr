#![feature(generic_const_exprs)]

use dyn_ptr::{Dyn, PointerLike, PtrRepr};
use std::{fmt::Display, future::Future, mem, mem::ManuallyDrop};

trait Trait {
    fn do_work(&self) -> i32;
}

#[derive(Copy, Clone)]
struct Thin(i32);
struct Fat(i32, [u8; 128]);

impl Trait for Thin {
    fn do_work(&self) -> i32 {
        self.0
    }
}

union DynRepr {
    inner: Thin,
    _repr: PtrRepr,
}

// rust has no delegation of traits :(
impl Trait for DynRepr {
    fn do_work(&self) -> i32 {
        unsafe { self.inner.do_work() }
    }
}

impl Trait for Fat {
    fn do_work(&self) -> i32 {
        self.0
    }
}

impl Trait for Box<Fat> {
    fn do_work(&self) -> i32 {
        (**self).do_work()
    }
}

fn thin() -> Dyn<dyn Trait> {
    // initialize zeroed to next `transmute_copy`
    let mut uni = DynRepr { _repr: unsafe { mem::zeroed() } };
    uni.inner = Thin(234);
    uni.do_dyn()
}

fn fat() -> Dyn<dyn Trait> {
    Box::new(Fat(234, [0; 128])).do_dyn()
}

#[test]
fn thin_ptr() {
    assert_eq!(thin().do_work(), fat().do_work())
}
