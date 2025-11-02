use crate::hs::functor::Functor;

pub trait Applicative<'a>: Functor {
    fn pure<A>(a: A) -> Self::Wrapped<A>
    where
        A: Clone + 'a;

    fn ap<X, B, FFn>(fa: &Self::Wrapped<X>, fab: Self::Wrapped<FFn>) -> Self::Wrapped<B>
    where
        X: Clone + 'a,
        FFn: Fn(X) -> B + 'a;

    fn liftA2<X, Y, C, F2>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>, f: F2) -> Self::Wrapped<C>
    where
        X: Clone + 'static,
        Y: Clone + 'static,
        F2: Fn(X, Y) -> C + 'static;

    fn then_keep_right<X, Y>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>) -> Self::Wrapped<Y>
    where
        X: Clone + 'a,
        Y: Clone + 'a;

    fn then_keep_left<X, Y>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>) -> Self::Wrapped<X>
    where
        X: Clone + 'a,
        Y: Clone + 'a;

    fn sequenceA<B>(fa: Vec<Self::Wrapped<B>>) -> Self::Wrapped<Vec<B>>
    where
        Self: Sized + Clone,
        B: Clone + 'static,
    {
        fa.into_iter()
            .rev()
            .fold(Self::pure(Vec::new()), |acc, fx| {
                Self::liftA2(&fx, &acc, |a, mut bs| {
                    bs.insert(0, a);
                    bs
                })
            })
    }

    fn traverse<A, B, F>(fa: Vec<A>, f: F) -> Self::Wrapped<Vec<B>>
    where
        Self: Sized + Clone,
        B: Clone + 'static,
        F: Fn(A) -> Self::Wrapped<B> + 'static,
    {
        Self::sequenceA(fa.fmap(f))
    }
}
