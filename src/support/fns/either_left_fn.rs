use core::marker::PhantomData;
use fn_traits::FnMut;
use futures_util::future::Either;

pub struct EitherLeftFn<B> {
    phantom: PhantomData<fn() -> B>,
}

impl<B> Default for EitherLeftFn<B> {
    fn default() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<B> Clone for EitherLeftFn<B> {
    fn clone(&self) -> Self {
        Self { phantom: self.phantom }
    }
}

impl<A, B> FnMut<(A,)> for EitherLeftFn<B> {
    type Output = Either<A, B>;

    fn call_mut(&mut self, args: (A,)) -> Self::Output {
        Either::Left(args.0)
    }
}
