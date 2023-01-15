use crate::future::and_then::AndThen;
use crate::future::and_then_async::AndThenAsync;
use crate::future::err_into::ErrInto;
use crate::future::flatten::Flatten;
use crate::future::flatten_async_iterator::FlattenAsyncIterator;
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
use crate::support::{self, FromResidual, IntoAsyncIterator, Never, ResultFuture, Try};
use core::future::{Future, IntoFuture};

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

    fn slim_and_then_async<F, Fut>(self, f: F) -> AndThenAsync<Self, F>
    where
        Self: Sized,
        Self::Output: Try,
        F: FnMut(<Self::Output as Try>::Output) -> Fut,
        Fut: IntoFuture,
        Fut::Output: FromResidual<<Self::Output as Try>::Residual> + Try,
    {
        support::assert_future::<_, Fut::Output>(AndThenAsync::new(self, f))
    }

    fn slim_err_into<T>(self) -> ErrInto<Self, T>
    where
        Self: ResultFuture + Sized,
        Self::Error: Into<T>,
    {
        support::assert_future::<_, Result<Self::Ok, T>>(ErrInto::new(self))
    }

    fn slim_flatten(self) -> Flatten<Self>
    where
        Self: Sized,
        Self::Output: IntoFuture,
    {
        support::assert_future::<_, <Self::Output as IntoFuture>::Output>(Flatten::new(self))
    }

    fn slim_flatten_async_iterator(self) -> FlattenAsyncIterator<Self>
    where
        Self: Sized,
        Self::Output: IntoAsyncIterator,
    {
        support::assert_async_iter::<_, <Self::Output as IntoAsyncIterator>::Item>(FlattenAsyncIterator::new(self))
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

    fn slim_map<F, T>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> T,
    {
        support::assert_future::<_, T>(Map::new(self, f))
    }

    fn slim_map_async<F, Fut>(self, f: F) -> MapAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> Fut,
        Fut: IntoFuture,
    {
        support::assert_future::<_, Fut::Output>(MapAsync::new(self, f))
    }

    fn slim_map_err<F, T>(self, f: F) -> MapErr<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Error) -> T,
    {
        support::assert_future::<_, Result<Self::Ok, T>>(MapErr::new(self, f))
    }

    fn slim_map_err_async<F, Fut>(self, f: F) -> MapErrAsync<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Error) -> Fut,
        Fut: IntoFuture,
    {
        support::assert_future::<_, Result<Self::Ok, Fut::Output>>(MapErrAsync::new(self, f))
    }

    fn slim_map_into<T>(self) -> MapInto<Self, T>
    where
        Self: Sized,
        Self::Output: Into<T>,
    {
        support::assert_future::<_, T>(MapInto::new(self))
    }

    fn slim_map_ok<F, T>(self, f: F) -> MapOk<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Ok) -> T,
    {
        support::assert_future::<_, Result<T, Self::Error>>(MapOk::new(self, f))
    }

    fn slim_map_ok_async<F, Fut>(self, f: F) -> MapOkAsync<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Ok) -> Fut,
        Fut: IntoFuture,
    {
        support::assert_future::<_, Result<Fut::Output, Self::Error>>(MapOkAsync::new(self, f))
    }

    fn slim_map_ok_or_else<D, F, T>(self, default: D, f: F) -> MapOkOrElse<Self, D, F>
    where
        Self: ResultFuture + Sized,
        D: FnMut(Self::Error) -> T,
        F: FnMut(Self::Ok) -> T,
    {
        support::assert_future::<_, T>(MapOkOrElse::new(self, default, f))
    }

    fn slim_map_ok_or_else_async<D, F, Fut1, Fut2>(self, default: D, f: F) -> MapOkOrElseAsync<Self, D, F>
    where
        Self: ResultFuture + Sized,
        D: FnMut(Self::Error) -> Fut1,
        F: FnMut(Self::Ok) -> Fut2,
        Fut1: IntoFuture,
        Fut2: IntoFuture<Output = Fut1::Output>,
    {
        support::assert_future::<_, Fut1::Output>(MapOkOrElseAsync::new(self, default, f))
    }

    fn slim_never_error(self) -> IntoResultFuture<Self, Never>
    where
        Self: Sized,
    {
        support::assert_future::<_, Result<Self::Output, Never>>(self.slim_into_result_future())
    }

    fn slim_ok_into<T>(self) -> OkInto<Self, T>
    where
        Self: ResultFuture + Sized,
        Self::Ok: Into<T>,
    {
        support::assert_future::<_, Result<T, Self::Error>>(OkInto::new(self))
    }

    fn slim_or_else<F, T>(self, f: F) -> OrElse<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Error) -> Result<Self::Ok, T>,
    {
        support::assert_future::<_, Result<Self::Ok, T>>(OrElse::new(self, f))
    }

    fn slim_or_else_async<F, Fut>(self, f: F) -> OrElseAsync<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Error) -> Fut,
        Fut: support::IntoResultFuture<Ok = Self::Ok>,
    {
        support::assert_future::<_, Fut::Output>(OrElseAsync::new(self, f))
    }

    fn slim_raw_map_ok_or_else_async<D, F, Fut>(self, default: D, f: F) -> RawMapOkOrElseAsync<Self, D, F>
    where
        Self: ResultFuture + Sized,
        D: FnMut(Self::Error) -> Fut,
        F: FnMut(Self::Ok) -> Fut,
        Fut: IntoFuture,
    {
        support::assert_future::<_, Fut::Output>(RawMapOkOrElseAsync::new(self, default, f))
    }

    fn slim_try_flatten(self) -> TryFlatten<Self>
    where
        Self: Sized,
        Self::Output: Try,
        <Self::Output as Try>::Output: IntoFuture,
        <<Self::Output as Try>::Output as IntoFuture>::Output: FromResidual<<Self::Output as Try>::Residual> + Try,
    {
        support::assert_future::<_, <<Self::Output as Try>::Output as IntoFuture>::Output>(TryFlatten::new(self))
    }

    fn slim_try_flatten_err(self) -> TryFlattenErr<Self>
    where
        Self: ResultFuture + Sized,
        Self::Error: support::IntoResultFuture<Ok = Self::Ok>,
    {
        support::assert_future::<_, <<Self::Error as IntoFuture>::IntoFuture as Future>::Output>(TryFlattenErr::new(
            self,
        ))
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

    fn slim_unwrap_or_else_async<F, Fut>(self, f: F) -> UnwrapOrElseAsync<Self, F>
    where
        Self: ResultFuture + Sized,
        F: FnMut(Self::Error) -> Fut,
        Fut: IntoFuture<Output = Self::Ok>,
    {
        support::assert_future::<_, Self::Ok>(UnwrapOrElseAsync::new(self, f))
    }
}

impl<Fut> FutureExt for Fut where Fut: Future + ?Sized {}
