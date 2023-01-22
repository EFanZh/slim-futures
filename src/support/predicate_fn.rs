use fn_traits::FnMut;

pub trait PredicateFn<T>: for<'a> FnMut<(&'a T,), Output = <Self as PredicateFn<T>>::Output> {
    type Output;
}

impl<T, F, R> PredicateFn<T> for F
where
    F: for<'a> FnMut<(&'a T,), Output = R>,
{
    type Output = R;
}
