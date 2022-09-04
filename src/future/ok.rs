use crate::future::into_try_future::IntoTryFuture;
use crate::future::ready::Ready;
use crate::support;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct Ok<T, E>
    where
        T: Copy,
    {
        #[pin]
        inner: IntoTryFuture<Ready<T>, E>
    }
}

impl<T, E> Ok<T, E>
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

impl<T, E> Clone for Ok<T, E>
where
    T: Clone + Copy,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T, E> Future for Ok<T, E>
where
    T: Copy,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

pub fn ok<T, E>(value: T) -> Ok<T, E>
where
    T: Copy,
{
    support::assert_future::<_, Result<T, E>>(Ok::new(value))
}

#[cfg(test)]
mod tests {
    use crate::future;
    use std::mem;

    #[tokio::test]
    async fn test_ok() {
        assert_eq!(future::ok::<u32, u32>(2).await, Ok(2));
    }

    #[tokio::test]
    async fn test_ok_clone() {
        let future = future::ok::<u32, u32>(2);
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(2));
        assert_eq!(future_2.await, Ok(2));
    }

    #[tokio::test]
    async fn test_ok_is_slim() {
        let value: u32 = 2;
        let future_1 = future::ok::<_, u32>(value);
        let future_2 = futures_util::future::ok::<_, u32>(value);

        assert_eq!(mem::size_of_val(&value), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(future_1.await, Ok(value));
        assert_eq!(future_2.await, Ok(value));
    }
}
