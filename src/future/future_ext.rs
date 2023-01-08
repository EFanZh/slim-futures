use crate::future::and_then::AndThen;
use crate::future::and_then_async::AndThenAsync;
use crate::future::err_into::ErrInto;
use crate::future::flatten::Flatten;
use crate::future::inspect::Inspect;
use crate::future::inspect_err::InspectErr;
use crate::future::inspect_ok::InspectOk;
use crate::future::into_result_future::IntoResultFuture;
use crate::future::map::Map;
use crate::future::map_async::MapAsync;
use crate::future::map_err::MapErr;
use crate::future::map_err_async::MapErrAsync;
use crate::future::map_into::MapInto;
use crate::future::map_ok::MapOk;
use crate::future::map_ok_async::MapOkAsync;
use crate::future::map_ok_or_else::MapOkOrElse;
use crate::future::map_ok_or_else_async::MapOkOrElseAsync;
use crate::future::ok_into::OkInto;
use crate::future::or_else::OrElse;
use crate::future::or_else_async::OrElseAsync;
use crate::future::raw_map_ok_or_else_async::RawMapOkOrElseAsync;
use crate::future::try_flatten::TryFlatten;
use crate::future::try_flatten_err::TryFlattenErr;
use crate::future::unwrap_or_else::UnwrapOrElse;
use crate::future::unwrap_or_else_async::UnwrapOrElseAsync;
use crate::support::{self, AsyncIterator, FromResidual, Never, ResultFuture, Try};
use core::future::Future;

pub trait FutureExt: Future {
    fn by_ref(&mut self) -> &mut Self {
        self
    }

    fn slim_and_then<F, R>(self, f: F) -> AndThen<Self, F>
    where
        Self: Sized,
        Self::Output: Try,
        F: FnMut(<Self::Output as Try>::Output) -> R,
        R: FromResidual<<Self::Output as Try>::Residual> + Try,
    {
        support::assert_future::<_, R>(AndThen::new(self, f))
    }

