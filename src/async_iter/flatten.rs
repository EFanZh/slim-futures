use crate::support::{AsyncIterator, FusedAsyncIterator, IntoAsyncIterator};
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use option_entry::{OptionEntryExt, OptionPinnedEntry};

#[derive(Clone)]
struct FlattenFn<F> {
    f: F,
}

impl<T, F, U> FnMut<((), T)> for FlattenFn<F>
where
    F: FnMut<(T,), Output = Option<U>>,
{
    type Output = ControlFlow<U>;

    fn call_mut(&mut self, args: ((), T)) -> Self::Output {
        self.f
            .call_mut((args.1,))
            .map_or(ControlFlow::Continue(()), ControlFlow::Break)
    }
}

pin_project_lite::pin_project! {
    pub struct Flatten<I>
    where
        I: AsyncIterator,
        I: ?Sized,
        I::Item: IntoAsyncIterator,
    {
        #[pin]
        state: Option<<I::Item as IntoAsyncIterator>::IntoAsyncIter>,
        #[pin]
        iter: I,
    }
}

impl<I> Flatten<I>
where
    I: AsyncIterator,
    I::Item: IntoAsyncIterator,
{
    pub(crate) fn new(iter: I) -> Self {
        Self { state: None, iter }
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
            state: self.state.clone(),
            iter: self.iter.clone(),
        }
    }
}

impl<I> AsyncIterator for Flatten<I>
where
    I: AsyncIterator + ?Sized,
    I::Item: IntoAsyncIterator,
{
    type Item = <I::Item as IntoAsyncIterator>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut state = this.state.pinned_entry();
        let mut iter = this.iter;

        loop {
            let mut sub_iter = match state {
                OptionPinnedEntry::None(none_state) => match task::ready!(iter.as_mut().poll_next(cx)) {
                    None => break Poll::Ready(None),
                    Some(into_state) => none_state.replace_some(into_state.into_async_iter()),
                },
                OptionPinnedEntry::Some(some_state) => some_state,
            };

            let item = task::ready!(sub_iter.get_pin_mut().poll_next(cx));

            if let Some(item) = item {
                break Poll::Ready(Some(item));
            }

            state = OptionPinnedEntry::None(sub_iter.replace_none());
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let has_more = self.iter.size_hint().1 != Some(0);

        self.state.as_ref().map_or_else(
            || (0, (!has_more).then_some(0)),
            |state| {
                let mut candidate = state.size_hint();

                if has_more {
                    candidate.1 = None;
                }

                candidate
            },
        )
    }
}

impl<I> FusedAsyncIterator for Flatten<I>
where
    I: FusedAsyncIterator + ?Sized,
    I::Item: IntoAsyncIterator,
    <I::Item as IntoAsyncIterator>::IntoAsyncIter: FusedAsyncIterator,
{
    fn is_terminated(&self) -> bool {
        self.state
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
