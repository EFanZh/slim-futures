use crate::support::states::{FoldState, FoldStateProject};
use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct FoldAsync<I, T, G, F>
    where
        I: AsyncIterator,
        F: FnMut<(T, I::Item)>,
        F: ?Sized,
        F::Output: IntoFuture,
    {
        #[pin]
        iter: I,
        getter: G,
        #[pin]
        state: FoldState<T, <F::Output as IntoFuture>::IntoFuture>,
        f: F,
    }
}

impl<I, T, G, F> FoldAsync<I, T, G, F>
where
    I: AsyncIterator,
    F: FnMut<(T, I::Item)>,
    F::Output: IntoFuture<Output = T>,
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

impl<I, T, G, F> Clone for FoldAsync<I, T, G, F>
where
    I: AsyncIterator + Clone,
    T: Clone,
    G: Clone,
    F: FnMut<(T, I::Item)> + Clone,
    F::Output: IntoFuture<Output = T>,
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

impl<I, T, G, F> Future for FoldAsync<I, T, G, F>
where
    I: AsyncIterator,
    G: for<'a> FnMut<(&'a mut T,), Output = T>,
    F: FnMut<(T, I::Item)> + ?Sized,
    F::Output: IntoFuture<Output = T>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let mut iter = this.iter;
        let getter = this.getter;
        let mut state = this.state.pin_project();
        let f = this.f;

        loop {
            let mut fut = match state {
                FoldStateProject::Accumulate(mut acc_state) => match task::ready!(iter.as_mut().poll_next(cx)) {
                    None => break Poll::Ready(getter.call_mut((acc_state.get_mut(),))),
                    Some(item) => {
                        let fut = f
                            .call_mut((getter.call_mut((acc_state.get_mut(),)), item))
                            .into_future();

                        acc_state.set_future(fut)
                    }
                },
                FoldStateProject::Future(fut_state) => fut_state,
            };

            let acc = task::ready!(fut.get_pin_mut().poll(cx));

            state = FoldStateProject::Accumulate(fut.set_accumulate(acc));
        }
    }
}

impl<I, T, G, F> FusedFuture for FoldAsync<I, T, G, F>
where
    I: FusedAsyncIterator,
    G: for<'a> FnMut<(&'a mut T,), Output = T>,
    F: FnMut<(T, I::Item)> + ?Sized,
    F::Output: IntoFuture<Output = T>,
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
    use futures_util::future::{self, Ready};
    use futures_util::stream;

    fn accumulate(state: u64, item: u32) -> Ready<u64> {
        future::ready(state * u64::from(item))
    }

    #[tokio::test]
    async fn test_fold_async() {
        let future = stream::iter([2, 3, 5]).slim_fold_async_by_copy(1_u64, accumulate);

        assert_eq!(future.await, 30_u64);
    }

    #[tokio::test]
    async fn test_fold_async_clone() {
        let future = stream::iter([2, 3, 5]).slim_fold_async_by_copy(1_u64, accumulate);
        let future_2 = future.clone();

        assert_eq!(future.await, 30_u64);
        assert_eq!(future_2.await, 30_u64);
    }
}
