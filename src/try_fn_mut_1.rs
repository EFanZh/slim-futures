use crate::fn_mut_1::FnMut1;

pub trait TryFnMut1<T>: FnMut1<T, Output = Result<Self::Ok, Self::Error>> {
    type Ok;
    type Error;
}

impl<T, F, U, E> TryFnMut1<T> for F
where
    F: FnMut1<T, Output = Result<U, E>>,
{
    type Ok = U;
    type Error = E;
}
