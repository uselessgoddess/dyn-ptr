

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
    type ImplFuture: Future<Output=&'a ()>;

    fn do_future() -> Self::ImplFuture;
}

impl<'a> Ctx<'a> for DynCtx {
    type ImplFuture = Dyn<dyn Future<Output=&'a ()> + Unpin>;

    fn do_future() -> Self::ImplFuture {
        Box::pin(async { &() }).do_dyn()
    }
}

impl<'a> Ctx<'a> for ImplCtx {
    type ImplFuture = impl Future<Output=&'a ()>;

    fn do_future() -> Self::ImplFuture {
        async { &() }
    }
}

```
