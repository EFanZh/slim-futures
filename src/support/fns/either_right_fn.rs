use core::marker::PhantomData;
use fn_traits::FnMut;
use futures_util::future::Either;

pub struct EitherRightFn<A> {
    phantom: PhantomData<fn() -> A>,
}

impl<A> Default for EitherRightFn<A> {
    fn default() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<A> Clone for EitherRightFn<A> {
    fn clone(&self) -> Self {
        Self { phantom: self.phantom }
    }
}

impl<A, B> FnMut<(B,)> for EitherRightFn<A> {
    type Output = Either<A, B>;

    fn call_mut(&mut self, args: (B,)) -> Self::Output {
        Either::Right(args.0)
    }
}
