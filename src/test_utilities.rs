use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
pub struct Yield {
    count: Option<usize>,
}

impl Yield {
    pub fn new(count: usize) -> Self {
        Self { count: Some(count) }
    }
}

impl Future for Yield {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if let Some(count) = &mut self.count {
            if *count == 0 {
                self.count = None;

                Poll::Ready(())
            } else {
                *count -= 1;

                cx.waker().wake_by_ref();

                Poll::Pending
            }
        } else {
            Poll::Ready(())
        }
    }
}

impl FusedFuture for Yield {
    fn is_terminated(&self) -> bool {
        self.count.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::Yield;
    use futures_core::FusedFuture;
    use std::task::Poll;

    #[tokio::test]
    async fn test_yield() {
        let mut future = Yield::new(2);

        assert!(!future.is_terminated());
        assert_eq!(futures_util::poll!(&mut future), Poll::Pending);
        assert!(!future.is_terminated());
        assert_eq!(futures_util::poll!(&mut future), Poll::Pending);
        assert!(!future.is_terminated());
        assert_eq!(futures_util::poll!(&mut future), Poll::Ready(()));
        assert!(future.is_terminated());
        assert_eq!(futures_util::poll!(&mut future), Poll::Ready(()));
        assert!(future.is_terminated());
        assert_eq!(futures_util::poll!(&mut future), Poll::Ready(()));
        assert!(future.is_terminated());
    }
}
