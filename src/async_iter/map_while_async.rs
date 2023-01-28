use crate::support::{AsyncIterator, OptionExt};
use core::future::IntoFuture;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::Future;

pin_project_lite::pin_project! {
    pub struct MapWhileAsync<I, F>
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

impl<I, F> MapWhileAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self { iter, fut: None, f }
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
            fut: self.fut.clone(),
            f: self.f.clone(),
        }
    }
}

impl<I, F, T> AsyncIterator for MapWhileAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)> + ?Sized,
    F::Output: IntoFuture<Output = Option<T>>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let mut fut_slot = this.fut;
        let f = this.f;

        match fut_slot.as_mut().get_or_try_insert_with_pinned(|| {
            ControlFlow::Break(match iter.as_mut().poll_next(cx) {
                Poll::Ready(item) => match item {
                    None => Poll::Ready(None),
                    Some(item) => return ControlFlow::Continue(f.call_mut((item,)).into_future()),
                },
                Poll::Pending => Poll::Pending,
            })
        }) {
            ControlFlow::Continue(fut) => {
                let item = task::ready!(fut.poll(cx));

                fut_slot.set(None);

                Poll::Ready(item)
            }
            ControlFlow::Break(result) => result,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut candidate = (0, self.iter.size_hint().1);

        if self.fut.is_some() {
            candidate.1 = candidate.1.and_then(|high| high.checked_add(1));
        }

        candidate
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
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
