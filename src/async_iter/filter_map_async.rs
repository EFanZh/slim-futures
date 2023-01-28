use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::future::IntoFuture;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::{FusedFuture, Future};

pin_project_lite::pin_project! {
    pub struct FilterMapAsync<I, F>
    where
        I: AsyncIterator,
        F: FnMut<(I::Item,)>,
        F: ?Sized,
        F::Output: IntoFuture,
    {
        #[pin]
        iter: I,
        #[pin]
        fut: Option<<F::Output as IntoFuture>::IntoFuture>,
        f: F,
    }
}

impl<I, F> FilterMapAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self { iter, f, fut: None }
    }
}

impl<I, F> Clone for FilterMapAsync<I, F>
where
    I: AsyncIterator + Clone,
    F: FnMut<(I::Item,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            f: self.f.clone(),
            fut: self.fut.clone(),
        }
    }
}

impl<I, F, T> AsyncIterator for FilterMapAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)> + ?Sized,
    F::Output: IntoFuture<Output = Option<T>>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let f = this.f;
        let mut fut_slot = this.fut;

        Poll::Ready(loop {
            match fut_slot.as_mut().as_pin_mut() {
                None => {}
                Some(fut) => {
                    let item = task::ready!(fut.poll(cx));

                    fut_slot.set(None);

                    if item.is_some() {
                        break item;
                    }
                }
            }

            match task::ready!(iter.as_mut().poll_next(cx)) {
                None => break None,
                Some(item) => fut_slot.set(Some(f.call_mut((item,)).into_future())),
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let high = self.iter.size_hint().1;

        let high = match &self.fut {
            None => high,
            Some(..) => high.and_then(|high| high.checked_add(1)),
        };

        (0, high)
    }
}

impl<I, F, T> FusedAsyncIterator for FilterMapAsync<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut<(I::Item,)> + ?Sized,
    F::Output: IntoFuture<Output = Option<T>>,
    <F::Output as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        match &self.fut {
            None => self.iter.is_terminated(),
            Some(fut) => fut.is_terminated(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::future::{self, Ready};
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    fn filter_map_fn(x: u32) -> Ready<Option<u32>> {
        future::ready((x % 2 == 0).then_some(x * 10))
    }

    #[tokio::test]
    async fn test_filter_map_async() {
        let iter = stream::iter(0..10).slim_filter_map_async(filter_map_fn);

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 20, 40, 60, 80]);
    }

    #[tokio::test]
    async fn test_filter_map_async_clone() {
        let iter = stream::iter(0..10).slim_filter_map_async(filter_map_fn);
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 20, 40, 60, 80]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [0, 20, 40, 60, 80]);
    }
}
