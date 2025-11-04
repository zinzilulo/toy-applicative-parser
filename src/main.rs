#![allow(non_snake_case)]

mod prelude;
use crate::prelude::{
    Alternative, Applicative, Just, Maybe, Nothing, alt, catMaybes, fmap, liftA2, many,
    then_keep_left, then_keep_right,
};

mod parser;
use crate::parser::*;

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

fn main() {
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

    let vec_liftA2_sum: Vec<i32> =
        liftA2::<Vec<()>, _, _, _>(&vec![1, 3, 4], &vec![2, 5, 6], |x, y| x + y);
    println!("{vec_liftA2_sum:?}");
}
