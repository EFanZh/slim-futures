use crate::support::states::{PredicateState, PredicateStateProject};
use crate::support::{AsyncIterator, FusedAsyncIterator, PredicateFn};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use futures_core::FusedFuture;
use option_entry::{OptionEntryExt, OptionPinnedEntry};

pin_project_lite::pin_project! {
    #[derive(Clone)]
    struct State<F, T, Fut> {
        #[pin]
        predicate_state: PredicateState<T, Fut>,
        f: F,
    }
}

type StateType<I, F> = State<
    F,
    <I as AsyncIterator>::Item,
    <<F as PredicateFn<<I as AsyncIterator>::Item>>::Output as IntoFuture>::IntoFuture,
>;

pin_project_lite::pin_project! {
    pub struct SkipWhileAsync<I, F>
    where
        I: AsyncIterator,
        F: PredicateFn<I::Item>,
        <F as PredicateFn<I::Item>>::Output: IntoFuture,
    {
        #[pin]
        iter: I,
        #[pin]
        state: Option<StateType<I, F>>,
    }
}

impl<I, F> SkipWhileAsync<I, F>
where
    I: AsyncIterator,
    F: PredicateFn<I::Item>,
    <F as PredicateFn<I::Item>>::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            iter,
            state: Some(State {
                f,
                predicate_state: PredicateState::default(),
            }),
        }
    }
}

impl<I, F> Clone for SkipWhileAsync<I, F>
where
    I: AsyncIterator + Clone,
    I::Item: Clone,
    F: PredicateFn<I::Item> + Clone,
    <F as PredicateFn<I::Item>>::Output: IntoFuture,
    <<F as PredicateFn<I::Item>>::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            state: self.state.clone(),
        }
    }
}

impl<I, F> AsyncIterator for SkipWhileAsync<I, F>
where
    I: AsyncIterator,
    F: PredicateFn<I::Item>,
    <F as PredicateFn<I::Item>>::Output: IntoFuture<Output = bool>,
{
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let OptionPinnedEntry::Some(mut state_slot) = this.state.pinned_entry() else { return iter.poll_next(cx) };
        let state = state_slot.get_pin_mut().project();
        let mut predicate_state = state.predicate_state;
        let f = state.f;

        loop {
            let mut fut_state = match predicate_state.as_mut().pin_project() {
                PredicateStateProject::Empty(empty_state) => match task::ready!(iter.as_mut().poll_next(cx)) {
                    None => break Poll::Ready(None),
                    Some(item) => {
                        let fut = f.call_mut((&item,)).into_future();

                        empty_state.set_future(item, fut)
                    }
                },
                PredicateStateProject::Future(fut_state) => fut_state,
            };

            let skip = task::ready!(fut_state.get_pinned_future().poll(cx));
            let item = fut_state.set_empty().0;

            if !skip {
                state_slot.set_none();

                break Poll::Ready(Some(item));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut candidate = self.iter.size_hint();

        if let Some(state) = &self.state {
            candidate.0 = 0;

            if state.predicate_state.get_future().is_some() {
                candidate.1 = candidate.1.and_then(|high| high.checked_add(1));
            }
        }

        candidate
    }
}

impl<I, F> FusedAsyncIterator for SkipWhileAsync<I, F>
where
    I: FusedAsyncIterator,
    F: PredicateFn<I::Item>,
    <F as PredicateFn<I::Item>>::Output: IntoFuture<Output = bool>,
    <<F as PredicateFn<I::Item>>::Output as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.state
            .as_ref()
            .and_then(|state| state.predicate_state.get_future())
            .map_or_else(|| self.iter.is_terminated(), FusedFuture::is_terminated)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::{future, stream, StreamExt};
    use std::vec::Vec;

    #[tokio::test]
    async fn test_skip_while_async() {
        let iter = stream::iter(0..10).slim_skip_while_async(|&x| future::ready(x < 5));

        assert_eq!(iter.collect::<Vec<_>>().await, [5, 6, 7, 8, 9]);
    }

    #[tokio::test]
    async fn test_skip_while_async_clone() {
        let iter = stream::iter(0..10).slim_skip_while_async(|&x| future::ready(x < 5));
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [5, 6, 7, 8, 9]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [5, 6, 7, 8, 9]);
    }
}
