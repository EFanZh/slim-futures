use crate::support;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
pub struct Ready<T> {
    value: T,
}

impl<T> Ready<T> {
    pub(crate) fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> Unpin for Ready<T> {}

impl<T> Future for Ready<T>
where
    T: Copy,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(self.value)
    }
}

pub fn ready<T>(value: T) -> Ready<T>
where
    T: Copy,
{
    support::assert_future::<_, T>(Ready::new(value))
}

#[cfg(test)]
mod tests {
    use crate::future;
    use std::mem;

    #[tokio::test]
    async fn test_ready() {
        assert_eq!(future::ready::<u32>(2).await, 2);
    }

    #[tokio::test]
    async fn test_ready_clone() {
        let future = future::ready::<u32>(2);
        let future_2 = future.clone();

        assert_eq!(future.await, 2);
        assert_eq!(future_2.await, 2);
    }

    #[tokio::test]
    async fn test_ready_is_slim() {
        let value: u32 = 2;
        let future_1 = future::ready(value);
        let future_2 = futures_util::future::ready(value);

        assert_eq!(mem::size_of_val(&value), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(future_1.await, 2);
        assert_eq!(future_2.await, 2);
    }
}
