use crate::support::states::{PredicateState, PredicateStateProject, PredicateStateReplace};
use crate::support::{AsyncIterator, FusedAsyncIterator, PredicateFn};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    #[derive(Clone)]
    struct State<F, T, Fut> {
        #[pin]
        polling_state: PredicateState<T, Fut>,
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
                polling_state: PredicateState::Empty,
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
        let mut state_slot = this.state;
        let Some(state) = state_slot.as_mut().as_pin_mut().map(State::project) else { return iter.poll_next(cx) };
        let mut fut_slot = state.polling_state;
        let f = state.f;

        loop {
            let fut = match fut_slot.as_mut().project() {
                PredicateStateProject::Empty => match task::ready!(iter.as_mut().poll_next(cx)) {
                    None => break Poll::Ready(None),
                    Some(item) => {
                        let fut = f.call_mut((&item,)).into_future();

                        fut_slot.as_mut().set_polling(item, fut)
                    }
                },
                PredicateStateProject::Polling { fut, .. } => fut,
            };

            let skip = task::ready!(fut.poll(cx));

            let item = match fut_slot.as_mut().project_replace(PredicateState::Empty) {
                PredicateStateReplace::Empty => unreachable!(),
                PredicateStateReplace::Polling { item, .. } => item,
            };

            if !skip {
                state_slot.set(None);

                break Poll::Ready(Some(item));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut candidate = self.iter.size_hint();

        if let Some(state) = &self.state {
            candidate.0 = 0;

            if matches!(state.polling_state, PredicateState::Polling { .. }) {
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
        if let Some(State {
            polling_state: PredicateState::Polling { fut, .. },
            ..
        }) = &self.state
        {
            fut.is_terminated()
        } else {
            self.iter.is_terminated()
        }
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
