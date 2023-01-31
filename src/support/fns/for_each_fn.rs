use fn_traits::FnMut;

#[derive(Clone)]
pub struct ForEachFn<F>
where
    F: ?Sized,
{
    f: F,
}

impl<F> ForEachFn<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<T, F> FnMut<((), T)> for ForEachFn<F>
where
    F: FnMut<(T,)> + ?Sized,
{
    type Output = F::Output;

    fn call_mut(&mut self, args: ((), T)) -> Self::Output {
        self.f.call_mut((args.1,))
    }
}
