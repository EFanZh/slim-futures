use crate::support::AsyncIterator;

pub trait IntoAsyncIterator {
    type Item;
    type IntoAsyncIter: AsyncIterator<Item = Self::Item>;

    fn into_async_iter(self) -> Self::IntoAsyncIter;
}

impl<I> IntoAsyncIterator for I
where
    I: AsyncIterator,
{
    type Item = I::Item;
    type IntoAsyncIter = Self;

    fn into_async_iter(self) -> Self::IntoAsyncIter {
        self
    }
}
