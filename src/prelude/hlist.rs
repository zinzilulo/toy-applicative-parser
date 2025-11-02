use crate::prelude::{Applicative, Functor, Monad};

impl<T> Functor for Vec<T> {
    type Inner = T;
    type Wrapped<B> = Vec<B>;

    fn fmap<B, F>(self, f: F) -> Self::Wrapped<B>
    where
        F: Fn(Self::Inner) -> B + 'static,
    {
        self.into_iter().map(f).collect()
    }
}

impl<'a, T> Applicative<'a> for Vec<T> {
    fn pure<A>(a: A) -> Self::Wrapped<A>
    where
        A: Clone + 'a,
    {
        vec![a]
    }

    fn ap<X, B, FFn>(fa: &Self::Wrapped<X>, fab: Self::Wrapped<FFn>) -> Self::Wrapped<B>
    where
        X: Clone + 'a,
        FFn: Fn(X) -> B + 'a,
    {
        let mut out = Vec::new();
        for f in fab {
            for x in fa.iter().cloned() {
                out.push(f(x));
            }
        }
        out
    }

    fn liftA2<X, Y, C, F2>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>, f: F2) -> Self::Wrapped<C>
    where
        X: Clone + 'a,
        Y: Clone + 'a,
        F2: Fn(X, Y) -> C + 'a,
    {
        let mut out = Vec::new();
        for x in fa.iter() {
            for y in fb.iter() {
                out.push(f(x.clone(), y.clone()));
            }
        }
        out
    }

    fn then_keep_right<X, Y>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>) -> Self::Wrapped<Y>
    where
        Y: Clone + 'a,
    {
        let mut out = Vec::new();
        for _ in fa.iter() {
            for y in fb.iter().cloned() {
                out.push(y);
            }
        }
        out
    }

    fn then_keep_left<X, Y>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>) -> Self::Wrapped<X>
    where
        X: Clone + 'a,
    {
        let mut out = Vec::new();
        for x in fa.iter() {
            for _ in fb.iter() {
                out.push(x.clone());
            }
        }
        out
    }
}

impl<'a, T> Monad<'a> for Vec<T>
where
    T: Clone + 'static,
{
    fn bind<B, K>(self, k: K) -> Self::Wrapped<B>
    where
        K: Fn(Self::Inner) -> Self::Wrapped<B> + 'static,
    {
        let mut out = Vec::new();
        for a in self {
            out.extend(k(a));
        }
        out
    }
}

impl<T, const N: usize> Functor for [T; N] {
    type Inner = T;
    type Wrapped<B> = [B; N];

    fn fmap<B, F>(self, f: F) -> Self::Wrapped<B>
    where
        F: Fn(Self::Inner) -> B + 'static,
    {
        self.map(f)
    }
}

impl<'a, T, const N: usize> Applicative<'a> for [T; N] {
    fn pure<A>(a: A) -> Self::Wrapped<A>
    where
        A: Clone + 'a,
    {
        std::array::from_fn(|_| a.clone())
    }

    fn ap<X, B, FFn>(fa: &Self::Wrapped<X>, fab: Self::Wrapped<FFn>) -> Self::Wrapped<B>
    where
        X: Clone + 'a,
        FFn: Fn(X) -> B + 'a,
    {
        let fa_ref = &fa;
        let fab_ref = &fab;
        std::array::from_fn(|i| (fab_ref[i])(fa_ref[i].clone()))
    }

    fn liftA2<X, Y, C, F2>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>, f: F2) -> Self::Wrapped<C>
    where
        X: Clone + 'a,
        Y: Clone + 'a,
        F2: Fn(X, Y) -> C + 'a,
    {
        std::array::from_fn(|i| f(fa[i].clone(), fb[i].clone()))
    }

    fn then_keep_right<X, Y>(fa: &Self::Wrapped<X>, fb: &Self::Wrapped<Y>) -> Self::Wrapped<Y>
    where
        Y: Clone + 'a,
    {
        let _ = fa;
        fb.clone()
    }

    fn then_keep_left<X, Y>(fa: &Self::Wrapped<X>, _fb: &Self::Wrapped<Y>) -> Self::Wrapped<X>
    where
        X: Clone + 'a,
    {
        fa.clone()
    }
}

impl<'a, T> Monad<'a> for [T; 1] {
    fn bind<B, K>(self, k: K) -> Self::Wrapped<B>
    where
        K: Fn(Self::Inner) -> Self::Wrapped<B> + 'static,
    {
        let [a] = self;
        k(a)
    }
}
