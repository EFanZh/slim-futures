use crate::async_iter::fold::Fold;
use crate::async_iter::fold_async::FoldAsync;
use crate::support::AsyncIterator;
use std::future::Future;

pub trait AsyncIteratorExt: AsyncIterator {
    fn fold<B, F>(self, init: B, f: F) -> Fold<Self, B, F>
    where
        Self: Sized,
        B: Copy,
        F: FnMut(B, Self::Item) -> B,
    {
        crate::support::assert_future::<_, B>(Fold::new(self, init, f))
    }

    fn fold_async<B, F, Fut>(self, init: B, f: F) -> FoldAsync<Self, B, F>
    where
        Self: Sized,
        B: Copy,
        F: FnMut(B, Self::Item) -> Fut,
        Fut: Future<Output = B>,
    {
        crate::support::assert_future::<_, B>(FoldAsync::new(self, init, f))
    }
}

impl<I> AsyncIteratorExt for I where I: AsyncIterator {}