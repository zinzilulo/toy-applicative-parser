use crate::prelude::{Applicative, Functor, Monad};

pub type Maybe<T> = Option<T>;
pub use std::option::Option::{None as Nothing, Some as Just};

pub fn catMaybes<T, I>(iter: I) -> Vec<T>
where
    I: IntoIterator<Item = Maybe<T>>,
{
    iter.into_iter().flatten().collect()
}

impl<'a, T0> Functor<'a> for Option<T0> {
    type Wrapped<T>
        = Option<T>
    where
        T: 'a;

    fn fmap<A, B, F>(fa: &Option<A>, f: F) -> Option<B>
    where
        A: Clone,
        F: Fn(A) -> B + 'a,
    {
        fa.as_ref().map(|x| f(x.clone()))
    }
}

impl<'a, T> Applicative<'a> for Maybe<T> {
    fn pure<A>(a: &A) -> Self::Wrapped<A>
    where
        A: 'a + Clone,
    {
        Just(a.clone())
    }

    fn ap<X, B, FFn>(fa: &Self::Wrapped<X>, fab: Self::Wrapped<FFn>) -> Self::Wrapped<B>
    where
        B: 'a,
        X: Clone + 'a,
        FFn: Fn(X) -> B + 'a,
    {
        match (fa, fab) {
            (Just(x), Just(f)) => Just(f(x.clone())),
            _ => Nothing,
        }
    }
}

impl<'a, T> Monad<'a> for Maybe<T> {
    fn bind<A, B, K>(ma: &Self::Wrapped<A>, k: K) -> Self::Wrapped<B>
    where
        A: 'a + Clone,
        B: 'a,
        K: Fn(A) -> Self::Wrapped<B> + 'a,
    {
        ma.as_ref().and_then(|x| k(x.clone()))
    }
}
