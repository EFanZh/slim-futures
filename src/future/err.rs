use crate::future::{ready::Ready, Map};
use crate::support;
use crate::support::fns::ErrFn;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct ErrFuture<T, E> {
        #[pin]
        inner: Map<Ready<E>, ErrFn<T, E>>
    }
}

impl<T, E> ErrFuture<T, E> {
    fn new(error: E) -> Self {
        Self {
            inner: Map::new(Ready::new(error), ErrFn::default()),
        }
    }
}

impl<T, E> Clone for ErrFuture<T, E>
where
    E: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T, E> Future for ErrFuture<T, E>
where
    E: Copy,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

pub fn err<T, E>(error: E) -> ErrFuture<T, E>
where
    E: Copy,
{
    support::assert_future::<_, Result<T, E>>(ErrFuture::new(error))
}

#[cfg(test)]
mod tests {
    use crate::future;

    #[tokio::test]
    async fn test_err() {
        assert_eq!(future::err::<u32, u32>(7).await, Err(7));
    }
}
