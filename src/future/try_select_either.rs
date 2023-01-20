use crate::future::map_ok_or_else::MapOkOrElse;
use crate::future::raw_select::RawSelect;
use crate::support::fns::{ComposeFn, EitherLeftFn, EitherRightFn, ErrFn, OkFn};
use crate::support::{self, IntoResultFuture, ResultFuture};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;
use futures_util::future::Either;

type OkEitherLeftFn<Fut2> = EitherLeftFn<<Fut2 as ResultFuture>::Ok>;
type OkEitherRightFn<Fut1> = EitherRightFn<<Fut1 as ResultFuture>::Ok>;
type ErrorEitherLeftFn<Fut2> = EitherLeftFn<<Fut2 as ResultFuture>::Error>;
type ErrorEitherRightFn<Fut1> = EitherRightFn<<Fut1 as ResultFuture>::Error>;

type OkEither<Fut1, Fut2> = Either<<Fut1 as ResultFuture>::Ok, <Fut2 as ResultFuture>::Ok>;
type ErrorEither<Fut1, Fut2> = Either<<Fut1 as ResultFuture>::Error, <Fut2 as ResultFuture>::Error>;

type OkEitherFn<Fut1, Fut2> = OkFn<ErrorEither<Fut1, Fut2>>;
type ErrorEitherFn<Fut1, Fut2> = ErrFn<OkEither<Fut1, Fut2>>;

type OkLeftFn<Fut1, Fut2> = ComposeFn<OkEitherLeftFn<Fut2>, OkEitherFn<Fut1, Fut2>>;
type OkRightFn<Fut1, Fut2> = ComposeFn<OkEitherRightFn<Fut1>, OkEitherFn<Fut1, Fut2>>;
type ErrorLeftFn<Fut1, Fut2> = ComposeFn<ErrorEitherLeftFn<Fut2>, ErrorEitherFn<Fut1, Fut2>>;
type ErrorRightFn<Fut1, Fut2> = ComposeFn<ErrorEitherRightFn<Fut1>, ErrorEitherFn<Fut1, Fut2>>;

type LeftFuture<Fut1, Fut2> = MapOkOrElse<Fut1, ErrorLeftFn<Fut1, Fut2>, OkLeftFn<Fut1, Fut2>>;
type RightFuture<Fut1, Fut2> = MapOkOrElse<Fut2, ErrorRightFn<Fut1, Fut2>, OkRightFn<Fut1, Fut2>>;

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct TrySelectEither<Fut1, Fut2>
    where
        Fut1: ResultFuture,
        Fut2: ResultFuture,
    {
        #[pin]
        inner: RawSelect<LeftFuture<Fut1, Fut2>, RightFuture<Fut1, Fut2>>
    }
}

impl<Fut1, Fut2> TrySelectEither<Fut1, Fut2>
where
    Fut1: ResultFuture,
    Fut2: ResultFuture,
{
    pub(crate) fn new(fut_1: Fut1, fut_2: Fut2) -> Self {
        Self {
            inner: RawSelect::new(
                MapOkOrElse::new(
                    fut_1,
                    ComposeFn::new(EitherLeftFn::default(), ErrFn::default()),
                    ComposeFn::new(EitherLeftFn::default(), OkFn::default()),
                ),
                MapOkOrElse::new(
                    fut_2,
                    ComposeFn::new(EitherRightFn::default(), ErrFn::default()),
                    ComposeFn::new(EitherRightFn::default(), OkFn::default()),
                ),
            ),
        }
    }
}

impl<Fut1, Fut2> Future for TrySelectEither<Fut1, Fut2>
where
    Fut1: ResultFuture,
    Fut2: ResultFuture,
{
    type Output = Result<Either<Fut1::Ok, Fut2::Ok>, Either<Fut1::Error, Fut2::Error>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut1, Fut2> FusedFuture for TrySelectEither<Fut1, Fut2>
where
    Fut1: ResultFuture + FusedFuture,
    Fut2: ResultFuture + FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

pub fn try_select_either<Fut1, Fut2>(fut_1: Fut1, fut_2: Fut2) -> TrySelectEither<Fut1::IntoFuture, Fut2::IntoFuture>
where
    Fut1: IntoResultFuture,
    Fut2: IntoResultFuture,
{
    support::assert_future::<_, Result<Either<Fut1::Ok, Fut2::Ok>, Either<Fut1::Error, Fut2::Error>>>(
        TrySelectEither::new(fut_1.into_future(), fut_2.into_future()),
    )
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
    async fn test_try_select_either() {
        let ok_2 = || future::ok::<u32, u32>(2);
        let ok_3 = || future::ok::<u32, u32>(3);
        let err_2 = || future::err::<u32, u32>(2);
        let err_3 = || future::err::<u32, u32>(3);

        assert!(matches!(
            future::try_select_either(ok_2(), ok_3()).await,
            Ok(Either::Left(2)),
        ));

        assert!(matches!(
            future::try_select_either(ok_2(), err_3()).await,
            Ok(Either::Left(2)),
        ));

        assert!(matches!(
            future::try_select_either(err_2(), ok_3()).await,
            Err(Either::Left(2)),
        ));

        assert!(matches!(
            future::try_select_either(err_2(), err_3()).await,
            Err(Either::Left(2)),
        ));

        assert!(matches!(
            future::try_select_either(test_utilities::delayed(ok_2()), ok_3()).await,
            Ok(Either::Right(3)),
        ));

        assert!(matches!(
            future::try_select_either(test_utilities::delayed(ok_2()), err_3()).await,
            Err(Either::Right(3)),
        ));

        assert!(matches!(
            future::try_select_either(test_utilities::delayed(err_2()), ok_3()).await,
            Ok(Either::Right(3)),
        ));

        assert!(matches!(
            future::try_select_either(test_utilities::delayed(err_2()), err_3()).await,
            Err(Either::Right(3)),
        ));
    }

    #[tokio::test]
    async fn test_try_select_either_clone() {
        let future = future::try_select_either(future::ok::<u32, u32>(2), future::ok::<u32, u32>(3));
        let future_2 = future.clone();

        assert!(matches!(future.await, Ok(Either::Left(2))));
        assert!(matches!(future_2.await, Ok(Either::Left(2))));
    }

    #[tokio::test]
    async fn test_try_select_either_fused_future() {
        let pending = || futures_util::future::ok::<(), ()>(());

        let terminated = || {
            let mut result = pending();

            (&mut result).now_or_never().unwrap().unwrap();

            result
        };

        assert!(!future::try_select_either(pending(), pending()).is_terminated());
        assert!(future::try_select_either(pending(), terminated()).is_terminated());
        assert!(future::try_select_either(terminated(), pending()).is_terminated());
        assert!(future::try_select_either(terminated(), terminated()).is_terminated());
    }

    #[tokio::test]
    async fn test_try_select_either_is_slim() {
        let make_base_future_1 = || future::lazy(|_| Ok::<u32, u32>(2));
        let make_base_future_2 = || future::ok::<_, u32>(NonZeroU32::new(3).unwrap()).slim_map_ok(NonZeroU32::get);
        let base_future_1 = make_base_future_1();
        let base_future_2 = make_base_future_2();
        let future = future::try_select_either(make_base_future_1(), make_base_future_2());

        assert_eq!(mem::size_of_val(&base_future_2), mem::size_of_val(&future));
        assert_eq!(base_future_1.await, Ok(2));
        assert_eq!(base_future_2.await, Ok(3));
        assert!(matches!(future.await, Ok(Either::Left(2))));
    }
}
