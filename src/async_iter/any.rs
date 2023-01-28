use crate::async_iter::try_fold::TryFold;
use crate::future::Map;
use crate::support::fns::ControlFlowIsBreakFn;
use crate::support::AsyncIterator;
use core::future::Future;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::CopyFn;
use fn_traits::FnMut;

#[derive(Clone)]
struct AnyFn<P> {
    predicate: P,
}

impl<T, P> FnMut<((), T)> for AnyFn<P>
where
    P: FnMut<(T,), Output = bool>,
{
    type Output = ControlFlow<()>;

    fn call_mut(&mut self, args: ((), T)) -> Self::Output {
        if self.predicate.call_mut((args.1,)) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

pin_project_lite::pin_project! {
    pub struct Any<I, P> {
        #[pin]
        inner: Map<TryFold<I, (), CopyFn, AnyFn<P>>, ControlFlowIsBreakFn>
    }
}

impl<I, P> Any<I, P> {
    pub(crate) fn new(iter: I, predicate: P) -> Self {
        Self {
            inner: Map::new(
                TryFold::new(iter, (), CopyFn::default(), AnyFn { predicate }),
                ControlFlowIsBreakFn::default(),
            ),
        }
    }
}

impl<I, P> Clone for Any<I, P>
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

impl<I, P> Future for Any<I, P>
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
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
    use futures_util::stream;

    fn greater_than_2(x: u32) -> bool {
        x > 2
    }

    fn equals_10(x: u32) -> bool {
        x == 10
    }

    #[tokio::test]
    async fn test_any() {
        let future = stream::iter([2, 3, 5]).slim_any(greater_than_2);

        assert!(future.await);
    }

    #[tokio::test]
    async fn test_any_fail() {
        let future = stream::iter([2, 3, 5]).slim_any(equals_10);

        assert!(!future.await);
    }

    #[tokio::test]
    async fn test_any_clone() {
        let future = stream::iter([2, 3, 5]).slim_any(greater_than_2);
        let future_2 = future.clone();

        assert!(future.await);
        assert!(future_2.await);
    }
}
