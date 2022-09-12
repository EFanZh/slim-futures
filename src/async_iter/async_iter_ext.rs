use crate::async_iter::Fold;
use crate::support::AsyncIterator;

pub trait AsyncIteratorExt: AsyncIterator {
    fn fold<B, F>(self, init: B, f: F) -> Fold<Self, B, F>
    where
        Self: Sized,
        B: Copy,
        F: FnMut(B, Self::Item) -> B,
    {
        crate::support::assert_future::<_, B>(Fold::new(self, init, f))
    }
}

impl<I> AsyncIteratorExt for I where I: AsyncIterator {}
