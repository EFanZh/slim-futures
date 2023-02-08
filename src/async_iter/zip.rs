use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use three_states::{StateAProject, StateBProject, StateCProject, ThreeStates, ThreeStatesProject};

#[derive(Clone)]
struct State<A, B> {
    inner: ThreeStates<(), (), (), A, (), B>,
}

impl<A, B> State<A, B> {
    fn project(&mut self) -> StateProject<A, B> {
        match self.inner.project_mut() {
            ThreeStatesProject::A(inner) => StateProject::Empty(EmptyState { inner }),
            ThreeStatesProject::B(inner) => StateProject::Left(LeftState { inner }),
            ThreeStatesProject::C(inner) => StateProject::Right(RightState { inner }),
        }
    }
}

impl<A, B> Default for State<A, B> {
    fn default() -> Self {
        Self {
            inner: ThreeStates::A {
                pinned: (),
                unpinned: (),
            },
        }
    }
}

struct EmptyState<'a, A, B> {
    inner: StateAProject<'a, (), (), (), A, (), B>,
}

impl<'a, A, B> EmptyState<'a, A, B> {
    fn set_left(self, value: A) -> LeftState<'a, A, B> {
        LeftState {
            inner: self.inner.replace_state_b((), value).0,
        }
    }

    fn set_right(self, value: B) -> RightState<'a, A, B> {
        RightState {
            inner: self.inner.replace_state_c((), value).0,
        }
    }
}

struct LeftState<'a, A, B> {
    inner: StateBProject<'a, (), (), (), A, (), B>,
}

impl<'a, A, B> LeftState<'a, A, B> {
    fn set_empty(self) -> (EmptyState<'a, A, B>, A) {
        let (inner, value) = self.inner.replace_state_a((), ());

        (EmptyState { inner }, value)
    }
}

struct RightState<'a, A, B> {
    inner: StateCProject<'a, (), (), (), A, (), B>,
}

impl<'a, A, B> RightState<'a, A, B> {
    fn set_empty(self) -> (EmptyState<'a, A, B>, B) {
        let (inner, value) = self.inner.replace_state_a((), ());

        (EmptyState { inner }, value)
    }
}

enum StateProject<'a, A, B> {
    Empty(EmptyState<'a, A, B>),
    Left(LeftState<'a, A, B>),
    Right(RightState<'a, A, B>),
}

pin_project_lite::pin_project! {
    pub struct Zip<A, B>
    where
        A: AsyncIterator,
        B: AsyncIterator,
    {
        #[pin]
        left: A,
        #[pin]
        right: B,
        state: State<A::Item, B::Item>,
    }
}

impl<A, B> Zip<A, B>
where
    A: AsyncIterator,
    B: AsyncIterator,
{
    pub(crate) fn new(left: A, right: B) -> Self {
        Self {
            left,
            right,
            state: State::default(),
        }
    }
}

impl<A, B> Clone for Zip<A, B>
where
    A: AsyncIterator + Clone,
    B: AsyncIterator + Clone,
    A::Item: Clone,
    B::Item: Clone,
{
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
            state: self.state.clone(),
        }
    }
}

impl<A, B> AsyncIterator for Zip<A, B>
where
    A: AsyncIterator,
    B: AsyncIterator,
{
    type Item = (A::Item, B::Item);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut left = this.left;
        let mut right = this.right;
        let mut state = this.state.project();

        Poll::Ready(Some(loop {
            match state {
                StateProject::Empty(empty_state) => match left.as_mut().poll_next(cx) {
                    Poll::Ready(None) => return Poll::Ready(None),
                    Poll::Ready(Some(item)) => state = StateProject::Left(empty_state.set_left(item)),
                    Poll::Pending => match task::ready!(right.as_mut().poll_next(cx)) {
                        None => return Poll::Ready(None),
                        Some(item) => state = StateProject::Right(empty_state.set_right(item)),
                    },
                },
                StateProject::Left(left_state) => match task::ready!(right.poll_next(cx)) {
                    None => return Poll::Ready(None),
                    Some(right_item) => {
                        let left_item = left_state.set_empty().1;

                        break (left_item, right_item);
                    }
                },
                StateProject::Right(right_state) => match task::ready!(left.poll_next(cx)) {
                    None => return Poll::Ready(None),
                    Some(left_item) => {
                        let right_item = right_state.set_empty().1;

                        break (left_item, right_item);
                    }
                },
            };
        }))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (mut left_low, mut left_high) = self.left.size_hint();
        let (mut right_low, mut right_high) = self.right.size_hint();

        match self.state.inner {
            ThreeStates::A { .. } => {}
            ThreeStates::B { .. } => {
                left_low = left_low.saturating_add(1);
                left_high = left_high.and_then(|left_high| left_high.checked_add(1));
            }
            ThreeStates::C { .. } => {
                right_low = right_low.saturating_add(1);
                right_high = right_high.and_then(|right_high| right_high.checked_add(1));
            }
        }

        let low = left_low.min(right_low);

        let high = match (left_high, right_high) {
            (None, None) => None,
            (None, Some(right_high)) => Some(right_high),
            (Some(left_high), None) => Some(left_high),
            (Some(left_high), Some(right_high)) => Some(left_high.min(right_high)),
        };

        (low, high)
    }
}

impl<A, B> FusedAsyncIterator for Zip<A, B>
where
    A: FusedAsyncIterator,
    B: FusedAsyncIterator,
{
    fn is_terminated(&self) -> bool {
        match self.state.inner {
            ThreeStates::A { .. } => self.left.is_terminated() || self.right.is_terminated(),
            ThreeStates::B { .. } => self.right.is_terminated(),
            ThreeStates::C { .. } => self.left.is_terminated(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::{stream, StreamExt};
    use std::vec::Vec;

    #[tokio::test]
    async fn test_zip() {
        let iter_1 = stream::iter(0..3).slim_zip(stream::iter(10..20));
        let iter_2 = stream::iter(10..20).slim_zip(stream::iter(0..3));

        assert_eq!(iter_1.collect::<Vec<_>>().await, [(0, 10), (1, 11), (2, 12)]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [(10, 0), (11, 1), (12, 2)]);
    }

    #[tokio::test]
    async fn test_zip_clone() {
        let iter_1 = stream::iter(0..3).slim_zip(stream::iter(10..20));
        let iter_1_clone = iter_1.clone();
        let iter_2 = stream::iter(10..20).slim_zip(stream::iter(0..3));
        let iter_2_clone = iter_2.clone();

        assert_eq!(iter_1.collect::<Vec<_>>().await, [(0, 10), (1, 11), (2, 12)]);
        assert_eq!(iter_1_clone.collect::<Vec<_>>().await, [(0, 10), (1, 11), (2, 12)]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [(10, 0), (11, 1), (12, 2)]);
        assert_eq!(iter_2_clone.collect::<Vec<_>>().await, [(10, 0), (11, 1), (12, 2)]);
    }
}
