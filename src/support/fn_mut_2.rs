pub trait FnMut2<T1, T2> {
    type Output;

    fn call_mut(&mut self, arg_1: T1, arg_2: T2) -> Self::Output;
}

impl<T1, T2, F, U> FnMut2<T1, T2> for F
where
    F: FnMut(T1, T2) -> U,
{
    type Output = U;

    fn call_mut(&mut self, arg_1: T1, arg_2: T2) -> Self::Output {
        self(arg_1, arg_2)
    }
}
