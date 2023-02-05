use crate::support::{AsyncIterator, FromResidual, FusedAsyncIterator, IntoAsyncIterator, Try};
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use option_entry::{OptionEntryExt, OptionPinnedEntry};

#[derive(Clone)]
struct TryFlattenFn<F> {
    f: F,
}

impl<T, F, U> FnMut<((), T)> for TryFlattenFn<F>
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
    pub struct TryFlatten<I>
    where
        I: AsyncIterator,
        I: ?Sized,
        I::Item: Try,
        <I::Item as Try>::Output: IntoAsyncIterator,
    {
        #[pin]
        state: Option<<<I::Item as Try>::Output as IntoAsyncIterator>::IntoAsyncIter>,
        #[pin]
        iter: I,
    }
}

impl<I> TryFlatten<I>
where
    I: AsyncIterator,
    I::Item: Try,
    <I::Item as Try>::Output: IntoAsyncIterator,
{
    pub(crate) fn new(iter: I) -> Self {
        Self { state: None, iter }
    }
}

impl<I> Clone for TryFlatten<I>
where
    I: AsyncIterator + Clone,
    I::Item: Try,
    <I::Item as Try>::Output: IntoAsyncIterator,
    <<I::Item as Try>::Output as IntoAsyncIterator>::IntoAsyncIter: Clone,
{
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            iter: self.iter.clone(),
        }
    }
}

impl<I> AsyncIterator for TryFlatten<I>
where
    I: AsyncIterator + ?Sized,
    I::Item: Try,
    <I::Item as Try>::Output: IntoAsyncIterator,
    <<I::Item as Try>::Output as IntoAsyncIterator>::Item: FromResidual<<I::Item as Try>::Residual>,
{
    type Item = <<I::Item as Try>::Output as IntoAsyncIterator>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut state = this.state.pinned_entry();
        let mut iter = this.iter;

        loop {
            let mut sub_iter = match state {
                OptionPinnedEntry::None(none_state) => match task::ready!(iter.as_mut().poll_next(cx)) {
                    None => break Poll::Ready(None),
                    Some(item) => match item.branch() {
                        ControlFlow::Continue(output) => none_state.set_some(output.into_async_iter()),
                        ControlFlow::Break(residual) => break Poll::Ready(Some(Self::Item::from_residual(residual))),
                    },
                },
                OptionPinnedEntry::Some(some_state) => some_state,
            };

            let item = task::ready!(sub_iter.get_pin_mut().poll_next(cx));

            if let Some(item) = item {
                break Poll::Ready(Some(item));
            }

            state = OptionPinnedEntry::None(sub_iter.set_none());
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

impl<I> FusedAsyncIterator for TryFlatten<I>
where
    I: FusedAsyncIterator + ?Sized,
    I::Item: Try,
    <I::Item as Try>::Output: IntoAsyncIterator,
    <<I::Item as Try>::Output as IntoAsyncIterator>::Item: FromResidual<<I::Item as Try>::Residual>,
    <<I::Item as Try>::Output as IntoAsyncIterator>::IntoAsyncIter: FusedAsyncIterator,
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
    async fn test_try_flatten() {
        let iter = stream::iter([
            Ok(stream::iter(std::vec![Ok(2), Err(3)])),
            Err(5),
            Ok(stream::iter(std::vec![Ok(7), Err(11), Ok(13)])),
        ])
        .slim_try_flatten();

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(2), Err(3), Err(5), Ok(7), Err(11), Ok(13)],
        );
    }

    #[tokio::test]
    async fn test_try_flatten_clone() {
        let iter = stream::iter([
            Ok(stream::iter(std::vec![Ok(2), Err(3)])),
            Err(5),
            Ok(stream::iter(std::vec![Ok(7), Err(11), Ok(13)])),
        ])
        .slim_try_flatten();

        let iter_2 = iter.clone();

        assert_eq!(
            iter.collect::<Vec<_>>().await,
            [Ok(2), Err(3), Err(5), Ok(7), Err(11), Ok(13)],
        );

        assert_eq!(
            iter_2.collect::<Vec<_>>().await,
            [Ok(2), Err(3), Err(5), Ok(7), Err(11), Ok(13)],
        );
    }
}
