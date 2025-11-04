pub trait Functor<'a> {
    type Wrapped<T>: 'a
    where
        T: 'a;

    fn fmap<A, B, F>(fa: &Self::Wrapped<A>, f: F) -> Self::Wrapped<B>
    where
        A: Clone + 'a,
        F: Fn(A) -> B + 'a;
}

pub fn fmap<'a, C, A, B>(p: &C::Wrapped<A>, f: impl Fn(A) -> B + 'a) -> C::Wrapped<B>
where
    A: Clone,
    C: Functor<'a>,
{
    C::fmap(p, f)
}
