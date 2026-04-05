use crate::prelude::{Alternative, Applicative, Functor, Just, Maybe, Nothing};
use std::sync::Arc;

pub struct Parser<'a, A, R>(
    pub  Arc<
        dyn Fn(
                &'a str,
                Arc<dyn Fn(A, &'a str) -> R + 'a>,
                Arc<dyn Fn(A) -> R + 'a>,
                Arc<dyn Fn(&'a str) -> R + 'a>,
                Arc<dyn Fn(&'a str) -> R + 'a>,
            ) -> R
            + 'a,
    >,
);

impl<A, R> Clone for Parser<'_, A, R> {
    fn clone(&self) -> Self {
        Parser(self.0.clone())
    }
}

pub fn runParser<'a, A>(p: Parser<'a, A, Maybe<A>>, inp: &'a str) -> Maybe<A>
where
    A: 'a + Clone,
{
    (p.0)(
        inp,
        Arc::new(|x: A, _| Just(x)),
        Arc::new(|x: A| Just(x)),
        Arc::new(|_| Nothing),
        Arc::new(|_| Nothing),
    )
}

impl<'a, T: 'a, R: 'a> Functor<'a> for Parser<'a, T, R> {
    type Wrapped<X>
        = Parser<'a, X, R>
    where
        X: 'a;

    fn fmap<A, B, F>(fa: Self::Wrapped<A>, f: F) -> Self::Wrapped<B>
    where
        A: 'a,
        B: 'a,
        F: Fn(A) -> B + 'a,
    {
        let Parser(p) = fa.clone();
        let f = Arc::new(f);

        Parser(Arc::new({
            move |inp: &'a str,
                  cok: Arc<dyn Fn(B, &'a str) -> R>,
                  eok: Arc<dyn Fn(B) -> R>,
                  cerr: Arc<dyn Fn(&'a str) -> R>,
                  eerr: Arc<dyn Fn(&'a str) -> R>|
                  -> R {
                p(
                    inp,
                    Arc::new({
                        let f1 = f.clone();
                        move |b, rest| cok(f1(b), rest)
                    }),
                    Arc::new({
                        let f2 = f.clone();
                        move |b| eok(f2(b))
                    }),
                    cerr,
                    eerr,
                )
            }
        }))
    }
}

impl<'a, T: 'a, R: 'a> Applicative<'a> for Parser<'a, T, R> {
    fn pure<B>(b: B) -> Self::Wrapped<B>
    where
        B: Clone + 'a,
    {
        let b = b.clone();
        Parser(Arc::new({
            move |_inp: &'a str,
                  _cok: Arc<dyn Fn(B, &'a str) -> R>,
                  eok: Arc<dyn Fn(B) -> R>,
                  _cerr: Arc<dyn Fn(&'a str) -> R>,
                  _eerr: Arc<dyn Fn(&'a str) -> R>|
                  -> R { eok(b.clone()) }
        }))
    }

    fn ap<A, B, F>(fa: Self::Wrapped<A>, fab: Self::Wrapped<F>) -> Self::Wrapped<B>
    where
        A: Clone + 'a,
        B: 'a,
        F: Fn(A) -> B + 'a,
    {
        let fa = fa.clone();
        let Parser(p_fun) = fab;

        Parser(Arc::new({
            move |inp: &'a str,
                  cok: Arc<dyn Fn(B, &'a str) -> R>,
                  eok: Arc<dyn Fn(B) -> R>,
                  cerr: Arc<dyn Fn(&'a str) -> R>,
                  eerr: Arc<dyn Fn(&'a str) -> R>|
                  -> R {
                p_fun(
                    inp,
                    Arc::new({
                        let fa = fa.clone();
                        let cok = cok.clone();
                        let eok = eok.clone();
                        let cerr = cerr.clone();
                        move |f, rest| {
                            let fa = fa.clone();
                            let cok = cok.clone();
                            let eok = eok.clone();
                            let cerr = cerr.clone();
                            let Parser(p_val) = Self::fmap(fa, f);
                            p_val(
                                rest,
                                cok,
                                eok,
                                cerr.clone(),
                                Arc::new(move |s| cerr.clone()(s)),
                            )
                        }
                    }),
                    Arc::new({
                        let fa = fa.clone();
                        let cok = cok.clone();
                        let eok = eok.clone();
                        let cerr = cerr.clone();
                        let eerr = eerr.clone();
                        move |f| {
                            let fa = fa.clone();
                            let cok = cok.clone();
                            let eok = eok.clone();
                            let cerr = cerr.clone();
                            let eerr = eerr.clone();
                            let Parser(p_val) = Self::fmap(fa, f);
                            p_val(inp, cok, eok, cerr, eerr)
                        }
                    }),
                    cerr,
                    eerr,
                )
            }
        }))
    }
}

impl<'a, T: 'a, R: 'a> Alternative<'a> for Parser<'a, T, R> {
    fn empty<B>() -> Self::Wrapped<B>
    where
        B: 'a,
    {
        Parser(Arc::new({
            move |inp: &'a str,
                  _cok: Arc<dyn Fn(B, &'a str) -> R>,
                  _eok: Arc<dyn Fn(B) -> R>,
                  _cerr: Arc<dyn Fn(&'a str) -> R>,
                  eerr: Arc<dyn Fn(&'a str) -> R>|
                  -> R { eerr(inp) }
        }))
    }

    fn alt<B>(fa: Self::Wrapped<B>, fb: Self::Wrapped<B>) -> Self::Wrapped<B>
    where
        B: 'a,
    {
        let p = fa.0.clone();
        let q = fb.0.clone();

        Parser(Arc::new({
            move |inp: &'a str,
                  cok: Arc<dyn Fn(B, &'a str) -> R>,
                  eok: Arc<dyn Fn(B) -> R>,
                  cerr: Arc<dyn Fn(&'a str) -> R>,
                  eerr: Arc<dyn Fn(&'a str) -> R>|
                  -> R {
                let q = q.clone();
                p(
                    inp,
                    cok.clone(),
                    eok.clone(),
                    cerr.clone(),
                    Arc::new(move |_| q(inp, cok.clone(), eok.clone(), cerr.clone(), eerr.clone())),
                )
            }
        }))
    }
}

pub trait IntoPure<'a, R>: Sized {
    fn into_pure(self) -> Parser<'a, Self, R>;
}

impl<'a, A: 'a + Clone, R: 'a + Clone> IntoPure<'a, R> for A {
    #[inline]
    fn into_pure(self) -> Parser<'a, Self, R> {
        <Parser<'a, Self, R> as Applicative<'a>>::pure(self)
    }
}
