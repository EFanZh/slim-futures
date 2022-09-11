use crate::support::fn_mut_1::FnMut1;

pub struct MapOkOrElseFn<D, F> {
    default: D,
    f: F,
}

impl<D, F> MapOkOrElseFn<D, F> {
    pub fn new(default: D, f: F) -> Self {
        Self { default, f }
    }
}

impl<D, F> Clone for MapOkOrElseFn<D, F>
where
    D: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            default: self.default.clone(),
            f: self.f.clone(),
        }
    }
}

impl<T, E, D, F> FnMut1<Result<T, E>> for MapOkOrElseFn<D, F>
where
    D: FnMut1<E>,
    F: FnMut1<T, Output = D::Output>,
{
    type Output = F::Output;

    fn call_mut(&mut self, arg: Result<T, E>) -> Self::Output {
        match arg {
            Ok(value) => self.f.call_mut(value),
            Err(error) => self.default.call_mut(error),
        }
    }
}
