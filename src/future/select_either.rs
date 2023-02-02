use crate::future::map::Map;
use crate::future::raw_select::RawSelect;
use crate::support;
use crate::support::fns::{EitherLeftFn, EitherRightFn};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;
use futures_util::future::Either;

type LeftFuture<A, B> = Map<A, EitherLeftFn<<B as Future>::Output>>;
type RightFuture<A, B> = Map<B, EitherRightFn<<A as Future>::Output>>;

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct SelectEither<Fut1, Fut2>
    where
        Fut1: Future,
        Fut2: Future,
    {
        #[pin]
        inner: RawSelect<LeftFuture<Fut1, Fut2>, RightFuture<Fut1, Fut2>>
    }
}

impl<Fut1, Fut2> SelectEither<Fut1, Fut2>
where
    Fut1: Future,
    Fut2: Future,
{
    pub(crate) fn new(fut_1: Fut1, fut_2: Fut2) -> Self {
        Self {
            inner: RawSelect::new(
                Map::new(fut_1, EitherLeftFn::default()),
                Map::new(fut_2, EitherRightFn::default()),
            ),
        }
    }

    #[must_use]
    pub fn get_inner_pinned(self: Pin<&mut Self>) -> (Pin<&mut Fut1>, Pin<&mut Fut2>) {
        let (fut_1, fut_2) = self.project().inner.get_inner_pinned();

        (fut_1.get_inner_pinned(), fut_2.get_inner_pinned())
    }
}

impl<Fut1, Fut2> Future for SelectEither<Fut1, Fut2>
where
    Fut1: Future,
    Fut2: Future,
{
    type Output = Either<Fut1::Output, Fut2::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut1, Fut2> FusedFuture for SelectEither<Fut1, Fut2>
where
    Fut1: FusedFuture,
    Fut2: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

pub fn select_either<Fut1, Fut2>(fut_1: Fut1, fut_2: Fut2) -> SelectEither<Fut1::IntoFuture, Fut2::IntoFuture>
where
    Fut1: IntoFuture,
    Fut2: IntoFuture,
{
    support::assert_future::<_, Either<Fut1::Output, Fut2::Output>>(SelectEither::new(
        fut_1.into_future(),
        fut_2.into_future(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use crate::{future, test_utilities};
    use futures_core::FusedFuture;
    use futures_util::future::Either;
    use futures_util::FutureExt as _;
    use std::mem;
    use std::num::NonZeroU32;

    #[tokio::test]
    async fn test_select_either() {
        assert!(matches!(
            future::select_either(future::ready_by_copy(2), future::ready_by_copy(3)).await,
            Either::Left(2),
        ));

        assert!(matches!(
            future::select_either(
                test_utilities::delayed(future::ready_by_copy(2)),
                future::ready_by_copy(3)
            )
            .await,
            Either::Right(3),
        ));
    }

    #[tokio::test]
    async fn test_select_either_clone() {
        let future = future::select_either(future::ready_by_copy(2), future::ready_by_copy(3));
        let future_2 = future.clone();

        assert!(matches!(future.await, Either::Left(2)));
        assert!(matches!(future_2.await, Either::Left(2)));
    }

    #[tokio::test]
    async fn test_select_either_fused_future() {
        let pending = || futures_util::future::ready(());

        let terminated = || {
            let mut result = pending();

            (&mut result).now_or_never().unwrap();

            result
        };

        assert!(!future::select_either(pending(), pending()).is_terminated());
        assert!(future::select_either(pending(), terminated()).is_terminated());
        assert!(future::select_either(terminated(), pending()).is_terminated());
        assert!(future::select_either(terminated(), terminated()).is_terminated());
    }

    #[tokio::test]
    async fn test_select_either_is_slim() {
        let make_base_future_1 = || future::lazy(|_| 2);
        let make_base_future_2 = || future::ready_by_copy(NonZeroU32::new(3).unwrap()).slim_map(NonZeroU32::get);
        let base_future_1 = make_base_future_1();
        let base_future_2 = make_base_future_2();
        let future = future::select_either(make_base_future_1(), make_base_future_2());

        assert_eq!(mem::size_of_val(&base_future_2), mem::size_of_val(&future));
        assert_eq!(base_future_1.await, 2);
        assert_eq!(base_future_2.await, 3);
        assert!(matches!(future.await, Either::Left(2)));
    }
}
