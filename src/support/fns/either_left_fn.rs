use core::marker::PhantomData;
use fn_traits::FnMut;
use futures_util::future::Either;

pub struct EitherLeftFn<A, B> {
    phantom: PhantomData<fn(A) -> Either<A, B>>,
}

impl<A, B> Default for EitherLeftFn<A, B> {
    fn default() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<A, B> Clone for EitherLeftFn<A, B> {
    fn clone(&self) -> Self {
        Self { phantom: self.phantom }
    }
}

impl<A, B> FnMut<(A,)> for EitherLeftFn<A, B> {
    type Output = Either<A, B>;

    fn call_mut(&mut self, args: (A,)) -> Self::Output {
        Either::Left(args.0)
    }
}
