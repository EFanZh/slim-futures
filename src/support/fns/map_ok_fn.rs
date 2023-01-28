use fn_traits::FnMut;

#[derive(Clone)]
pub struct MapOkFn<F>
where
    F: ?Sized,
{
    f: F,
}

impl<F> MapOkFn<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<T, E, F> FnMut<(Result<T, E>,)> for MapOkFn<F>
where
    F: FnMut<(T,)> + ?Sized,
{
    type Output = Result<F::Output, E>;

    fn call_mut(&mut self, args: (Result<T, E>,)) -> Self::Output {
        args.0.map(|value| self.f.call_mut((value,)))
    }
}
