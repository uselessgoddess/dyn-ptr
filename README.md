A box that stores types like pointers,
forgetting everything besides `Self: Unsize<dyn Trait>`

### Basic Usage
```rust
use dyn_ptr::{Dyn, PointerLike};

fn impl_(_: impl fmt::Display) {}
fn dyn_(x: Dyn<dyn fmt::Display>) { /* sizeof(x) == (ptr + metadata) */ }

impl_(12);
impl_(12_usize);
impl_(Box::new(12));

dyn_((&12).do_dyn()); // ref to `i32` is repr as `PtrRepr`
dyn_(12_usize.do_dyn()); // `usize` is repr as `PtrRepr`
dyn_(Box::new(12).do_dyn()); // `usize` is repr as `PtrRepr`
```
Instead of `*const dyn` and `*mut dyn` fat pointers, that fat pointer store dtor and works like `Box`.\
And accept by value (instead of ref/ptr)

This can be used for example in async traits, in which you cannot use `Box<dyn Future>`, because `core` has no allocators.
`Dyn` can help erase type about store (for example `Pin<Box>`)
```rust
// possible desugared for `async fn in dyn traits`
trait Async<'a> {
    // original: `async async_fn() -> &'a ();`
    fn async_fn<C: Ctx<'a>>() -> C::ImplFuture {
        // one of possible implementation:
        todo!(/* async { &() } */)
    }
}

// select: `dyn` or `impl` context
struct DynCtx;
struct ImplCtx;

trait Ctx<'a> {
    type ImplFuture: Future<Output = &'a ()>;

    fn do_future() -> Self::ImplFuture;
}

impl<'a> Ctx<'a> for DynCtx {
    type ImplFuture = Dyn<dyn Future<Output = &'a ()> + Unpin>;

    fn do_future() -> Self::ImplFuture {
        Box::pin(async { &() }).do_dyn()
    }
}

impl<'a> Ctx<'a> for ImplCtx {
    type ImplFuture = impl Future<Output = &'a ()>;

    fn do_future() -> Self::ImplFuture {
        async { &() }
    }
}

```

### Features
By default, unstable `generic_const_exprs` is enabled. It is supported in the first place. The default version lags behind 