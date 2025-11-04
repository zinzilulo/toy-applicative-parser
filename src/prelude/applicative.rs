use crate::prelude::functor::Functor;
use std::sync::Arc;

pub trait Applicative<'a>: Functor<'a> {
    fn pure<A>(a: &A) -> Self::Wrapped<A>
    where
        A: Clone + 'a;

    fn ap<X, B, FFn>(fa: &Self::Wrapped<X>, fab: Self::Wrapped<FFn>) -> Self::Wrapped<B>
    where
        X: Clone + 'a,
        FFn: Fn(X) -> B + 'a;

    fn liftA2<X, Y, C, F2>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>, f: F2) -> Self::Wrapped<C>
    where
        Self: Sized,
        X: Clone + 'a,
        Y: Clone + 'a,
        F2: Fn(X, Y) -> C + 'a,
    {
        let f_arc = Arc::new(f);
        let ff = <Self as Functor>::fmap(fa, {
            let f_arc = Arc::clone(&f_arc);
            move |x: X| {
                let f_arc = Arc::clone(&f_arc);
                move |y: Y| (f_arc)(x.clone(), y)
            }
        });
        Self::ap::<Y, C, _>(fb, ff)
    }

    fn then_keep_right<X, Y>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>) -> Self::Wrapped<Y>
    where
        Self: Sized,
        X: Clone + 'a,
        Y: Clone + 'a,
    {
        Self::liftA2(fa, fb, |_, y| y)
    }

    fn then_keep_left<X, Y>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>) -> Self::Wrapped<X>
    where
        Self: Sized,
        X: Clone + 'a,
        Y: Clone + 'a,
    {
        Self::liftA2(fa, fb, |x, _| x)
    }

    fn sequenceA<B>(fa: Vec<Self::Wrapped<B>>) -> Self::Wrapped<Vec<B>>
    where
        Self: Sized + Clone,
        B: Clone + 'a,
    {
        fa.into_iter()
            .rev()
            .fold(Self::pure(&Vec::new()), |acc, fx| {
                Self::liftA2(&fx, &acc, |a, mut bs| {
                    bs.insert(0, a);
                    bs
                })
            })
    }

    fn traverse<A, B, F>(fa: Vec<A>, f: F) -> Self::Wrapped<Vec<B>>
    where
        Self: Sized + Clone,
        B: Clone + 'a,
        F: Fn(A) -> Self::Wrapped<B> + 'a,
    {
        let mapped: Vec<Self::Wrapped<B>> = fa.into_iter().map(f).collect();
        Self::sequenceA(mapped)
    }
}

pub fn pure<'a, C, A>(x: A) -> C::Wrapped<A>
where
    C: Applicative<'a>,
    A: 'a + Clone,
{
    C::pure::<A>(&x)
}

pub fn liftA2<'a, C, A, B, Z>(
    pa: &C::Wrapped<A>,
    pb: &C::Wrapped<B>,
    f: impl Fn(A, B) -> Z + 'a,
) -> C::Wrapped<Z>
where
    A: Clone,
    B: Clone,
    C: Applicative<'a>,
{
    C::liftA2(pa, pb, f)
}

pub fn then_keep_left<'a, C, A, B>(pa: &C::Wrapped<A>, pb: &C::Wrapped<B>) -> C::Wrapped<A>
where
    A: Clone,
    B: Clone,
    C: Applicative<'a>,
{
    C::then_keep_left(pa, pb)
}

pub fn then_keep_right<'a, C, A, B>(pa: &C::Wrapped<A>, pb: &C::Wrapped<B>) -> C::Wrapped<B>
where
    A: Clone,
    B: Clone,
    C: Applicative<'a>,
{
    C::then_keep_right(pa, pb)
}

pub fn sequenceA<'a, C, B>(fa: Vec<C::Wrapped<B>>) -> C::Wrapped<Vec<B>>
where
    C: Applicative<'a> + Sized + Clone,
    B: Clone + 'a,
{
    C::sequenceA(fa)
}

pub fn traverse<'a, C, A, B, F>(fa: Vec<A>, f: F) -> C::Wrapped<Vec<B>>
where
    C: Applicative<'a> + Sized + Clone,
    B: Clone + 'a,
    F: Fn(A) -> C::Wrapped<B> + 'a,
{
    C::traverse(fa, f)
}
