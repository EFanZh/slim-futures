use crate::support::AsyncIterator;
use core::future::IntoFuture;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::Future;

pin_project_lite::pin_project! {
    pub struct MapWhileAsync<I, F>
    where
        I: AsyncIterator,
        F: FnMut<(I::Item,)>,
        F::Output: IntoFuture,
    {
        #[pin]
        iter: I,
        f: F,
        #[pin]
        fut: Option<<F::Output as IntoFuture>::IntoFuture>,
    }
}

impl<I, F> MapWhileAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self { iter, f, fut: None }
    }
}

impl<I, F> Clone for MapWhileAsync<I, F>
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

impl<I, F, T> AsyncIterator for MapWhileAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)>,
    F::Output: IntoFuture<Output = Option<T>>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut fut_slot = this.fut;

        Poll::Ready('outer: {
            let fut = match fut_slot.as_mut().as_pin_mut() {
                None => {
                    match task::ready!(this.iter.poll_next(cx)) {
                        None => break 'outer None,
                        Some(item) => fut_slot.set(Some(this.f.call_mut((item,)).into_future())),
                    }

                    fut_slot.as_mut().as_pin_mut().unwrap()
                }
                Some(fut) => fut,
            };

            let item = task::ready!(fut.poll(cx));

            fut_slot.set(None);

            item
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::future::{self, Ready};
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    fn map_while_async_fn(x: u32) -> Ready<Option<u32>> {
        future::ready((x < 5).then_some(x * 10))
    }

    #[tokio::test]
    async fn test_map_while_async() {
        let iter = stream::iter(0..10).slim_map_while_async(map_while_async_fn);

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 10, 20, 30, 40]);
    }

    #[tokio::test]
    async fn test_map_while_async_clone() {
        let iter = stream::iter(0..10).slim_map_while_async(map_while_async_fn);
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 10, 20, 30, 40]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [0, 10, 20, 30, 40],);
    }
}
