use crate::support::fn_mut_1::FnMut1;
use core::marker::PhantomData;
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

impl<A, B> FnMut1<A> for EitherLeftFn<A, B> {
    type Output = Either<A, B>;

    fn call_mut(&mut self, arg: A) -> Self::Output {
        Either::Left(arg)
    }
}
