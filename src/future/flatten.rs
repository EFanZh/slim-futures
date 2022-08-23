use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    #[project = FlattenInnerProject]
    enum FlattenInner<Fut>
    where
        Fut: Future,
    {
        First {
            #[pin]
            fut: Fut,
        },
        Second {
            #[pin]
            fut: Fut::Output,
        },
    }
}

pin_project_lite::pin_project! {
    pub struct Flatten<Fut>
    where
        Fut: Future,
    {
        #[pin]
        inner: FlattenInner<Fut>,
    }
}

impl<Fut> Flatten<Fut>
where
    Fut: Future,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: FlattenInner::First { fut },
        }
    }
}

impl<Fut> Clone for Flatten<Fut>
where
    Fut: Future + Clone,
    Fut::Output: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: match &self.inner {
                FlattenInner::First { fut } => FlattenInner::First { fut: fut.clone() },
                FlattenInner::Second { fut } => FlattenInner::Second { fut: fut.clone() },
            },
        }
    }
}

impl<Fut> Future for Flatten<Fut>
where
    Fut: Future,
    Fut::Output: Future,
{
    type Output = <Fut::Output as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = self.project().inner;

        if let FlattenInnerProject::First { fut } = inner.as_mut().project() {
            let fut = futures_core::ready!(fut.poll(cx));

            inner.set(FlattenInner::Second { fut });
        }

        if let FlattenInnerProject::Second { fut } = inner.project() {
            fut.poll(cx)
        } else {
            unreachable!() // TODO: Is `unreachable_unchecked()` necessary for compiler to optimize away this branch?
        }
    }
}

impl<Fut> FusedFuture for Flatten<Fut>
where
    Fut: FusedFuture,
    Fut::Output: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        match &self.inner {
            FlattenInner::First { fut } => fut.is_terminated(),
            FlattenInner::Second { fut } => fut.is_terminated(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Flatten;
    use crate::future::future_ext::FutureExt;
    use crate::test_utilities::Defer;
    use futures_core::FusedFuture;
    use futures_util::future;
    use std::task::Poll;

    #[tokio::test]
    async fn test_flatten_future() {
        let original = future::ready(future::ready(7));
        let wrapped = Flatten::new(original.clone());

        assert_eq!(original.await.await, 7);
        assert_eq!(wrapped.await, 7);
    }

    #[tokio::test]
    async fn test_flatten_future_first_pending() {
        let original = Defer::new(1).slim_map(|()| future::ready(7));
        let wrapped = Flatten::new(original.clone());

        assert_eq!(original.await.await, 7);
        assert_eq!(wrapped.await, 7);
    }

    #[tokio::test]
    async fn test_flatten_clone() {
        let mut wrapped = Flatten::new(future::ready(Defer::new(1)));

        assert_eq!(futures_util::poll!(wrapped.clone()), Poll::Pending);

        assert!(futures_util::poll!(&mut wrapped).is_pending());

        assert_eq!(futures_util::poll!(wrapped.clone()), Poll::Ready(()));
    }

    #[tokio::test]
    async fn test_flatten_fused_future() {
        let mut wrapped = Flatten::new(future::ready(Defer::new(1)));

        assert!(!wrapped.is_terminated());

        assert!(futures_util::poll!(&mut wrapped).is_pending());

        assert!(!wrapped.is_terminated());

        assert_eq!(futures_util::poll!(&mut wrapped), Poll::Ready(()));

        assert!(wrapped.is_terminated());
    }
}
