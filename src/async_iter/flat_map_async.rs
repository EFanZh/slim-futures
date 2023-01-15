use crate::async_iter::flatten::Flatten;
use crate::async_iter::map_async::MapAsync;
use crate::support::{AsyncIterator, FnMut1, FusedAsyncIterator, IntoAsyncIterator};
use core::future::IntoFuture;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct FlatMapAsync<I, F>
    where
        I: AsyncIterator,
        F: FnMut1<I::Item>,
        F::Output: IntoFuture,
        <F::Output as IntoFuture>::Output: IntoAsyncIterator,
    {
        #[pin]
        inner: Flatten<MapAsync<I, F>>,
    }
}

impl<I, F> FlatMapAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut1<I::Item>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: IntoAsyncIterator,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Flatten::new(MapAsync::new(iter, f)),
        }
    }
}

impl<I, F> Clone for FlatMapAsync<I, F>
where
    I: AsyncIterator + Clone,
    F: FnMut1<I::Item> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: IntoAsyncIterator,
    <F::Output as IntoFuture>::IntoFuture: Clone,
    <<F::Output as IntoFuture>::Output as IntoAsyncIterator>::IntoAsyncIter: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F> AsyncIterator for FlatMapAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut1<I::Item>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: IntoAsyncIterator,
{
    type Item = <<F::Output as IntoFuture>::Output as IntoAsyncIterator>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }
}

impl<I, F> FusedAsyncIterator for FlatMapAsync<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut1<I::Item>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: IntoAsyncIterator,
    <F::Output as IntoFuture>::IntoFuture: FusedFuture,
    <<F::Output as IntoFuture>::Output as IntoAsyncIterator>::IntoAsyncIter: FusedAsyncIterator,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use crate::support::AsyncIterator;
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use futures_util::{future, stream, StreamExt};
    use std::vec::Vec;

    fn flat_map_async_fn(n: u32) -> impl Future<Output = impl AsyncIterator<Item = u32> + Clone> + Clone {
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

        future::ready(Iter(n))
    }

    #[tokio::test]
    async fn test_flat_map_async() {
        let iter = stream::iter(0..6).slim_flat_map_async(flat_map_async_fn);

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [0, 1, 0, 2, 1, 0, 3, 2, 1, 0, 4, 3, 2, 1, 0],
        );
    }

    #[tokio::test]
    async fn test_flat_map_async_clone() {
        let iter = stream::iter(0..6).slim_flat_map_async(flat_map_async_fn);
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
