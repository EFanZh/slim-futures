pub trait FnMut1<T> {
    type Output;

    fn call_mut(&mut self, arg: T) -> Self::Output;
}

impl<T, F, U> FnMut1<T> for F
where
    F: FnMut(T) -> U,
{
    type Output = U;

    fn call_mut(&mut self, arg: T) -> Self::Output {
        self(arg)
    }
}
