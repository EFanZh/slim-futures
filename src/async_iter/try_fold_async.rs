use crate::support::states::{FoldState, FoldStateProject};
use crate::support::{AsyncIterator, FromResidual, Try};
use core::future::{Future, IntoFuture};
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct TryFoldAsync<I, T, G, F>
    where
        I: AsyncIterator,
        F: FnMut<(T, I::Item)>,
        F: ?Sized,
        F::Output: IntoFuture,
        <F::Output as IntoFuture>::Output: Try,
    {
        #[pin]
        iter: I,
        getter: G,
        #[pin]
        state: FoldState<T, <F::Output as IntoFuture>::IntoFuture>,
        f: F,
    }
}

impl<I, T, G, F> TryFoldAsync<I, T, G, F>
where
    I: AsyncIterator,
    F: FnMut<(T, I::Item)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: Try<Output = T>,
{
    pub(crate) fn new(iter: I, acc: T, getter: G, f: F) -> Self {
        Self {
            iter,
            getter,
            state: FoldState::new(acc),
            f,
        }
    }
}

impl<I, T, G, F> Clone for TryFoldAsync<I, T, G, F>
where
    I: AsyncIterator + Clone,
    T: Clone,
    G: Clone,
    F: FnMut<(T, I::Item)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: Try<Output = T>,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            getter: self.getter.clone(),
            state: self.state.clone(),
            f: self.f.clone(),
        }
    }
}

impl<I, T, G, F> Future for TryFoldAsync<I, T, G, F>
where
    I: AsyncIterator,
    G: for<'a> FnMut<(&'a mut T,), Output = T>,
    F: FnMut<(T, I::Item)> + ?Sized,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: Try<Output = T>,
{
    type Output = <F::Output as IntoFuture>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let mut iter = this.iter;
        let getter = this.getter;
        let mut state = this.state;
        let f = this.f;

        Poll::Ready(loop {
            let mut fut_state = match state.as_mut().pin_project() {
                FoldStateProject::Accumulate(mut acc_state) => match task::ready!(iter.as_mut().poll_next(cx)) {
                    None => break Self::Output::from_output(getter.call_mut((acc_state.get_mut(),))),
                    Some(item) => {
                        let fut = f
                            .call_mut((getter.call_mut((acc_state.get_mut(),)), item))
                            .into_future();

                        acc_state.set_future(fut)
                    }
                },
                FoldStateProject::Future(fut_state) => fut_state,
            };

            match task::ready!(fut_state.get_pinned().poll(cx)).branch() {
                ControlFlow::Continue(acc) => {
                    fut_state.set_accumulate(acc);
                }
                ControlFlow::Break(residual) => break Self::Output::from_residual(residual),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::future::{self, Ready};
    use futures_util::stream;
    use std::convert::Infallible;

    fn accumulate(state: u64, item: u32) -> Ready<Result<u64, Infallible>> {
        future::ready(Ok(state * u64::from(item)))
    }

    #[tokio::test]
    async fn test_try_fold_async() {
        let future = stream::iter([2, 3, 5]).slim_try_fold_async_by_copy(1_u64, accumulate);

        assert_eq!(future.await, Ok(30_u64));
    }

    #[tokio::test]
    async fn test_try_fold_async_with_option() {
        let future = stream::iter([2, 3, 5])
            .slim_try_fold_async_by_copy(1_u64, |state, item: u32| future::ready(Some(state * u64::from(item))));

        assert_eq!(future.await, Some(30_u64));
    }

    #[tokio::test]
    async fn test_try_fold_async_error() {
        let mut counter = 0;

        let future = stream::iter([2, 3, 5]).slim_try_fold_async_by_copy(1_u64, |state, item: u32| {
            if counter < 2 {
                counter += 1;

                future::ok(state * u64::from(item))
            } else {
                future::err(7)
            }
        });

        assert_eq!(future.await, Err(7));
        assert_eq!(counter, 2);
    }

    #[tokio::test]
    async fn test_try_fold_async_clone() {
        let future = stream::iter([2, 3, 5]).slim_try_fold_async_by_copy(1_u64, accumulate);
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(30_u64));
        assert_eq!(future_2.await, Ok(30_u64));
    }
}
