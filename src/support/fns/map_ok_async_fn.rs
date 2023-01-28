use crate::future::Map;
use crate::support::fns::try_from_output_fn::TryFromOutputFn;
use crate::support::try_::Residual;
use core::future::IntoFuture;
use core::marker::PhantomData;
use fn_traits::FnMut;

pub struct MapOkAsyncFn<F, R>
where
    F: ?Sized,
{
    phantom: PhantomData<R>,
    f: F,
}

impl<F, R> MapOkAsyncFn<F, R> {
    pub fn new(f: F) -> Self {
        Self {
            phantom: PhantomData,
            f,
        }
    }
}

impl<F, R> Clone for MapOkAsyncFn<F, R>
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

impl<T, F, R> FnMut<(T,)> for MapOkAsyncFn<F, R>
where
    F: FnMut<(T,)> + ?Sized,
    F::Output: IntoFuture,
    R: Residual<<F::Output as IntoFuture>::Output>,
{
    type Output = Map<<F::Output as IntoFuture>::IntoFuture, TryFromOutputFn<R::TryType>>;

    fn call_mut(&mut self, args: (T,)) -> Self::Output {
        Map::new(self.f.call_mut(args).into_future(), TryFromOutputFn::default())
    }
}
