// #![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::correctness)]
// #![warn(clippy::deprecated)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::similar_names
)]
#![warn(clippy::perf)]
#![warn(clippy::style)]
#![warn(clippy::suspicious)]
use std::io::Write;

use ariadne::sources;

use ariadne::Color;

use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;

use chumsky::Parser;
use lexer::lexer;

use span::Span;

use crate::interpreter::exec;
use crate::parser::parser;

pub mod ast;
pub mod common;
pub mod interpreter;
pub mod lexer;
pub mod literal;
pub mod parser;
pub mod span;
// #[doc(hidden)]
// mod special_chars;
pub mod playground;
#[cfg(test)]
mod tests;

// const VVV: f64 = 3.1415;

pub use chumsky;
#[must_use]
pub fn lex(src: &str) -> (Option<Vec<Span<lexer::Token>>>, String) {
    let oe = lexer().parse(src).into_output_errors();

    let mut stdo = String::new();

    oe.1.clone()
        .into_iter()
        .map(|x: _| x.map_token(|c: _| c.to_string()))
        .for_each(|x: _| {
            Report::build(ReportKind::Error, "test.vea", x.span().start)
                // .with_config(Config::default().with_char_set(CharSet::Ascii))
                .with_message(x.to_string())
                .with_label(
                    Label::new(("test.vea", x.span().into_range()))
                        .with_message(x.reason().to_string())
                        .with_color(Color::Red),
                )
                .finish()
                .write_for_stdout(sources([("test.vea", src)]), unsafe { stdo.as_mut_vec() })
                .unwrap();
        });

    (oe.0, stdo)
}

#[must_use]
pub fn parse<'t>(
    src: &str,
    b: &[Span<lexer::Token<'t>>],
) -> (Option<Vec<Span<ast::Expr<'t>>>>, String) {
    let a = b.iter().map(|x| x.0.clone()).collect::<Vec<_>>();
    let p = parser().parse(a.as_slice()).into_output_errors();

    let mut stdo = String::new();

    p.1.clone()
        .into_iter()
        .map(|x: _| x.map_token(|c: _| format!("{c:?}")))
        .for_each(|x: _| {
            Report::build(ReportKind::Error, "test.vea", b[x.span().start].1.start)
                // .with_config(Config::default().with_char_set(CharSet::Ascii))
                .with_message(x.to_string())
                .with_label(
                    Label::new((
                        "test.vea",
                        b[x.span().start].1.start..b[x.span().end - 1].1.end,
                    ))
                    .with_message(x.reason().to_string())
                    .with_color(Color::Red),
                )
                .finish()
                .write_for_stdout(sources([("test.vea", src)]), unsafe { stdo.as_mut_vec() })
                .unwrap();
        });

    (p.0, stdo)
}

#[must_use]
pub fn interp(src: &str, t: &[Span<lexer::Token<'_>>], p: Vec<Span<ast::Expr>>) -> String {
    let mut env = interpreter::Env::default();
    let e = exec(p, &mut env);

    let mut stdo = String::new();

    if let Err(Span(x, y)) = &e {
        // e.into_iter().for_each(|(x, y)| {
        Report::build(ReportKind::Error, "test.vea", t[y.start].1.start)
            // .with_config(Config::default().with_char_set(CharSet::Ascii))
            .with_message(x.clone())
            .with_label(
                Label::new(("test.vea", t[y.start].1.start..t[y.end - 1].1.end))
                    .with_message(x)
                    .with_color(Color::Red),
            )
            .finish()
            .write(sources([("test.vea", src)]), unsafe { stdo.as_mut_vec() })
            .unwrap();
        // });
    } else if let Ok(p) = &e {
        write!(unsafe { stdo.as_mut_vec() }, "{}", p.stdout).unwrap();
    }

    stdo
}

#[must_use]
pub fn main() -> String {
    let src = "let x = 0; print(x);";
    let mut stdo = String::new();

    let oe = lex(src);

    stdo += &oe.1;

    if let Some(a) = oe.0.clone() {
        let p = parse(src, a.as_slice());

        stdo += &p.1;

        if let Some(p) = p.0.clone() {
            let m = interp(src, a.as_slice(), p);

            stdo += &m;
        }
    }

    stdo
}
