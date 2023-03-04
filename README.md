## What if the `dyn` were something of known size?

With normal usage `async` (whether `std` or `core`) you can write steadily:

```rust
async fn async_things(...) -> T {}

// it is equivalent to:
fn async_things(...) -> impl Future<Output=T> {}
```

This is okay, because in place of `impl` can be any type, and we do not care where it is and how its memory is freed. \
Everything stays fine until you want to `async trait'.
Imagine that you have such a trait:

```rust
mod io {
    type Result<T> = ...;
}

pub trait AsyncRead {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;

    ...
}
```

Previously you would have had to rewrite `read' like this:

```rust
fn poll_read(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut [u8]
) -> Poll<io::Result<usize>>;
```

But now you can write it this way:

```rust
fn read(&mut self, buf: &mut [u8]) -> impl Future<Output=io::Result<usize>>;
// aka: async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
```

This creates a new type implementing `Future` for each individual implementation, so it is not `object-safety`:

```
error[E0038]: the trait `AsyncRead` cannot be made into an object
  --> src\main.rs:20:12
   |
20 |     let _: dyn AsyncRead = todo!();
   |            ^^^^^^^^^^^^^ `AsyncRead` cannot be made into an object
   |
note: for a trait to be "object safe" it needs to allow building a vtable to allow the call to be resolvable dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-safety>
  --> src\main.rs:16:43
   |
15 | pub trait AsyncRead {
   |           --------- this trait cannot be made into an object...
16 |     fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = io::Result<usize>>;
   |                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ...because method `read` references an `impl Trait` type in its return type
   = help: consider moving `read` to another trait
```

### Problem: no memory allocator in core

The obvious solution is to use `Box<dyn Future>`, But in `no-std` systems (where only `core` and `alloc` are available)
the `Box` hardcode seems redundant, since it needs allocators.

### Problem: &dyn Traits can't be substituted for impl Traits

But what if there was a type that allows you to save a type whose size is the same as the pointer along with its Vtable,
which stores the pointer to `drop`. Then we would get a kind of new fat pointer that lives as `T: 'a' (as opposed to `
&'a T' of `&dyn Trait')

### Basic Usage

In today's rast, there is no mechanism for pass `dyn Trait` to `impl Trait`:

```rust
fn print(x: impl Display) {
    println!(“{x}”);
}
```

and we can easily use it in `'static` context:

```rust
fn print_later(x: impl Display + Send + 'static) {
    thread::spawn(move || println!("{x}"));
}
```

But how can we send here &dyn Trait not by reference, but by value? Of course `Box`.

```rust
fn print_later(x: Box<dyn Display + Send>) {
    thread::spawn(move || println!("{x}"));
}
```

However, this always requires a memory allocation, which often (and `no_std` almost always) does not make sense.

### What if you just store `dyn Trait`?

But how much space does it take up? And how is it aligned? You have to know all this at compile time, and it's
definitely not about `dyn`. \
But how do you choose the size? \
Let's remember one
fact - [`Box<T>` is transparent as pointer](https://doc.rust-lang.org/nightly/std/boxed/index.html#memory-layout). Then,
if you imagine our new type, it looks something like this:

```rust
struct DynPtr<Dyn: ?Sized> {
    repr: PtrRepr,
    // is alias to `*const ()` not to be confused 
    drop: unsafe fn(*const ()),
}
```

Remind you of anything? This is very similar to the most
popular way to create [vtables](https://doc.rust-lang.org/nightly/src/core/task/wake.rs.html#82-113) in `std`.
This is what the raw creation of `DynPtr` for `Box` might look like:

```rust
let ptr = DynPtr::<dyn Display> {
repr: Box::into_raw(x) as PtrRepr,
drop: | repr | unsafe {
let _ = Box::from_raw(repr as *mut i32);
},
};
(ptr.drop)(ptr.repr); // drop original box 
```

You may notice that `from_raw`/`into_raw` can be generalized to the case of `mem::transmute` since the repr of Box
similar pointer (`*const ()`). \
Also in favor of this method would be the fact that in practice many futures consist of a very small state (which is
equal to the pointer or even smaller) or its state is also boxed in `Arc`-like.
This means that in the general case you can continue to use `Box`, which is also comparable to a pointer, and for
sufficiently thin primitives use your own transformation.

This is how `Dyn<dyn Trait>` appears (`dyn` in this case is needed because of the requirements of the `2021 edition` of
Rust), which can be used almost as easily as `impl Trait`:

```rust
fn print_later(x: Dyn<dyn Display + Send + 'static>) {
    thread::spawn(move || println!("{}", &*x)); // `Dyn` is not automatically delegate traits like `dyn`
}

// use `.do_dyn()` to coerce type into `Dyn<dyn ...>`
let x = & 12;
print_later(x.do_dyn());
print_later(12_usize.do_dyn());
print_later(Box::new(x).do_dyn());
```
