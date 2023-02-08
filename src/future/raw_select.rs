use crate::support;
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct RawSelect<Fut1, Fut2> {
        #[pin]
        fut_1: Fut1,
        #[pin]
        fut_2: Fut2,
    }
}

impl<Fut1, Fut2> RawSelect<Fut1, Fut2> {
    pub(crate) fn new(fut_1: Fut1, fut_2: Fut2) -> Self {
        Self { fut_1, fut_2 }
    }

    #[must_use]
    pub fn get_inner_pinned(self: Pin<&mut Self>) -> (Pin<&mut Fut1>, Pin<&mut Fut2>) {
        let this = self.project();

        (this.fut_1, this.fut_2)
    }
}

impl<Fut1, Fut2> Clone for RawSelect<Fut1, Fut2>
where
    Fut1: Clone,
    Fut2: Clone,
{
    fn clone(&self) -> Self {
        Self {
            fut_1: self.fut_1.clone(),
            fut_2: self.fut_2.clone(),
        }
    }
}

impl<Fut1, Fut2> Future for RawSelect<Fut1, Fut2>
where
    Fut1: Future,
    Fut2: Future<Output = Fut1::Output>,
{
    type Output = Fut1::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        match this.fut_1.poll(cx) {
            Poll::Ready(result) => Poll::Ready(result),
            Poll::Pending => this.fut_2.poll(cx),
        }
    }
}

impl<Fut1, Fut2> FusedFuture for RawSelect<Fut1, Fut2>
where
    Fut1: FusedFuture,
    Fut2: FusedFuture<Output = Fut1::Output>,
{
    fn is_terminated(&self) -> bool {
        self.fut_1.is_terminated() || self.fut_2.is_terminated()
    }
}

pub fn raw_select<Fut1, Fut2>(fut_1: Fut1, fut_2: Fut2) -> RawSelect<Fut1::IntoFuture, Fut2::IntoFuture>
where
    Fut1: IntoFuture,
    Fut2: IntoFuture<Output = Fut1::Output>,
{
    support::assert_future::<_, Fut1::Output>(RawSelect::new(fut_1.into_future(), fut_2.into_future()))
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use crate::future::ready;
    use crate::{future, test_utilities};
    use futures_core::FusedFuture;
    use futures_util::FutureExt as _;
    use std::mem;
    use std::num::NonZeroU32;

    #[tokio::test]
    async fn test_raw_select() {
        assert_eq!(
            super::raw_select(ready::ready_by_copy(2), ready::ready_by_copy(3)).await,
            2
        );

        assert_eq!(
            super::raw_select(
                test_utilities::delayed(ready::ready_by_copy(2)),
                ready::ready_by_copy(3)
            )
            .await,
            3,
        );
    }

    #[tokio::test]
    async fn test_raw_select_clone() {
        let future = super::raw_select(ready::ready_by_copy(2), ready::ready_by_copy(3));
        let future_2 = future.clone();

        assert_eq!(future.await, 2);
        assert_eq!(future_2.await, 2);
    }

    #[tokio::test]
    async fn test_raw_select_fused_future() {
        let pending = || futures_util::future::ready(());

        let terminated = || {
            let mut result = pending();

            (&mut result).now_or_never().unwrap();

            result
        };

        assert!(!super::raw_select(pending(), pending()).is_terminated());
        assert!(super::raw_select(pending(), terminated()).is_terminated());
        assert!(super::raw_select(terminated(), pending()).is_terminated());
        assert!(super::raw_select(terminated(), terminated()).is_terminated());
    }

    #[tokio::test]
    async fn test_raw_select_is_slim() {
        let make_base_future_1 = || future::lazy(|_| 2);
        let make_base_future_2 = || ready::ready_by_copy(NonZeroU32::new(3).unwrap()).slim_map(NonZeroU32::get);
        let base_future_1 = make_base_future_1();
        let base_future_2 = make_base_future_2();
        let future = super::raw_select(make_base_future_1(), make_base_future_2());

        assert_eq!(mem::size_of_val(&base_future_2), mem::size_of_val(&future));
        assert_eq!(base_future_1.await, 2);
        assert_eq!(base_future_2.await, 3);
        assert_eq!(future.await, 2);
    }
}
