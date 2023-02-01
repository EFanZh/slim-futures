use crate::async_iter::flatten::Flatten;
use crate::async_iter::map::Map;
use crate::support::{AsyncIterator, FusedAsyncIterator, IntoAsyncIterator};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct FlatMap<I, F>
    where
        I: AsyncIterator,
        F: FnMut<(I::Item,)>,
        F: ?Sized,
        F::Output: IntoAsyncIterator,
    {
        #[pin]
        inner: Flatten<Map<I, F>>,
    }
}

impl<I, F> FlatMap<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)>,
    F::Output: IntoAsyncIterator,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Flatten::new(Map::new(iter, f)),
        }
    }
}

impl<I, F> Clone for FlatMap<I, F>
where
    I: AsyncIterator + Clone,
    F: FnMut<(I::Item,)> + Clone,
    F::Output: IntoAsyncIterator,
    <F::Output as IntoAsyncIterator>::IntoAsyncIter: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F> AsyncIterator for FlatMap<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)> + ?Sized,
    F::Output: IntoAsyncIterator,
{
    type Item = <F::Output as IntoAsyncIterator>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I, F> FusedAsyncIterator for FlatMap<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut<(I::Item,)> + ?Sized,
    F::Output: IntoAsyncIterator,
    <F::Output as IntoAsyncIterator>::IntoAsyncIter: FusedAsyncIterator,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use crate::support::AsyncIterator;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    fn flat_map_fn(n: u32) -> impl AsyncIterator<Item = u32> + Clone {
        #[derive(Clone)]
        struct Iter(u32);

        impl AsyncIterator for Iter {
            type Item = u32;

            fn poll_next(self: Pin<&mut Self>, _: &mut Context) -> Poll<Option<Self::Item>> {
                let this = self.get_mut();

                Poll::Ready(if this.0 == 0 {
                    None
                } else {
                    this.0 -= 1;

                    Some(this.0)
                })
            }
        }

        Iter(n)
    }

    #[tokio::test]
    async fn test_flat_map() {
        let iter = stream::iter(0..6).slim_flat_map(flat_map_fn);

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [0, 1, 0, 2, 1, 0, 3, 2, 1, 0, 4, 3, 2, 1, 0],
        );
    }

    #[tokio::test]
    async fn test_flat_map_clone() {
        let iter = stream::iter(0..6).slim_flat_map(flat_map_fn);
        let iter_2 = iter.clone();

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [0, 1, 0, 2, 1, 0, 3, 2, 1, 0, 4, 3, 2, 1, 0],
        );

        assert_eq!(
            iter_2.collect::<Vec<_>>().await,
            [0, 1, 0, 2, 1, 0, 3, 2, 1, 0, 4, 3, 2, 1, 0],
        );
    }
}
