#![allow(non_snake_case)]

mod prelude;
use crate::prelude::liftA2;
mod miniparsec;
mod parser;

mod parser_demo {
    use crate::parser::{parse, IntoPure, Parser};
    use crate::prelude::{
        alt, catMaybes, fmap, liftA2, many, then_keep_left, then_keep_right, Alternative,
        Applicative, Just, Maybe, Nothing,
    };
    use std::sync::Arc;

    type P<'a> = Parser<'a, ()>;

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Command {
        F,
        L,
        R,
        B(Vec<Command>),
    }

    fn lazy<'a, A, F>(mk: F) -> Parser<'a, A>
    where
        F: Fn() -> Parser<'a, A> + 'a,
        A: 'a,
    {
        Parser(Arc::new(move |s: &'a str| parse(&mk(), s)))
    }

    fn satisfy<'a, F>(f: F) -> Parser<'a, char>
    where
        F: Fn(char) -> bool + 'a,
    {
        fn eat<'a, F>(s: &'a str, f: &F) -> Vec<(char, &'a str)>
        where
            F: Fn(char) -> bool,
        {
            if let Just(c) = s.chars().next() {
                let rest = &s[c.len_utf8()..];
                if f(c) {
                    return vec![(c, rest)];
                }
            }
            vec![]
        }

        Parser(Arc::new(move |s: &'a str| eat(s, &f)))
    }

    fn char_<'a>(c: char) -> Parser<'a, char> {
        satisfy(move |chr| c == chr)
    }

    fn string_<'a, S>(gs: S) -> Parser<'a, String>
    where
        S: Into<String>,
    {
        let s = gs.into();
        if s.is_empty() {
            String::new().into_pure()
        } else {
            let mut it = s.chars();
            let c = it.next().unwrap();
            let cs = it.as_str();

            let head: Parser<'a, char> = char_(c);
            let tail: Parser<'a, String> = string_(cs.to_string());

            let out: Parser<'a, String> = <Parser<'a, char>>::liftA2(&head, &tail, |h, t| {
                let mut s = String::with_capacity(1 + t.len());
                s.push(h);
                s.push_str(&t);
                s
            });
            out
        }
    }

    fn parse_fully<'a, A: Clone>(p: &Parser<'a, A>, s: &'a str) -> Vec<A> {
        parse(p, s)
            .into_iter()
            .filter_map(|(x, rest)| if rest.is_empty() { Just(x) } else { Nothing })
            .collect()
    }

    fn cmd<'a>() -> Parser<'a, Maybe<Command>> {
        let oneOf = |cs: String| satisfy(move |c| cs.contains(c));
        let noneOf = |cs: String| satisfy(move |c| !cs.contains(c));

        let inner_b: Parser<'a, Command> = fmap::<Parser<'a, ()>, _, _>(&lazy(cmds), Command::B);

        let f: Parser<'a, Maybe<Command>> = then_keep_left::<Parser<'a, ()>, _, _>(
            &(Just(Command::F).into_pure()),
            &oneOf("MN".to_string()),
        );

        let l: Parser<'a, Maybe<Command>> =
            then_keep_left::<Parser<'a, ()>, _, _>(&(Just(Command::L).into_pure()), &char_('+'));

        let r: Parser<'a, Maybe<Command>> =
            then_keep_left::<Parser<'a, ()>, _, _>(&(Just(Command::R).into_pure()), &char_('-'));

        let b_head: Parser<'a, Command> = <Parser<'a, char>>::then_keep_left(
            &then_keep_right::<Parser<'a, ()>, _, _>(&char_('['), &inner_b),
            &char_(']'),
        );

        let b: Parser<'a, Maybe<Command>> = fmap::<Parser<'a, ()>, _, _>(&b_head, Just);

        let n: Parser<'a, Maybe<Command>> =
            fmap::<Parser<'a, ()>, _, _>(&noneOf("MN+-[]".to_string()), |_| Nothing::<Command>);

        <Parser<'a, Maybe<Command>>>::alt(
            &alt::<Parser<'a, ()>, _>(&<Parser<'a, Maybe<Command>>>::alt(&f, &l), &r),
            &alt::<Parser<'a, ()>, _>(&b, &n),
        )
    }

    fn cmds<'a>() -> Parser<'a, Vec<Command>> {
        fmap::<P<'a>, _, _>(&many::<Parser<'a, ()>, _>(&cmd()), catMaybes)
    }

    pub fn run() {
        let item: Parser<'_, char> = satisfy(|_| true);

        let digit: Parser<'_, char> = satisfy(|c| c.is_ascii_digit());
        let digit_tests: Vec<Vec<(char, &str)>> =
            vec![parse(&digit, "a"), parse(&digit, "0"), parse(&digit, "23")];
        println!("{digit_tests:?}");

        let item_tests: Vec<Vec<(char, &str)>> = vec![parse(&item, ""), parse(&item, "a1")];
        println!("{item_tests:?}");

        let multi_digit: Parser<'_, (char, char)> =
            liftA2::<P<'_>, _, _, _>(&digit, &digit, |x, y| (x, y));
        let pair_of_digits: Vec<((char, char), &str)> = parse(&multi_digit, "423");
        println!("{pair_of_digits:?}");

        let string_bang: Parser<'_, String> =
            then_keep_left::<P<'_>, _, _>(&string_("hello"), &char_('!'));
        let string_tests: Vec<Vec<(String, &str)>> = vec![
            parse(&string_("hello"), "hello world"),
            parse(&string_bang, "hello!"),
        ];
        println!("{string_tests:?}");

        let many_digit_values: Vec<(Vec<u32>, &str)> = parse(
            &many::<P<'_>, _>(&fmap::<Parser<'_, ()>, _, _>(&digit, |c: char| -> u32 {
                c.to_digit(10).unwrap()
            })),
            "12a",
        );
        println!("{many_digit_values:?}");

        let to_ord = |c: char| c as u32;
        let ord_item_tests: Vec<Vec<(u32, &str)>> = vec![
            parse(&fmap::<P<'_>, _, _>(&item, to_ord), "a"),
            parse(&fmap::<P<'_>, _, _>(&digit, to_ord), "1"),
        ];
        println!("{ord_item_tests:?}");

        let full_only: Vec<Vec<char>> = parse_fully(&many::<P<'_>, _>(&digit), "12a");
        println!("{full_only:?}");

        let command_parse: Vec<(Vec<Command>, &str)> = parse(&cmds(), "M+X[-N]+[]");
        let first_result_commands: &Vec<Command> = &(command_parse.first().unwrap()).0;
        println!("{first_result_commands:?}");
    }
}

mod miniparsec_demo {
    use crate::miniparsec::{runParser, IntoPure, Parser};
    use crate::prelude::{alt, fmap, then_keep_left, then_keep_right, Applicative, Maybe};
    use std::sync::Arc;

    fn satisfy<'a, F, R: Clone>(pf: F) -> Parser<'a, char, R>
    where
        F: Fn(char) -> bool + 'static,
    {
        Parser(Arc::new(
            move |inp: &'a str,
                  cok: Arc<dyn Fn(char, &'a str) -> R>,
                  _eok: Arc<dyn Fn(char) -> R>,
                  _cerr: Arc<dyn Fn(&'a str) -> R>,
                  eerr: Arc<dyn Fn(&'a str) -> R>|
                  -> R {
                let mut chars = inp.chars();

                match chars.next() {
                    Some(c) if pf(c) => cok(c, chars.as_str()),
                    _ => eerr(inp),
                }
            },
        ))
    }

    fn char_<'a>(c: char) -> Parser<'a, char, Maybe<String>> {
        satisfy(move |chr| c == chr)
    }

    fn string_<'a, S>(gs: S) -> Parser<'a, String, Maybe<String>>
    where
        S: Into<String>,
    {
        let s = gs.into();
        if s.is_empty() {
            String::new().into_pure()
        } else {
            let mut it = s.chars();
            let c = it.next().unwrap();
            let cs = it.as_str();

            let head: Parser<'a, char, Maybe<String>> = char_(c);
            let tail: Parser<'a, String, Maybe<String>> = string_(cs.to_string());

            let out: Parser<'a, String, Maybe<String>> =
                <Parser<'a, char, Maybe<String>>>::liftA2(&head, &tail, |h, t| {
                    let mut s = String::with_capacity(1 + t.len());
                    s.push(h);
                    s.push_str(&t);
                    s
                });
            out
        }
    }

    fn atomic<'a, A: 'a, R: 'a>(p: Parser<'a, A, R>) -> Parser<'a, A, R>
where {
        Parser(Arc::new(
            move |inp: &'a str,
                  cok: Arc<dyn Fn(A, &'a str) -> R>,
                  eok: Arc<dyn Fn(A) -> R>,
                  _cerr: Arc<dyn Fn(&'a str) -> R>,
                  eerr: Arc<dyn Fn(&'a str) -> R>|
                  -> R { (p.0)(inp, cok, eok, eerr.clone(), eerr.clone()) },
        ))
    }

    pub fn run() {
        let item: Parser<'_, char, Maybe<char>> = satisfy(|_| true);
        let item_tests: Vec<Maybe<char>> = vec![runParser(&item, ""), runParser(&item, "a1")];
        println!("{item_tests:?}");

        let digit: Parser<'_, char, Maybe<char>> = satisfy(|c| c.is_ascii_digit());
        let digit_tests: Vec<Maybe<char>> = vec![
            runParser(&digit, "a"),
            runParser(&digit, "0"),
            runParser(&digit, "23"),
        ];
        println!("{digit_tests:?}");

        let non_atomic: Parser<'_, String, Maybe<String>> =
            alt::<Parser<'_, String, Maybe<String>>, _>(&string_("hi"), &string_("hello"));
        let non_atomic_tests = vec![
            runParser(&non_atomic, "hihello"),
            runParser(&non_atomic, "hellohi"),
        ];
        println!("{non_atomic_tests:?}");

        let full: Parser<'_, String, Maybe<String>> =
            alt::<Parser<'_, String, Maybe<String>>, _>(&atomic(string_("hi")), &string_("hello"));
        let full_tests: Vec<Maybe<String>> =
            vec![runParser(&full, "hihello"), runParser(&full, "hellohi")];
        println!("{full_tests:?}");

        let empty_string = string_("");
        let tests = vec![
            runParser(&empty_string, ""),
            runParser(&empty_string, "abc"),
        ];
        println!("{tests:?}");

        fn char_item<'a>(c: char) -> Parser<'a, char, Maybe<char>> {
            satisfy(move |chr| c == chr)
        }
        let char_a: Parser<'_, char, Maybe<char>> = char_item('a');
        let char_b: Parser<'_, char, Maybe<char>> = char_item('b');
        let char_alt: Parser<'_, char, Maybe<char>> =
            alt::<Parser<'_, char, Maybe<char>>, _>(&char_a, &char_b);
        let char_alt_tests: Vec<Maybe<char>> = vec![
            runParser(&char_alt, "b"),
            runParser(&char_alt, "a"),
            runParser(&char_alt, "c"),
        ];
        println!("{char_alt_tests:?}");

        let prefix_no_atomic: Parser<'_, String, Maybe<String>> =
            alt::<Parser<'_, String, Maybe<String>>, _>(&string_("hi"), &string_("hello"));
        let prefix_no_atomic_tests: Vec<Maybe<String>> = vec![
            runParser(&prefix_no_atomic, "hello"),
            runParser(&prefix_no_atomic, "hellohi"),
        ];
        println!("{prefix_no_atomic_tests:?}");

        let prefix_with_atomic: Parser<'_, String, Maybe<String>> =
            alt::<Parser<'_, String, Maybe<String>>, _>(&atomic(string_("hi")), &string_("hello"));
        let prefix_with_atomic_tests: Vec<Maybe<String>> = vec![
            runParser(&prefix_with_atomic, "hello"),
            runParser(&prefix_with_atomic, "hellohi"),
        ];
        println!("{prefix_with_atomic_tests:?}");

        let hello_then_bang: Parser<'_, String, Maybe<String>> =
            then_keep_left::<Parser<'_, String, Maybe<String>>, _, _>(
                &string_("hello"),
                &char_('!'),
            );
        let bang_then_hello: Parser<'_, String, Maybe<String>> =
            then_keep_right::<Parser<'_, String, Maybe<String>>, _, _>(
                &char_('!'),
                &string_("hello"),
            );
        let then_tests: Vec<Maybe<String>> = vec![
            runParser(&hello_then_bang, "hello!"),
            runParser(&hello_then_bang, "hello"),
            runParser(&bang_then_hello, "!hello"),
            runParser(&bang_then_hello, "hello!"),
        ];
        println!("{then_tests:?}");

        const fn to_upper(c: char) -> char {
            c.to_ascii_uppercase()
        }
        const fn next_char(c: char) -> char {
            (c as u8 + 1) as char
        }
        let fmap_left: Parser<'_, char, Maybe<char>> = fmap::<Parser<'_, char, Maybe<char>>, _, _>(
            &fmap::<Parser<'_, char, Maybe<char>>, _, _>(&digit, to_upper),
            next_char,
        );
        let fmap_right: Parser<'_, char, Maybe<char>> =
            fmap::<Parser<'_, char, Maybe<char>>, _, _>(&digit, |c| next_char(to_upper(c)));
        let fmap_tests: Vec<Maybe<char>> =
            vec![runParser(&fmap_left, "1"), runParser(&fmap_right, "1")];
        println!("{fmap_tests:?}");
    }
}

fn main() {
    println!("-- List-of-results parser");
    parser_demo::run();

    println!("\n-- CPS Mini-Parsec");
    miniparsec_demo::run();

    println!("\n-- liftA2 on vector");
    let vec_liftA2_sum: Vec<i32> =
        liftA2::<Vec<()>, _, _, _>(&vec![1, 3, 4], &vec![2, 5, 6], |x, y| x + y);
    println!("{vec_liftA2_sum:?}");
}
