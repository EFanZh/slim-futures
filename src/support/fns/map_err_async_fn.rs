use crate::future::Map;
use core::future::IntoFuture;
use core::marker::PhantomData;
use fn_traits::fns::ResultErrFn;
use fn_traits::FnMut;

pub struct MapErrAsyncFn<F, T>
where
    F: ?Sized,
{
    phantom: PhantomData<T>,
    f: F,
}

impl<F, T> MapErrAsyncFn<F, T> {
    pub fn new(f: F) -> Self {
        Self {
            phantom: PhantomData,
            f,
        }
    }
}

impl<F, T> Clone for MapErrAsyncFn<F, T>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            phantom: self.phantom,
            f: self.f.clone(),
        }
    }
}

impl<E, F, T> FnMut<(E,)> for MapErrAsyncFn<F, T>
where
    F: FnMut<(E,)> + ?Sized,
    F::Output: IntoFuture,
{
    type Output = Map<<F::Output as IntoFuture>::IntoFuture, ResultErrFn<T>>;

    fn call_mut(&mut self, args: (E,)) -> Self::Output {
        Map::new(self.f.call_mut(args).into_future(), ResultErrFn::default())
    }
}
