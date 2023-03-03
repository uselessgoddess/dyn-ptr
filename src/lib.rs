#![no_std]
#![cfg_attr(feature = "any-ptr", allow(incomplete_features))]
#![cfg_attr(feature = "any-ptr", feature(generic_const_exprs, specialization))]
#![feature(unsize, ptr_metadata, exact_size_is_empty, trivial_bounds)]
//#![feature(specialization)]

extern crate alloc;

mod impls;

#[cfg_attr(feature = "any-ptr", allow(unused_imports))]
use alloc::boxed::Box;
use core::{
    marker::Unsize,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr::{self, Pointee},
};

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PtrRepr(pub(self) *const ());

pub unsafe fn faith_metadata<Ptr, Dyn: ?Sized>(val: Ptr) -> (PtrRepr, <Dyn as Pointee>::Metadata)
where
    Ptr: Unsize<Dyn>,
{
    let val = &*ManuallyDrop::new(val);
    (mem::transmute_copy(val), ptr::metadata(val as *const Dyn))
}

pub trait PointerLike<Dyn: ?Sized>: Sized {
    fn meta_repr(self) -> (PtrRepr, <Dyn as Pointee>::Metadata);

    fn do_dyn(self) -> DynPtr<Dyn>
    where
        Self: Unsize<Dyn>,
    {
        // SAFETY: ...
        unsafe { DynPtr::from_parts(faith_metadata::<_, Dyn>(self)) }
    }
}

struct AlignOf<const N: usize>;
struct SizeOf<const N: usize>;

trait SamePtr {}

impl SamePtr for SizeOf<{ mem::size_of::<PtrRepr>() }> {}
impl SamePtr for AlignOf<{ mem::align_of::<PtrRepr>() }> {}

#[cfg(feature = "any-ptr")]
impl<Ptr, Dyn: ?Sized> PointerLike<Dyn> for Ptr
where
    Ptr: Unsize<Dyn>,
    SizeOf<{ mem::size_of::<Ptr>() }>: SamePtr,
    AlignOf<{ mem::align_of::<Ptr>() }>: SamePtr,
{
    fn meta_repr(self) -> (PtrRepr, <Dyn as Pointee>::Metadata) {
        // SAFETY: `Ptr` has repr same as `ReprPtr` (aka `*const ()`)
        unsafe { faith_metadata::<_, Dyn>(self) }
    }
}

#[cfg(not(feature = "any-ptr"))]
impl<T, Dyn: ?Sized> PointerLike<Dyn> for Box<T>
where
    Self: Unsize<Dyn>,
{
    fn meta_repr(self) -> (PtrRepr, <Dyn as Pointee>::Metadata) {
        unsafe { faith_metadata::<_, Dyn>(self) }
    }
}

#[cfg(not(feature = "any-ptr"))]
impl<Dyn: ?Sized> PointerLike<Dyn> for *const ()
where
    Self: Unsize<Dyn>,
{
    fn meta_repr(self) -> (PtrRepr, <Dyn as Pointee>::Metadata) {
        unsafe { faith_metadata::<_, Dyn>(self) }
    }
}

#[cfg(not(feature = "any-ptr"))]
impl<Dyn: ?Sized> PointerLike<Dyn> for usize
where
    Self: Unsize<Dyn>,
{
    fn meta_repr(self) -> (PtrRepr, <Dyn as Pointee>::Metadata) {
        unsafe { faith_metadata::<_, Dyn>(self) }
    }
}

#[repr(C)]
pub struct DynPtr<Dyn: ?Sized> {
    repr: PtrRepr,
    meta: <Dyn as Pointee>::Metadata,
}

unsafe impl<Dyn: ?Sized + Sync> Sync for DynPtr<Dyn> {}
unsafe impl<Dyn: ?Sized + Send> Send for DynPtr<Dyn> {}

impl<Dyn: ?Sized> DynPtr<Dyn> {
    pub fn new<T>(val: T) -> Self
    where
        T: PointerLike<Dyn>,
    {
        // SAFETY: `T` is guaranty has repr like pointer
        unsafe { Self::from_parts(val.meta_repr()) }
    }

    pub unsafe fn from_parts((repr, meta): (PtrRepr, <Dyn as Pointee>::Metadata)) -> Self {
        Self { repr, meta }
    }

    fn const_ptr(&self) -> *const Dyn {
        ptr::from_raw_parts(ptr::addr_of!(self.repr).cast(), self.meta)
    }

    fn mut_ptr(&mut self) -> *mut Dyn {
        ptr::from_raw_parts_mut(ptr::addr_of_mut!(self.repr).cast(), self.meta)
    }
}

impl<Dyn: ?Sized> Deref for DynPtr<Dyn> {
    type Target = Dyn;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.const_ptr() }
    }
}

impl<Dyn: ?Sized> DerefMut for DynPtr<Dyn> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mut_ptr() }
    }
}

impl<Dyn: ?Sized> Drop for DynPtr<Dyn> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.mut_ptr()) }
    }
}

pub type Dyn<Trait> = DynPtr<Trait>;
