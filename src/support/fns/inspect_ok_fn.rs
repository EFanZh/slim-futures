use fn_traits::FnMut;

#[derive(Clone)]
pub struct InspectOkFn<F>
where
    F: ?Sized,
{
    f: F,
}

impl<F> InspectOkFn<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<'a, T, E, F> FnMut<(&'a Result<T, E>,)> for InspectOkFn<F>
where
    F: FnMut<(&'a T,), Output = ()> + ?Sized,
{
    type Output = ();

    fn call_mut(&mut self, args: (&'a Result<T, E>,)) -> Self::Output {
        if let Ok(value) = args.0 {
            self.f.call_mut((value,));
        }
    }
}
