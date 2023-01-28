use fn_traits::FnMut;

#[derive(Clone)]
pub struct MapErrFn<F>
where
    F: ?Sized,
{
    f: F,
}

impl<F> MapErrFn<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<T, E, F> FnMut<(Result<T, E>,)> for MapErrFn<F>
where
    F: FnMut<(E,)> + ?Sized,
{
    type Output = Result<T, F::Output>;

    fn call_mut(&mut self, args: (Result<T, E>,)) -> Self::Output {
        args.0.map_err(|value| self.f.call_mut((value,)))
    }
}
