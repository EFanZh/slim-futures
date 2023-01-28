use crate::support::{AsyncIterator, FusedAsyncIterator, IntoAsyncIterator, OptionExt};
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;

#[derive(Clone)]
struct FlattenFn<F> {
    inner: F,
}

impl<T, F, U> FnMut<((), T)> for FlattenFn<F>
where
    F: FnMut<(T,), Output = Option<U>>,
{
    type Output = ControlFlow<U>;

    fn call_mut(&mut self, args: ((), T)) -> Self::Output {
        self.inner
            .call_mut((args.1,))
            .map_or(ControlFlow::Continue(()), ControlFlow::Break)
    }
}

pin_project_lite::pin_project! {
    pub struct Flatten<I>
    where
        I: AsyncIterator,
        I::Item: IntoAsyncIterator,
    {
        #[pin]
        iter: I,
        #[pin]
        sub_iter: Option<<I::Item as IntoAsyncIterator>::IntoAsyncIter>,
    }
}

impl<I> Flatten<I>
where
    I: AsyncIterator,
    I::Item: IntoAsyncIterator,
{
    pub(crate) fn new(iter: I) -> Self {
        Self { iter, sub_iter: None }
    }
}

impl<I> Clone for Flatten<I>
where
    I: AsyncIterator + Clone,
    I::Item: IntoAsyncIterator,
    <I::Item as IntoAsyncIterator>::IntoAsyncIter: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            sub_iter: self.sub_iter.clone(),
        }
    }
}

impl<I> AsyncIterator for Flatten<I>
where
    I: AsyncIterator,
    I::Item: IntoAsyncIterator,
{
    type Item = <I::Item as IntoAsyncIterator>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let mut sub_iter_slot = this.sub_iter;

        Poll::Ready(loop {
            let sub_iter = match sub_iter_slot.as_mut().as_pin_mut() {
                None => match task::ready!(iter.as_mut().poll_next(cx)) {
                    None => break None,
                    Some(into_sub_iter) => sub_iter_slot.as_mut().insert_pinned(into_sub_iter.into_async_iter()),
                },
                Some(sub_iter) => sub_iter,
            };

            let item = task::ready!(sub_iter.poll_next(cx));

            if item.is_some() {
                break item;
            }

            sub_iter_slot.set(None);
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<I> FusedAsyncIterator for Flatten<I>
where
    I: FusedAsyncIterator,
    I::Item: IntoAsyncIterator,
    <I::Item as IntoAsyncIterator>::IntoAsyncIter: FusedAsyncIterator,
{
    fn is_terminated(&self) -> bool {
        self.sub_iter
            .as_ref()
            .map_or_else(|| self.iter.is_terminated(), FusedAsyncIterator::is_terminated)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    #[tokio::test]
    async fn test_flatten() {
        let iter = stream::iter([
            stream::iter(std::vec![]),
            stream::iter(std::vec![2]),
            stream::iter(std::vec![3, 5]),
            stream::iter(std::vec![7, 11, 13]),
        ])
        .slim_flatten();

        assert_eq!(iter.collect::<Vec<_>>().await, [2, 3, 5, 7, 11, 13]);
    }

    #[tokio::test]
    async fn test_flatten_clone() {
        let iter = stream::iter([
            stream::iter(std::vec![]),
            stream::iter(std::vec![2]),
            stream::iter(std::vec![3, 5]),
            stream::iter(std::vec![7, 11, 13]),
        ])
        .slim_flatten();

        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [2, 3, 5, 7, 11, 13]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [2, 3, 5, 7, 11, 13]);
    }
}
