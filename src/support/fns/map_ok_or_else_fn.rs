use crate::support::fn_mut_1::FnMut1;

pub struct MapOkOrElseFn<F, G> {
    ok_fn: F,
    err_fn: G,
}

impl<F, G> MapOkOrElseFn<F, G> {
    pub fn new(ok_fn: F, err_fn: G) -> Self {
        Self { ok_fn, err_fn }
    }
}

impl<F, G> Clone for MapOkOrElseFn<F, G>
where
    F: Clone,
    G: Clone,
{
    fn clone(&self) -> Self {
        Self {
            ok_fn: self.ok_fn.clone(),
            err_fn: self.err_fn.clone(),
        }
    }
}

impl<T, E, F, G> FnMut1<Result<T, E>> for MapOkOrElseFn<F, G>
where
    F: FnMut1<T>,
    G: FnMut1<E, Output = F::Output>,
{
    type Output = F::Output;

    fn call_mut(&mut self, arg: Result<T, E>) -> Self::Output {
        match arg {
            Ok(value) => self.ok_fn.call_mut(value),
            Err(error) => self.err_fn.call_mut(error),
        }
    }
}
