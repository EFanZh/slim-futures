use crate::future::into_try_future::IntoTryFuture;
use crate::future::ready::Ready;
use crate::support;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct OkFuture<T, E>
    where
        T: Copy,
    {
        #[pin]
        inner: IntoTryFuture<Ready<T>, E>
    }
}

impl<T, E> OkFuture<T, E>
where
    T: Copy,
{
    fn new(value: T) -> Self
    where
        T: Copy,
    {
        Self {
            inner: IntoTryFuture::new(Ready::new(value)),
        }
    }
}

impl<T, E> Clone for OkFuture<T, E>
where
    T: Clone + Copy,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T, E> Future for OkFuture<T, E>
where
    T: Copy,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

pub fn ok<T, E>(value: T) -> OkFuture<T, E>
where
    T: Copy,
{
    support::assert_future::<_, Result<T, E>>(OkFuture::new(value))
}

#[cfg(test)]
mod tests {
    use crate::future;

    #[tokio::test]
    async fn test_err() {
        assert_eq!(future::err::<u32, u32>(7).await, Err(7));
    }
}
