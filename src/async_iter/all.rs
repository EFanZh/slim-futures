use crate::async_iter::try_fold::TryFold;
use crate::future::Map;
use crate::support::fns::ControlFlowIsContinueFn;
use crate::support::AsyncIterator;
use core::future::Future;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::CopyFn;
use fn_traits::FnMut;

#[derive(Clone)]
struct AllFn<P> {
    predicate: P,
}

impl<T, P> FnMut<((), T)> for AllFn<P>
where
    P: FnMut<(T,), Output = bool>,
{
    type Output = ControlFlow<()>;

    fn call_mut(&mut self, args: ((), T)) -> Self::Output {
        if self.predicate.call_mut((args.1,)) {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

pin_project_lite::pin_project! {
    pub struct All<I, P> {
        #[pin]
        inner: Map<TryFold<I, (), CopyFn, AllFn<P>>, ControlFlowIsContinueFn>
    }
}

impl<I, P> All<I, P> {
    pub(crate) fn new(iter: I, predicate: P) -> Self {
        Self {
            inner: Map::new(
                TryFold::new(iter, (), CopyFn::default(), AllFn { predicate }),
                ControlFlowIsContinueFn::default(),
            ),
        }
    }
}

impl<I, P> Clone for All<I, P>
where
    I: Clone,
    P: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, P> Future for All<I, P>
where
    I: AsyncIterator,
    P: FnMut<(I::Item,), Output = bool>,
{
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::stream;

    fn less_than_10(x: u32) -> bool {
        x < 10
    }

    fn equals_2(x: u32) -> bool {
        x == 2
    }

    #[tokio::test]
    async fn test_all() {
        let future = stream::iter([2, 3, 5]).slim_all(less_than_10);

        assert!(future.await);
    }

    #[tokio::test]
    async fn test_all_fail() {
        let future = stream::iter([2, 3, 5]).slim_all(equals_2);

        assert!(!future.await);
    }

    #[tokio::test]
    async fn test_all_clone() {
        let future = stream::iter([2, 3, 5]).slim_all(less_than_10);
        let future_2 = future.clone();

        assert!(future.await);
        assert!(future_2.await);
    }
}
