use std::{
    fmt::Display,
    io::{self, Read},
    ops::RangeInclusive,
    process,
};

use clap::Clap;
use hashbrown::HashMap;

type Code = &'static str;
type Result<T, E = Error> = std::result::Result<T, E>;

static CODE: &[&str] = &[
    ".-", "-...", "-.-.", "-..", ".", "..-.", "--.", "....", "..", ".---", "-.-", ".-..", "--",
    "-.", "---", ".--.", "--.-", ".-.", "...", "-", "..-", "...-", ".--", "-..-", "-.--", "--..",
    "-----", ".----", "..---", "...--", "....-", ".....", "-....", "--...", "---..", "----.",
];

#[derive(Clap, Clone)]
enum Opts {
    Encode,
    Decode,
}

#[derive(Debug)]
enum Error {
    Encode(char),
    Decode(String),
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Encode(u) => write!(f, "unable to encode value: {:?}", u),
            Error::Decode(code) => write!(f, "unable to decode sequence: {:?}", code),
            Error::Io(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

struct CharacterDecoder {
    map: HashMap<Code, char>,
}

impl CharacterDecoder {
    fn new() -> Self {
        // Looking back, I can't for the life of me figure
        // why I did this in two iterations.
        let letters = &CODE[..26];
        let letters = letters
            .iter()
            .copied()
            .zip("ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars());

        let numbers = &CODE[26..];
        let numbers = numbers.iter().copied().zip("0123456789".chars());

        Self {
            map: letters.chain(numbers).collect(),
        }
    }

    #[inline]
    fn decode(&self, character: &str) -> Result<char> {
        self.map
            .get(character)
            .copied()
            .ok_or_else(|| Error::Decode(character.to_string()))
    }
}

fn main() {
    let opts = Opts::parse();
    if let Err(e) = run(&opts) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run(opts: &Opts) -> Result<()> {
    let mut buf = String::new();
    let message = io::stdin()
        .read_to_string(&mut buf)
        .map(|_| buf)
        .map_err(Error::Io)?;

    match opts {
        Opts::Encode => {
            let message: String = message
                .trim()
                .bytes()
                .filter(|&u| u == b' ' || u.is_ascii_alphanumeric())
                .map(|u| u as char)
                .collect();
            println!("{}", encode_message(&message)?);
        }

        Opts::Decode => {
            let character_decoder = CharacterDecoder::new();
            println!("{}", decode_message(message.trim(), &character_decoder)?);
        }
    }

    Ok(())
}

fn encode_message(message: &str) -> Result<String> {
    let mut buf = String::with_capacity(message.len() * 4);
    let mut bytes = message.bytes();

    if let Some(u) = bytes.next() {
        buf.push_str(encode_byte(u)?);
    }

    for u in bytes {
        match u {
            b' ' => buf.push_str(" /"),
            u => {
                buf.push(' ');
                buf.push_str(encode_byte(u)?);
            }
        }
    }

    Ok(buf)
}

fn decode_message(message: &str, character_decoder: &CharacterDecoder) -> Result<String> {
    let mut buf = String::new();
    let mut words = message.split('/');

    if let Some(word) = words.next() {
        decode_word_into(word, &mut buf, character_decoder)?;
    }

    for word in words {
        buf.push(' ');
        decode_word_into(word, &mut buf, character_decoder)?;
    }

    Ok(buf)
}

#[inline]
fn encode_byte(u: u8) -> Result<Code> {
    static NUMERIC_RANGE: RangeInclusive<u8> = b'0'..=b'9';
    match u {
        u if u.is_ascii_alphabetic() => Ok(CODE[(u.to_ascii_uppercase() - b'A') as usize]),
        u if NUMERIC_RANGE.contains(&u) => Ok(CODE[(u - b'0' + 26) as usize]),
        _ => Err(Error::Encode(u as char)),
    }
}

fn decode_word_into(
    word: &str,
    buf: &mut String,
    character_decoder: &CharacterDecoder,
) -> Result<()> {
    let mut characters = word.split_whitespace();

    if let Some(character) = characters.next() {
        buf.push(character_decoder.decode(character)?);
    }

    for character in characters {
        buf.push(character_decoder.decode(character)?);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn char_to_code_works() {
        let sequence = "abcdefghijklmnopqrstuvwxyz0123456789";
        let pairs = sequence.bytes().zip(super::CODE);

        for (u, &code) in pairs {
            assert_eq!(super::encode_byte(u).unwrap(), code);
        }
    }
}
