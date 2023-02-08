use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;
use three_states::{StateAPinProject, StateBPinProject, StateCPinProject, ThreeStates, ThreeStatesPinProject};

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct State<T, Fut> {
        #[pin]
        inner: ThreeStates<(), (), (), T, Fut, ()>,
    }
}

impl<T, Fut> Default for State<T, Fut> {
    fn default() -> Self {
        Self {
            inner: ThreeStates::A {
                pinned: (),
                unpinned: (),
            },
        }
    }
}

impl<T, Fut> State<T, Fut> {
    fn get_future(&self) -> Option<&Fut> {
        match &self.inner {
            ThreeStates::C { pinned, .. } => Some(pinned),
            _ => None,
        }
    }

    fn pin_project(self: Pin<&mut Self>) -> StateProject<T, Fut> {
        match self.project().inner.pin_project() {
            ThreeStatesPinProject::A(project) => StateProject::Empty(EmptyState { inner: project }),
            ThreeStatesPinProject::B(project) => StateProject::Accumulate(AccumulateState { inner: project }),
            ThreeStatesPinProject::C(project) => StateProject::Future(FutureState { inner: project }),
        }
    }
}

pub struct EmptyState<'a, T, Fut> {
    inner: StateAPinProject<'a, (), (), (), T, Fut, ()>,
}

impl<'a, T, Fut> EmptyState<'a, T, Fut> {
    fn set_accumulate(self, acc: T) -> AccumulateState<'a, T, Fut> {
        AccumulateState {
            inner: self.inner.replace_state_b((), acc).0,
        }
    }

    fn set_future(self, fut: Fut) -> FutureState<'a, T, Fut> {
        FutureState {
            inner: self.inner.replace_state_c(fut, ()).0,
        }
    }
}

pub struct AccumulateState<'a, T, Fut> {
    inner: StateBPinProject<'a, (), (), (), T, Fut, ()>,
}

impl<'a, T, Fut> AccumulateState<'a, T, Fut> {
    fn set_empty(self) -> (EmptyState<'a, T, Fut>, T) {
        let (inner, item) = self.inner.replace_state_a((), ());

        (EmptyState { inner }, item)
    }
}

pub struct FutureState<'a, T, Fut> {
    inner: StateCPinProject<'a, (), (), (), T, Fut, ()>,
}

impl<'a, T, Fut> FutureState<'a, T, Fut> {
    fn get_pin_mut(&mut self) -> Pin<&mut Fut> {
        self.inner.get_project().pinned
    }

    fn set_accumulate(self, acc: T) -> AccumulateState<'a, T, Fut> {
        AccumulateState {
            inner: self.inner.replace_state_b((), acc).0,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub enum StateProject<'a, T, Fut> {
    Empty(EmptyState<'a, T, Fut>),
    Accumulate(AccumulateState<'a, T, Fut>),
    Future(FutureState<'a, T, Fut>),
}

pin_project_lite::pin_project! {
    pub struct ReduceAsync<I, F>
    where
        I: AsyncIterator,
        F: FnMut<(I::Item, I::Item)>,
        F: ?Sized,
        F::Output: IntoFuture,
    {
        #[pin]
        iter: I,
        #[pin]
        state: State<<F::Output as IntoFuture>::Output, <F::Output as IntoFuture>::IntoFuture>,
        f: F,
    }
}

impl<I, F> ReduceAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item, I::Item)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            iter,
            state: State::default(),
            f,
        }
    }
}

impl<I, F> Clone for ReduceAsync<I, F>
where
    I: AsyncIterator + Clone,
    I::Item: Clone,
    F: FnMut<(I::Item, I::Item)> + Clone,
    F::Output: IntoFuture<Output = I::Item>,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            state: self.state.clone(),
            f: self.f.clone(),
        }
    }
}

impl<I, F> Future for ReduceAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item, I::Item)> + ?Sized,
    F::Output: IntoFuture<Output = I::Item>,
{
    type Output = Option<I::Item>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let mut iter = this.iter;
        let mut state = this.state.pin_project();
        let f = this.f;

        loop {
            let mut fut = match state {
                StateProject::Empty(empty_state) => match task::ready!(iter.as_mut().poll_next(cx)) {
                    None => return Poll::Ready(None),
                    Some(item) => {
                        state = StateProject::Accumulate(empty_state.set_accumulate(item));

                        continue;
                    }
                },
                StateProject::Accumulate(acc_state) => match task::ready!(iter.as_mut().poll_next(cx)) {
                    None => return Poll::Ready(Some(acc_state.set_empty().1)),
                    Some(item) => {
                        let (empty_state, acc) = acc_state.set_empty();
                        let fut = f.call_mut((acc, item)).into_future();

                        empty_state.set_future(fut)
                    }
                },
                StateProject::Future(fut_state) => fut_state,
            };

            let acc = task::ready!(fut.get_pin_mut().poll(cx));

            state = StateProject::Accumulate(fut.set_accumulate(acc));
        }
    }
}

impl<I, F> FusedFuture for ReduceAsync<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut<(I::Item, I::Item)> + ?Sized,
    F::Output: IntoFuture<Output = I::Item>,
    <F::Output as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.state
            .get_future()
            .map_or_else(|| self.iter.is_terminated(), FusedFuture::is_terminated)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::future::Ready;
    use futures_util::{future, stream};

    fn add(lhs: u32, rhs: u32) -> Ready<u32> {
        future::ready(lhs + rhs)
    }

    #[tokio::test]
    async fn test_reduce_async() {
        let future = stream::iter([2, 3, 5]).slim_reduce_async(add);

        assert_eq!(future.await, Some(10));
    }

    #[tokio::test]
    async fn test_reduce_async_empty() {
        let future = stream::iter(None::<u32>).slim_reduce_async(add);

        assert_eq!(future.await, None);
    }

    #[tokio::test]
    async fn test_reduce_async_single() {
        let future = stream::iter(Some(2)).slim_reduce_async(add);

        assert_eq!(future.await, Some(2));
    }

    #[tokio::test]
    async fn test_reduce_async_clone() {
        let future = stream::iter([2, 3, 5]).slim_reduce_async(add);
        let future_2 = future.clone();

        assert_eq!(future.await, Some(10));
        assert_eq!(future_2.await, Some(10));
    }
}
