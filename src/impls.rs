use crate::DynPtr;
use core::{
    future::Future,
    iter::FusedIterator,
    pin::Pin,
    task::{Context, Poll},
};

impl<F: ?Sized + Future + Unpin> Future for DynPtr<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut **self).poll(cx)
    }
}

impl<I: Iterator + ?Sized> Iterator for DynPtr<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        (**self).next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
    fn last(self) -> Option<I::Item> {
        #[inline]
        fn some<T>(_: Option<T>, x: T) -> Option<T> {
            Some(x)
        }

        self.fold(None, some)
    }
    fn nth(&mut self, n: usize) -> Option<I::Item> {
        (**self).nth(n)
    }
}

impl<I: DoubleEndedIterator + ?Sized> DoubleEndedIterator for DynPtr<I> {
    fn next_back(&mut self) -> Option<I::Item> {
        (**self).next_back()
    }
    fn nth_back(&mut self, n: usize) -> Option<I::Item> {
        (**self).nth_back(n)
    }
}

impl<I: ExactSizeIterator + ?Sized> ExactSizeIterator for DynPtr<I> {
    fn len(&self) -> usize {
        (**self).len()
    }
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }
}

impl<I: FusedIterator + ?Sized> FusedIterator for DynPtr<I> {}
