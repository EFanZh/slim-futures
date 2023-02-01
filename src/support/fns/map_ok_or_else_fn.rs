use fn_traits::FnMut;

pub struct MapOkOrElseFn<D, F>
where
    F: ?Sized,
{
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

impl<T, E, D, F> FnMut<(Result<T, E>,)> for MapOkOrElseFn<D, F>
where
    D: FnMut<(E,)>,
    F: FnMut<(T,), Output = D::Output> + ?Sized,
{
    type Output = F::Output;

    fn call_mut(&mut self, args: (Result<T, E>,)) -> Self::Output {
        match args.0 {
            Ok(value) => self.f.call_mut((value,)),
            Err(error) => self.default.call_mut((error,)),
        }
    }
}
