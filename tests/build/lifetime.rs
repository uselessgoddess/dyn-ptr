#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use dyn_ptr::{Dyn, PointerLike};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

struct Async<'a> {
    val: &'a i32,
}

impl Future for Async<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        println!("{}", self.val);
        Poll::Ready(())
    }
}

fn impl_<'a>(val: &'a i32) -> impl Future + 'a {
    Async { val }
}

fn dyn_<'a>(val: &'a i32) -> Dyn<dyn Future<Output = ()> + 'a> {
    Async { val }.do_dyn()
}

fn main() {}
