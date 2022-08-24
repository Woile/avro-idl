use core::fmt;

use chumsky::prelude::*;

#[derive(Debug, Clone)]
enum Avdl {
    Protocol(String, Box<Avdl>),
    Empty
}

impl fmt::Display for Avdl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Avdl::Protocol(name, expr) => write!(f, "protocol: \"{name}\""),
            Avdl::Empty => write!(f, ""),
        }
    }
}

fn parser() -> impl Parser<char, Avdl, Error = Simple<char>> {
    let ident = text::ident().padded();

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
    .map(|(name, body)| Avdl::Protocol(name.0, Box::new(body))).labelled("protocol");

    // recursive(|value| {

    protocol.then_ignore(end().recover_with(skip_then_retry_until([])))
}
fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let parsed = parser().parse(src);
    match parsed {
        Ok(ast) => {
            println!("{:?}", ast);
            // TODO: Add translation to json
        },
        Err(parse_errs) => parse_errs
            .into_iter()
            .for_each(|e| println!("Parse error: {}", e)),
    }

}
