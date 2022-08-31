use crate::future::map::Map;
use crate::support::FnMut1;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
struct OrElseFn<F> {
    inner: F,
}

impl<T, E, F, U> FnMut1<Result<T, E>> for OrElseFn<F>
where
    F: FnMut1<E, Output = Result<T, U>>,
{
    type Output = Result<T, U>;

    fn call_mut(&mut self, arg: Result<T, E>) -> Self::Output {
        arg.or_else(|value| self.inner.call_mut(value))
    }
}

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct OrElse<Fut, F> {
        #[pin]
        inner: Map<Fut, OrElseFn<F>>,
    }
}

impl<Fut, F> OrElse<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Map::new(fut, OrElseFn { inner: f }),
        }
    }
}

impl<Fut, F, T, E, U> Future for OrElse<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<E, Output = Result<T, U>>,
{
    type Output = Result<T, U>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F, T, E, U> FusedFuture for OrElse<Fut, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnMut1<E, Output = Result<T, U>>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use futures_core::FusedFuture;
    use futures_util::future;

    #[tokio::test]
    async fn test_or_else() {
        assert_eq!(
            future::ready(Ok::<u32, u32>(2))
                .slim_or_else(|value| Ok::<u32, u32>(value + 3))
                .await,
            Ok(2)
        );

        assert_eq!(
            future::ready(Ok::<u32, u32>(2))
                .slim_or_else(|value| Err(value + 3))
                .await,
            Ok(2)
        );

        assert_eq!(
            future::ready(Err::<u32, u32>(2))
                .slim_or_else(|value| Ok::<u32, u32>(value + 3))
                .await,
            Ok(5)
        );

        assert_eq!(
            future::ready(Err::<u32, u32>(2))
                .slim_or_else(|value| Err(value + 3))
                .await,
            Err(5)
        );
    }

    #[tokio::test]
    async fn test_or_else_clone() {
        let future = future::ready(Err::<u32, u32>(2)).slim_or_else(|value| Err(value + 3));
        let future_2 = future.clone();

        assert_eq!(future.await, Err(5));
        assert_eq!(future_2.await, Err(5));
    }

    #[tokio::test]
    async fn test_or_else_fused_future() {
        let mut future = future::ready(Err::<u32, u32>(2)).slim_or_else(|value| Err(value + 3));

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, Err(5));
        assert!(future.is_terminated());
    }
}
