use crate::support::states::{FoldState, FoldStateProject};
use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

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
        state: FoldState<Option<<F::Output as IntoFuture>::Output>, <F::Output as IntoFuture>::IntoFuture>,
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
            state: FoldState::new(None),
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
        let mut state = this.state;
        let f = this.f;

        Poll::Ready('outer: loop {
            let mut fut_state = match state.as_mut().pin_project() {
                FoldStateProject::Accumulate(mut acc_state) => loop {
                    match task::ready!(iter.as_mut().poll_next(cx)) {
                        None => break 'outer acc_state.get_mut().take(),
                        Some(item) => match acc_state.get_mut().take() {
                            None => *acc_state.get_mut() = Some(item),
                            Some(acc) => {
                                let fut = f.call_mut((acc, item)).into_future();

                                break acc_state.set_future(fut);
                            }
                        },
                    }
                },
                FoldStateProject::Future(future_state) => future_state,
            };

            let acc = task::ready!(fut_state.get_pinned().poll(cx));

            fut_state.set_accumulate(Some(acc));
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
