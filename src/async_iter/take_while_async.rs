use crate::support::states::{PredicateState, PredicateStateProject, PredicateStateReplace};
use crate::support::{AsyncIterator, PredicateFn};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{self, Context, Poll};

pin_project_lite::pin_project! {
    pub struct TakeWhileAsync<I, F>
    where
        I: AsyncIterator,
        F: PredicateFn<I::Item>,
        F: ?Sized,
        <F as PredicateFn<I::Item>>::Output: IntoFuture,
    {
        #[pin]
        iter: I,
        #[pin]
        state: PredicateState<I::Item, <<F as PredicateFn<I::Item>>::Output as IntoFuture>::IntoFuture>,
        f: F,
    }
}

impl<I, F> TakeWhileAsync<I, F>
where
    I: AsyncIterator,
    F: PredicateFn<I::Item>,
    <F as PredicateFn<I::Item>>::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            iter,
            state: PredicateState::Empty,
            f,
        }
    }
}

impl<I, F> Clone for TakeWhileAsync<I, F>
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
            f: self.f.clone(),
        }
    }
}

impl<I, F> AsyncIterator for TakeWhileAsync<I, F>
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
        let f = this.f;

        let fut = match state_slot.as_mut().project() {
            PredicateStateProject::Empty => match task::ready!(iter.as_mut().poll_next(cx)) {
                None => return Poll::Ready(None),
                Some(item) => {
                    let fut = f.call_mut((&item,)).into_future();

                    state_slot.as_mut().set_polling(item, fut)
                }
            },
            PredicateStateProject::Polling { fut, .. } => fut,
        };

        let take = task::ready!(fut.poll(cx));

        let PredicateStateReplace::Polling { item, .. } = state_slot.as_mut().project_replace(PredicateState::Empty) else { unreachable!() };

        if take {
            Poll::Ready(Some(item))
        } else {
            Poll::Ready(None)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut candidate = (0, self.iter.size_hint().1);

        if matches!(self.state, PredicateState::Polling { .. }) {
            candidate.1 = candidate.1.and_then(|high| high.checked_add(1));
        }

        candidate
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::{future, stream, StreamExt};
    use std::vec::Vec;

    #[tokio::test]
    async fn test_take_while_async() {
        let iter = stream::iter(0..10).slim_take_while_async(|&x| future::ready(x < 5));

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_take_while_async_clone() {
        let iter = stream::iter(0..10).slim_take_while_async(|&x| future::ready(x < 5));
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 1, 2, 3, 4]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [0, 1, 2, 3, 4]);
    }
}
