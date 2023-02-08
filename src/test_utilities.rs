use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;
use futures_util::FutureExt;

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

pub fn delayed<F>(fut: F) -> impl Future<Output = F::Output>
where
    F: Future,
{
    Yield::new(1).then(|()| fut)
}

#[cfg(test)]
mod tests {
    use super::Yield;
    use crate::future::{self, FutureExt};
    use futures_core::FusedFuture;
    use std::task::Poll;

    #[tokio::test]
    async fn test_yield() {
        let mut future = Yield::new(2);

        assert!(!future.is_terminated());
        assert_eq!(futures_util::poll!(future.by_ref()), Poll::Pending);
        assert!(!future.is_terminated());
        assert_eq!(futures_util::poll!(future.by_ref()), Poll::Pending);
        assert!(!future.is_terminated());
        assert_eq!(futures_util::poll!(future.by_ref()), Poll::Ready(()));
        assert!(future.is_terminated());
        assert_eq!(futures_util::poll!(future.by_ref()), Poll::Ready(()));
        assert!(future.is_terminated());
        assert_eq!(futures_util::poll!(future.by_ref()), Poll::Ready(()));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_delayed() {
        let mut future = super::delayed(future::ready_by_copy::<u32>(2));

        assert_eq!(futures_util::poll!(future.by_ref()), Poll::Pending);
        assert_eq!(futures_util::poll!(future.by_ref()), Poll::Ready(2));
    }
}
