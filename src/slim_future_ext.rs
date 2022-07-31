use crate::assert_future;
use crate::slim_and_then::SlimAndThen;
use crate::slim_flatten::SlimFlatten;
use crate::slim_inspect::SlimInspect;
use crate::slim_map::SlimMap;
use crate::slim_map_async::SlimMapAsync;
use crate::slim_map_into::SlimMapInto;
use crate::slim_map_ok::SlimMapOk;
use crate::slim_try_flatten::SlimTryFlatten;
use std::future::Future;

pub trait SlimFutureExt: Future {
    fn slim_and_then<F, T, E, U>(self, f: F) -> SlimAndThen<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> Result<U, E>,
    {
        assert_future::assert_future::<_, Result<U, E>>(SlimAndThen::new(self, f))
    }

    fn slim_flatten(self) -> SlimFlatten<Self>
    where
        Self: Sized,
        Self::Output: Future,
    {
        assert_future::assert_future::<_, <Self::Output as Future>::Output>(SlimFlatten::new(self))
    }

    fn slim_inspect<F>(self, f: F) -> SlimInspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Output),
    {
        assert_future::assert_future::<_, Self::Output>(SlimInspect::new(self, f))
    }

    fn slim_map<F, U>(self, f: F) -> SlimMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> U,
    {
        assert_future::assert_future::<_, U>(SlimMap::new(self, f))
    }

    fn slim_map_async<F, U>(self, f: F) -> SlimMapAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> U,
        U: Future,
    {
        assert_future::assert_future::<_, U::Output>(SlimMapAsync::new(self, f))
    }

    fn slim_map_into<U>(self) -> SlimMapInto<Self, U>
    where
        Self: Sized,
        Self::Output: Into<U>,
    {
        assert_future::assert_future::<_, U>(SlimMapInto::new(self))
    }

    fn slim_map_ok<F, T, E, U>(self, f: F) -> SlimMapOk<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> U,
    {
        assert_future::assert_future::<_, Result<U, E>>(SlimMapOk::new(self, f))
    }

    fn slim_try_flatten<T, E, U>(self) -> SlimTryFlatten<Self>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        T: Future<Output = Result<U, E>> + Sized,
    {
        assert_future::assert_future::<_, Result<U, E>>(SlimTryFlatten::new(self))
    }
}

impl<T> SlimFutureExt for T where T: Future + ?Sized {}
