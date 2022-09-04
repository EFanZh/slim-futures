use crate::future::map::Map;
use crate::support::FnMut1;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
struct UnwrapOrElseFn<F> {
    inner: F,
}

impl<T, E, F> FnMut1<Result<T, E>> for UnwrapOrElseFn<F>
where
    F: FnMut1<E, Output = T>,
{
    type Output = T;

    fn call_mut(&mut self, arg: Result<T, E>) -> Self::Output {
        arg.unwrap_or_else(|error| self.inner.call_mut(error))
    }
}

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct UnwrapOrElse<Fut, F> {
        #[pin]
        inner: Map<Fut, UnwrapOrElseFn<F>>,
    }
}

impl<Fut, F> UnwrapOrElse<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Map::new(fut, UnwrapOrElseFn { inner: f }),
        }
    }
}

impl<Fut, F, T, E> Future for UnwrapOrElse<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<E, Output = T>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F, T, E> FusedFuture for UnwrapOrElse<Fut, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnMut1<E, Output = T>,
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
    async fn test_unwrap_or_else() {
        assert_eq!(
            future::ok::<u32, u32>(2).slim_unwrap_or_else(|value| value + 3).await,
            2,
        );

        assert_eq!(
            future::err::<u32, u32>(2).slim_unwrap_or_else(|value| value + 3).await,
            5,
        );
    }

    #[tokio::test]
    async fn test_unwrap_or_else_clone() {
        let future = future::err::<u32, u32>(2).slim_unwrap_or_else(|value| value + 3);
        let future_2 = future.clone();

        assert_eq!(future.await, 5);
        assert_eq!(future_2.await, 5);
    }

    #[tokio::test]
    async fn test_unwrap_or_else_fused_future() {
        let mut future = future::err::<u32, u32>(2).slim_unwrap_or_else(|value| value + 3);

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, 5);
        assert!(future.is_terminated());
    }
}
