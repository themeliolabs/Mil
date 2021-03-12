use primitive_types::U256;
use crate::types::{BuiltIn, Atom, Expr};
use nom::{
    IResult, Parser,
    error::{context, ParseError},
    branch::alt,
    bytes::complete::tag,
    combinator::{map_res, map, map_opt},
    error::VerboseError,
    character::complete::{alpha1, multispace1, multispace0, one_of, digit1},
    multi::{separated_list1},
    character::complete::char,
    sequence::{preceded, delimited, separated_pair},
};

type ParseRes<'a, O> = IResult<&'a str, O, VerboseError<&'a str>>;

impl BuiltIn {
    /*
    fn token<'a>(&'a self) -> &'a str {
        match self {
            BuiltIn::Add => "+",
            BuiltIn::Sub => "-",
            BuiltIn::PushI => "pushI",
        }
    }
    */

    fn from_token(s: &str) -> Option<BuiltIn> {
        match s {
            "+" => Some(BuiltIn::Add),
            "-" => Some(BuiltIn::Sub),
            "pushI" => Some(BuiltIn::PushI),
            _ => None,
        }
    }
}

fn builtin<'a>(input: &'a str)
-> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    //map_opt(alt((map(one_of("+-"), |c| c.to_string().as_str()),
    context("builtin",
        map_opt(alt((tag("+"), tag("-"),
                    alpha1)),
                |s| BuiltIn::from_token(s)))
                //.map(BuiltIn::from_token)
            .parse(input)
}

fn int_atom<'a>(input: &'a str)
-> IResult<&'a str, Atom, VerboseError<&'a str>> {
    context("int",
        map_res(digit1, |n_str: &str| n_str.parse::<U256>())
            .map(Atom::Int))
            .parse(input)
    /*
    alt((map_res(digit1, |n_str: &str| n_str.parse::<U256>()
              .map(Atom::Int)),
        map_res(preceded(char('-'), digit1),
                |n_str: &str| n_str.parse::<U256>())
            .map(|n| -1*n)
            .map(Atom::Int)))
        (input)
        */
}

fn atom_expr<'a>(input: &'a str)
-> IResult<&'a str, Expr, VerboseError<&'a str>> {
    context("Atom",
        int_atom.map(Expr::Atom))
        .parse(input)
}

fn app_expr<'a>(input: &'a str)
-> IResult<&'a str, Expr, VerboseError<&'a str>> {
    //let identifier = alpha1;
    let args = separated_list1(multispace1, expr);
    let content = separated_pair(builtin,
                                 multispace1,
                                 args);

    context("App expr",
        ws(delimited(char('('),
                  content,
                  char(')')))
        .map(|(op, args)| Expr::App(op, args)))
        .parse(input)
}

pub fn expr<'a>(input: &'a str)
-> IResult<&'a str, Expr, VerboseError<&'a str>> {
    alt((atom_expr, app_expr))(input)
}

/// Surrounding whitespace parser combinator
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F)
-> impl Parser<&'a str, O, E>
//-> impl Fn(&'a str) -> IResult<&'a str, O, E>
  where
  F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
  delimited(
    multispace0,
    inner,
    multispace0
  )
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Debug;

    /// Map a parser over a list of tests and assert equality. Panics when equality fails.
    fn batch_test<'a, O: Debug + Eq>(
        mut f: impl Parser<&'a str, O, VerboseError<&'a str>>,
        tests: Vec<(&'a str, ParseRes<'a, O>)>)
    {
        tests.into_iter()
             .for_each(|(i,o)|
                 assert_eq!(f.parse(&i), o))
    }

    #[test]
    fn parse_int_as_atom() {
        let tests = vec![("10",   Ok(("", Atom::Int(10)))),
                         ("-742", Ok(("", Atom::Int(-742))))];

        batch_test(int_atom, tests)
    }

    #[test]
    fn parse_app_expr() {
        let tests = vec![
            ("(+ 1 2)",
             Ok(("", Expr::App(BuiltIn::Add,
                               vec![Expr::Atom(Atom::Int(1)),
                                    Expr::Atom(Atom::Int(2))])))),
            ("(+ 1 (- 5 -8))",
             Ok(("", Expr::App(BuiltIn::Add,
                               vec![Expr::Atom(Atom::Int(1)),
                                    Expr::App(BuiltIn::Sub,
                                              vec![Expr::Atom(Atom::Int(5)),
                                                   Expr::Atom(Atom::Int(-8))])]))))
        ];

        batch_test(app_expr, tests)
    }
}
