use std::io::{self, Read, Write};
use std::process::exit;

fn main() {
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() {
        exit(1);
    }
    let parsed = cjson::parse_with_length_opts(&input, input.len().saturating_add(1), false);
    let Some(item) = parsed.item else {
        exit(2);
    };
    let Some(text) = cjson::print_unformatted(&item) else {
        exit(3);
    };
    if io::stdout().write_all(text.as_bytes()).is_err() {
        exit(1);
    }
}
