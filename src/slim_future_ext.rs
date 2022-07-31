use crate::assert_future;
use crate::slim_flatten::SlimFlatten;
use crate::slim_map::SlimMap;
use crate::slim_map_async::SlimMapAsync;
use std::future::Future;

pub trait SlimFutureExt: Future {
    fn slim_flatten(self) -> SlimFlatten<Self>
    where
        Self: Sized,
        Self::Output: Future,
    {
        assert_future::assert_future::<_, <Self::Output as Future>::Output>(SlimFlatten::new(self))
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
}

impl<T> SlimFutureExt for T where T: Future + ?Sized {}
