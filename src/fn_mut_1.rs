pub trait FnMut1<T> {
    type Output;
    type Raw: FnMut(T) -> Self::Output;
}

impl<T, F, U> FnMut1<T> for F
where
    F: FnMut(T) -> U,
{
    type Output = U;
    type Raw = Self;
}
