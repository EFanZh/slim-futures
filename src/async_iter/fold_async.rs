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
            state: FoldState::Accumulate { acc },
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
        let mut state_slot = this.state;
        let f = this.f;

        Poll::Ready(loop {
            let fut = match state_slot.as_mut().project() {
                FoldStateProject::Accumulate { acc } => match task::ready!(iter.as_mut().poll_next(cx)) {
                    None => break getter.call_mut((acc,)),
                    Some(item) => {
                        let fut = f.call_mut((getter.call_mut((acc,)), item)).into_future();

                        state_slot.as_mut().set_future(fut)
                    }
                },
                FoldStateProject::Future { fut } => fut,
            };

            let acc = task::ready!(fut.poll(cx));

            state_slot.set(FoldState::Accumulate { acc });
        })
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
        match &self.state {
            FoldState::Accumulate { .. } => self.iter.is_terminated(),
            FoldState::Future { fut } => fut.is_terminated(),
        }
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
