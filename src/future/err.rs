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
    use std::mem;

    #[tokio::test]
    async fn test_err() {
        assert_eq!(future::err::<u32, u32>(7).await, Err(7));
    }

    #[tokio::test]
    async fn test_err_clone() {
        let future = future::err::<u32, u32>(7);
        let future_2 = future.clone();

        assert_eq!(future.await, Err(7));
        assert_eq!(future_2.await, Err(7));
    }

    #[tokio::test]
    async fn test_err_is_slim() {
        let value: u32 = 2;
        let future_1 = future::err::<u32, _>(value);
        let future_2 = futures_util::future::err::<u32, _>(value);

        assert_eq!(mem::size_of_val(&value), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(future_1.await, Err(value));
        assert_eq!(future_2.await, Err(value));
    }
}
