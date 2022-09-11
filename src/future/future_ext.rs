use crate::future::and_then::AndThen;
use crate::future::and_then_async::AndThenAsync;
use crate::future::err_into::ErrInto;
use crate::future::flatten::Flatten;
use crate::future::inspect::Inspect;
use crate::future::inspect_err::InspectErr;
use crate::future::inspect_ok::InspectOk;
use crate::future::into_try_future::IntoTryFuture;
use crate::future::map::Map;
use crate::future::map_async::MapAsync;
use crate::future::map_err::MapErr;
use crate::future::map_err_async::MapErrAsync;
use crate::future::map_into::MapInto;
use crate::future::map_ok::MapOk;
use crate::future::map_ok_async::MapOkAsync;
use crate::future::map_ok_or_else::MapOkOrElse;
use crate::future::ok_into::OkInto;
use crate::future::or_else::OrElse;
use crate::future::or_else_async::OrElseAsync;
use crate::future::try_flatten::TryFlatten;
use crate::future::try_flatten_err::TryFlattenErr;
use crate::future::unwrap_or_else::UnwrapOrElse;
use crate::future::unwrap_or_else_async::UnwrapOrElseAsync;
use crate::support::{self, AsyncIterator, Never, TryFuture};
use std::future::Future;

pub trait FutureExt: Future {
    fn slim_and_then<F, T, E, U>(self, f: F) -> AndThen<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> Result<U, E>,
    {
        support::assert_future::<_, Result<U, E>>(AndThen::new(self, f))
    }

    fn slim_and_then_async<F, T, E, Fut2, U>(self, f: F) -> AndThenAsync<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> Fut2,
        Fut2: Future<Output = Result<U, E>>,
    {
        support::assert_future::<_, Result<U, E>>(AndThenAsync::new(self, f))
    }

    fn slim_err_into<U>(self) -> ErrInto<Self, U>
    where
        Self: TryFuture + Sized,
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

    fn slim_inspect_err<F, T, E>(self, f: F) -> InspectErr<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(&E),
    {
        support::assert_future::<_, Self::Output>(InspectErr::new(self, f))
    }

    fn slim_inspect_ok<F, T, E>(self, f: F) -> InspectOk<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(&T),
    {
        support::assert_future::<_, Self::Output>(InspectOk::new(self, f))
    }

    fn slim_into_try_future<E>(self) -> IntoTryFuture<Self, E>
    where
        Self: Sized,
    {
        support::assert_future::<_, Result<Self::Output, E>>(IntoTryFuture::new(self))
    }

    fn slim_map<F, T>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> T,
    {
        support::assert_future::<_, T>(Map::new(self, f))
    }

    fn slim_map_async<F, Fut2>(self, f: F) -> MapAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> Fut2,
        Fut2: Future,
    {
        support::assert_future::<_, Fut2::Output>(MapAsync::new(self, f))
    }

    fn slim_map_err<F, T, E, U>(self, f: F) -> MapErr<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(E) -> U,
    {
        support::assert_future::<_, Result<T, U>>(MapErr::new(self, f))
    }

    fn slim_map_err_async<F, T, E, Fut2>(self, f: F) -> MapErrAsync<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(E) -> Fut2,
        Fut2: Future,
    {
        support::assert_future::<_, Result<T, Fut2::Output>>(MapErrAsync::new(self, f))
    }

    fn slim_map_into<T>(self) -> MapInto<Self, T>
    where
        Self: Sized,
        Self::Output: Into<T>,
    {
        support::assert_future::<_, T>(MapInto::new(self))
    }

    fn slim_map_ok<F, T, E, U>(self, f: F) -> MapOk<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> U,
    {
        support::assert_future::<_, Result<U, E>>(MapOk::new(self, f))
    }

    fn slim_map_ok_async<F, T, E, Fut2>(self, f: F) -> MapOkAsync<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> Fut2,
        Fut2: Future,
    {
        support::assert_future::<_, Result<Fut2::Output, E>>(MapOkAsync::new(self, f))
    }

    fn slim_map_ok_or_else<F, T, E, G, U>(self, ok_fn: F, err_fn: G) -> MapOkOrElse<Self, F, G>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> U,
        G: FnMut(E) -> U,
    {
        support::assert_future::<_, U>(MapOkOrElse::new(self, ok_fn, err_fn))
    }

    fn slim_never_error(self) -> IntoTryFuture<Self, Never>
    where
        Self: Sized,
    {
        support::assert_future::<_, Result<Self::Output, Never>>(self.slim_into_try_future())
    }

    fn slim_ok_into<U>(self) -> OkInto<Self, U>
    where
        Self: TryFuture + Sized,
        Self::Ok: Into<U>,
    {
        support::assert_future::<_, Result<U, Self::Error>>(OkInto::new(self))
    }

    fn slim_or_else<F, T, E, U>(self, f: F) -> OrElse<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(E) -> Result<T, U>,
    {
        support::assert_future::<_, Result<T, U>>(OrElse::new(self, f))
    }

    fn slim_or_else_async<F, T, E, Fut2, U>(self, f: F) -> OrElseAsync<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(E) -> Fut2,
        Fut2: Future<Output = Result<T, U>>,
    {
        support::assert_future::<_, Result<T, U>>(OrElseAsync::new(self, f))
    }

    fn slim_try_flatten<Fut2, E, T>(self) -> TryFlatten<Self>
    where
        Self: Future<Output = Result<Fut2, E>> + Sized,
        Fut2: Future<Output = Result<T, E>> + Sized,
    {
        support::assert_future::<_, Result<T, E>>(TryFlatten::new(self))
    }

    fn slim_try_flatten_err<Fut2, T, E>(self) -> TryFlattenErr<Self>
    where
        Self: Future<Output = Result<T, Fut2>> + Sized,
        Fut2: Future<Output = Result<T, E>> + Sized,
    {
        support::assert_future::<_, Result<T, E>>(TryFlattenErr::new(self))
    }

    fn slim_unit_error(self) -> IntoTryFuture<Self, ()>
    where
        Self: Sized,
    {
        support::assert_future::<_, Result<Self::Output, ()>>(self.slim_into_try_future())
    }

    fn slim_unwrap_or_else<F, T, E>(self, f: F) -> UnwrapOrElse<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(E) -> T,
    {
        support::assert_future::<_, T>(UnwrapOrElse::new(self, f))
    }

    fn slim_unwrap_or_else_async<F, T, E, Fut2>(self, f: F) -> UnwrapOrElseAsync<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(E) -> Fut2,
        Fut2: Future<Output = T>,
    {
        support::assert_future::<_, T>(UnwrapOrElseAsync::new(self, f))
    }
}

impl<Fut> FutureExt for Fut where Fut: Future + ?Sized {}