    fn slim_and_then_async<F, Fut2>(self, f: F) -> AndThenAsync<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Ok) -> Fut2,
        Fut2: ResultFuture<Error = Self::Error>,
    {
        support::assert_future::<_, Result<Fut2::Ok, Self::Error>>(AndThenAsync::new(self, f))
    }

    fn slim_err_into<U>(self) -> ErrInto<Self, U>
    where
        Self: ResultFuture + Sized,
        Self::Error: Into<U>,
    {
        support::assert_future::<_, Result<Self::Ok, U>>(ErrInto::new(self))
    }

    fn slim_flatten(self) -> Flatten<Self>
    where
        Self: Sized,
        Self::Output: Future,
    {
        support::assert_future::<_, <Self::Output as Future>::Output>(Flatten::new(self))
    }

    fn slim_flatten_async_iter(self) -> Flatten<Self>
    where
        Self: Sized,
        Self::Output: AsyncIterator,
    {
        support::assert_async_iter::<_, <Self::Output as AsyncIterator>::Item>(Flatten::new(self))
    }

    fn slim_inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Output),
    {
        support::assert_future::<_, Self::Output>(Inspect::new(self, f))
    }

    fn slim_inspect_err<F>(self, f: F) -> InspectErr<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(&Self::Error),
    {
        support::assert_future::<_, Self::Output>(InspectErr::new(self, f))
    }

    fn slim_inspect_ok<F>(self, f: F) -> InspectOk<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(&Self::Ok),
    {
        support::assert_future::<_, Self::Output>(InspectOk::new(self, f))
    }

    fn slim_into_result_future<E>(self) -> IntoResultFuture<Self, E>
    where
        Self: Sized,
    {
        support::assert_future::<_, Result<Self::Output, E>>(IntoResultFuture::new(self))
    }

    fn slim_map<F, U>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> U,
    {
        support::assert_future::<_, U>(Map::new(self, f))
    }

    fn slim_map_async<F, Fut2>(self, f: F) -> MapAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> Fut2,
        Fut2: Future,
    {
        support::assert_future::<_, Fut2::Output>(MapAsync::new(self, f))
    }

    fn slim_map_err<F, U>(self, f: F) -> MapErr<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Error) -> U,
    {
        support::assert_future::<_, Result<Self::Ok, U>>(MapErr::new(self, f))
    }

    fn slim_map_err_async<F, Fut2>(self, f: F) -> MapErrAsync<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Error) -> Fut2,
        Fut2: Future,
    {
        support::assert_future::<_, Result<Self::Ok, Fut2::Output>>(MapErrAsync::new(self, f))
    }

    fn slim_map_into<T>(self) -> MapInto<Self, T>
    where
        Self: Sized,
        Self::Output: Into<T>,
    {
        support::assert_future::<_, T>(MapInto::new(self))
    }

    fn slim_map_ok<F, U>(self, f: F) -> MapOk<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Ok) -> U,
    {
        support::assert_future::<_, Result<U, Self::Error>>(MapOk::new(self, f))
    }

    fn slim_map_ok_async<F, Fut2>(self, f: F) -> MapOkAsync<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Ok) -> Fut2,
        Fut2: Future,
    {
        support::assert_future::<_, Result<Fut2::Output, Self::Error>>(MapOkAsync::new(self, f))
    }

    fn slim_map_ok_or_else<D, F, U>(self, default: D, f: F) -> MapOkOrElse<Self, D, F>
    where
        Self: ResultFuture + Sized,
        D: FnMut(Self::Error) -> U,
        F: FnMut(Self::Ok) -> U,
    {
        support::assert_future::<_, U>(MapOkOrElse::new(self, default, f))
    }

    fn slim_map_ok_or_else_async<D, F, Fut1, Fut2>(self, default: D, f: F) -> MapOkOrElseAsync<Self, D, F>
    where
        Self: ResultFuture + Sized,
        D: FnMut(Self::Error) -> Fut1,
        F: FnMut(Self::Ok) -> Fut2,
        Fut1: Future,
        Fut2: Future<Output = Fut1::Output>,
    {
        support::assert_future::<_, Fut1::Output>(MapOkOrElseAsync::new(self, default, f))
    }

    fn slim_never_error(self) -> IntoResultFuture<Self, Never>
    where
        Self: Sized,
    {
        support::assert_future::<_, Result<Self::Output, Never>>(self.slim_into_result_future())
    }

    fn slim_ok_into<U>(self) -> OkInto<Self, U>
    where
        Self: ResultFuture + Sized,
        Self::Ok: Into<U>,
    {
        support::assert_future::<_, Result<U, Self::Error>>(OkInto::new(self))
    }

    fn slim_or_else<F, U>(self, f: F) -> OrElse<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Error) -> Result<Self::Ok, U>,
    {
        support::assert_future::<_, Result<Self::Ok, U>>(OrElse::new(self, f))
    }

    fn slim_or_else_async<F, Fut2>(self, f: F) -> OrElseAsync<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Error) -> Fut2,
        Fut2: ResultFuture<Ok = Self::Ok>,
    {
        support::assert_future::<_, Result<Self::Ok, Fut2::Error>>(OrElseAsync::new(self, f))
    }

    fn slim_raw_map_ok_or_else_async<D, F, Fut>(self, default: D, f: F) -> RawMapOkOrElseAsync<Self, D, F>
    where
        Self: ResultFuture + Sized,
        D: FnMut(Self::Error) -> Fut,
        F: FnMut(Self::Ok) -> Fut,
        Fut: Future,
    {
        support::assert_future::<_, Fut::Output>(RawMapOkOrElseAsync::new(self, default, f))
    }

    fn slim_try_flatten(self) -> TryFlatten<Self>
    where
        Self: Sized,
        Self::Output: Try,
        <Self::Output as Try>::Output: Future,
        <<Self::Output as Try>::Output as Future>::Output: FromResidual<<Self::Output as Try>::Residual> + Try,
    {
        support::assert_future::<_, <<Self::Output as Try>::Output as Future>::Output>(TryFlatten::new(self))
    }

    fn slim_try_flatten_err(self) -> TryFlattenErr<Self>
    where
        Self: ResultFuture + Sized,
        Self::Error: ResultFuture<Ok = Self::Ok> + Sized,
    {
        support::assert_future::<_, Result<Self::Ok, <Self::Error as ResultFuture>::Error>>(TryFlattenErr::new(self))
    }

    fn slim_unit_error(self) -> IntoResultFuture<Self, ()>
    where
        Self: Sized,
    {
        support::assert_future::<_, Result<Self::Output, ()>>(self.slim_into_result_future())
    }

    fn slim_unwrap_or_else<F>(self, f: F) -> UnwrapOrElse<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Error) -> Self::Ok,
    {
        support::assert_future::<_, Self::Ok>(UnwrapOrElse::new(self, f))
    }

    fn slim_unwrap_or_else_async<F, Fut2>(self, f: F) -> UnwrapOrElseAsync<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Error) -> Fut2,
        Fut2: Future<Output = Self::Ok>,
    {
        support::assert_future::<_, Self::Ok>(UnwrapOrElseAsync::new(self, f))
    }
}

impl<Fut> FutureExt for Fut where Fut: Future + ?Sized {}
