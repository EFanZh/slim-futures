use fn_traits::FnMut;

#[derive(Clone)]
pub struct InspectErrFn<F>
where
    F: ?Sized,
{
    f: F,
}

impl<F> InspectErrFn<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<'a, T, E, F> FnMut<(&'a Result<T, E>,)> for InspectErrFn<F>
where
    F: FnMut<(&'a E,), Output = ()> + ?Sized,
{
    type Output = ();

    fn call_mut(&mut self, args: (&'a Result<T, E>,)) -> Self::Output {
        if let Err(error) = args.0 {
            self.f.call_mut((error,));
        }
    }
}
