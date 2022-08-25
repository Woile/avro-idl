use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::prelude::*;
use core::fmt;

#[derive(Debug, Clone)]
enum Avdl {
    Protocol(String, Box<Avdl>),
    Empty,
    Comment,
}

impl fmt::Display for Avdl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Avdl::Protocol(name, expr) => write!(f, "protocol: \"{name}\", {expr}"),
            Avdl::Empty => write!(f, ""),
            Avdl::Comment => write!(f, ""),
        }
    }
}

fn parser() -> impl Parser<char, Avdl, Error = Simple<char>> {
    let ident = text::ident().padded();
    let comment = just("/*")
        .or_not()
        .then(take_until(just("*/")))
        .padded()
        .to(Avdl::Comment)
        .labelled("comment");

    let args = text::whitespace()
        .delimited_by(just('{'), just('}'))
        .to(Avdl::Empty)
        .labelled("content");

    let protocol = just("protocol")
        .ignore_then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("protocol name"),
        )
        .then(args)
        .padded()
        .map(|(name, body)| Avdl::Protocol(name.0, Box::new(body)))
        .labelled("protocol");

    // recursive(|value| {

    protocol
        .or(comment)
        .then_ignore(end().recover_with(skip_then_retry_until([])))
}
fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let parsed = parser().parse(src.trim());
    match parsed {
        Ok(ast) => {
            println!("{:?}", ast);
            // TODO: Add translation to json
        }
        Err(errs) => errs.into_iter().for_each(|e| {
            let msg = if let chumsky::error::SimpleReason::Custom(msg) = e.reason() {
                msg.clone()
            } else {
                format!(
                    "{}{}, expected {}",
                    if e.found().is_some() {
                        "Unexpected token"
                    } else {
                        "Unexpected end of input"
                    },
                    if let Some(label) = e.label() {
                        format!(" while parsing {}", label)
                    } else {
                        String::new()
                    },
                    if e.expected().len() == 0 {
                        "something else".to_string()
                    } else {
                        e.expected()
                            .map(|expected| match expected {
                                Some(expected) => expected.to_string(),
                                None => "end of input".to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    },
                )
            };

            let report = Report::build(ReportKind::Error, (), e.span().start)
                .with_code(3)
                .with_message(msg)
                .with_label(
                    Label::new(e.span())
                        .with_message(match e.reason() {
                            chumsky::error::SimpleReason::Custom(msg) => msg.clone(),
                            _ => format!(
                                "Unexpected {}",
                                e.found()
                                    .map(|c| format!("token {}", c.fg(Color::Red)))
                                    .unwrap_or_else(|| "end of input".to_string())
                            ),
                        })
                        .with_color(Color::Red),
                );

            let report = match e.reason() {
                chumsky::error::SimpleReason::Unclosed { span, delimiter } => report.with_label(
                    Label::new(span.clone())
                        .with_message(format!(
                            "Unclosed delimiter {}",
                            delimiter.fg(Color::Yellow)
                        ))
                        .with_color(Color::Yellow),
                ),
                chumsky::error::SimpleReason::Unexpected => report,
                chumsky::error::SimpleReason::Custom(_) => report,
            };

            report.finish().print(Source::from(&src)).unwrap();
        }),
    }
}
