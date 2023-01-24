use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    #[project = StateProject]
    #[project_replace = StateProjectReplace]
    #[derive(Clone)]
    enum State<Fut>
    where
        Fut: Future,
    {
        Empty,
        Single {
            acc: Fut::Output,
        },
        Future {
            #[pin]
            fut: Fut,
        },
    }
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
        state: State<<F::Output as IntoFuture>::IntoFuture>,
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
            state: State::Empty,
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
        let mut state_slot = this.state;
        let f = this.f;

        Poll::Ready(loop {
            if let StateProject::Future { fut } = state_slot.as_mut().project() {
                let acc = task::ready!(fut.poll(cx));

                state_slot.set(State::Single { acc });
            }

            if let Some(item) = task::ready!(iter.as_mut().poll_next(cx)) {
                let state = match state_slot.as_ref().get_ref() {
                    State::Empty => State::Single { acc: item },
                    State::Single { .. } => {
                        let acc = match state_slot.as_mut().project_replace(State::Empty) {
                            StateProjectReplace::Single { acc } => acc,
                            _ => unreachable!(),
                        };

                        State::Future {
                            fut: f.call_mut((acc, item)).into_future(),
                        }
                    }
                    State::Future { .. } => unreachable!(),
                };

                state_slot.set(state);
            } else {
                break match state_slot.as_mut().project_replace(State::Empty) {
                    StateProjectReplace::Empty => None,
                    StateProjectReplace::Single { acc } => Some(acc),
                    StateProjectReplace::Future { .. } => unreachable!(),
                };
            }
        })
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
        if let State::Future { fut } = &self.state {
            fut.is_terminated()
        } else {
            self.iter.is_terminated()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
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
    async fn test_reduce_async_clone() {
        let future = stream::iter([2, 3, 5]).slim_reduce_async(add);
        let future_2 = future.clone();

        assert_eq!(future.await, Some(10));
        assert_eq!(future_2.await, Some(10));
    }
}
