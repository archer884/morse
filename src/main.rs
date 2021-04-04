use std::{
    fmt::Display,
    io::{self, Read},
    ops::RangeInclusive,
    process,
};

use clap::Clap;

type Code = &'static str;
type Result<T, E = Error> = std::result::Result<T, E>;

mod data {
    pub static ENCODED_SEQUENCES: &[&str] = &[
        ".-", "-...", "-.-.", "-..", ".", "..-.", "--.", "....", "..", ".---", "-.-", ".-..", "--",
        "-.", "---", ".--.", "--.-", ".-.", "...", "-", "..-", "...-", ".--", "-..-", "-.--",
        "--..", "-----", ".----", "..---", "...--", "....-", ".....", "-....", "--...", "---..",
        "----.",
    ];

    pub static DECODING_ARRAY: &[Option<u8>] = &[
        Some(b'0'),
        None,
        None,
        None,
        Some(b'9'),
        None,
        Some(b'O'),
        None,
        None,
        None,
        None,
        None,
        Some(b'8'),
        None,
        Some(b'M'),
        None,
        None,
        None,
        Some(b'Q'),
        None,
        None,
        None,
        Some(b'G'),
        None,
        None,
        None,
        Some(b'Z'),
        None,
        Some(b'7'),
        None,
        Some(b'T'),
        None,
        None,
        None,
        Some(b'Y'),
        None,
        None,
        None,
        Some(b'K'),
        None,
        None,
        None,
        Some(b'C'),
        None,
        None,
        None,
        Some(b'N'),
        None,
        None,
        None,
        Some(b'X'),
        None,
        None,
        None,
        Some(b'D'),
        None,
        None,
        None,
        Some(b'B'),
        None,
        Some(b'6'),
        None,
        None,
        None,
        Some(b'1'),
        None,
        Some(b'J'),
        None,
        None,
        None,
        Some(b'W'),
        None,
        None,
        None,
        Some(b'P'),
        None,
        None,
        None,
        Some(b'A'),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(b'R'),
        None,
        None,
        None,
        Some(b'L'),
        None,
        None,
        None,
        Some(b'E'),
        None,
        Some(b'2'),
        None,
        None,
        None,
        None,
        None,
        Some(b'U'),
        None,
        None,
        None,
        Some(b'F'),
        None,
        None,
        None,
        Some(b'I'),
        None,
        Some(b'3'),
        None,
        Some(b'V'),
        None,
        None,
        None,
        Some(b'S'),
        None,
        Some(b'4'),
        None,
        Some(b'H'),
        None,
        Some(b'5'),
    ];
}

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
            println!("{}", decode_message(message.trim())?);
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

fn decode_message(message: &str) -> Result<String> {
    let mut buf = String::new();
    let mut words = message.split('/');

    if let Some(word) = words.next() {
        decode_word_into(word, &mut buf)?;
    }

    for word in words {
        buf.push(' ');
        decode_word_into(word, &mut buf)?;
    }

    Ok(buf)
}

#[inline]
fn encode_byte(u: u8) -> Result<Code> {
    static NUMERIC_RANGE: RangeInclusive<u8> = b'0'..=b'9';
    match u {
        u if u.is_ascii_alphabetic() => {
            Ok(data::ENCODED_SEQUENCES[(u.to_ascii_uppercase() - b'A') as usize])
        }
        u if NUMERIC_RANGE.contains(&u) => Ok(data::ENCODED_SEQUENCES[(u - b'0' + 26) as usize]),
        _ => Err(Error::Encode(u as char)),
    }
}

fn decode_word_into(word: &str, buf: &mut String) -> Result<()> {
    let mut characters = word.split_whitespace();

    if let Some(character) = characters.next() {
        buf.push(decode_character(character)? as char);
    }

    for character in characters {
        buf.push(decode_character(character)? as char);
    }

    Ok(())
}

#[inline]
fn decode_character(character: &str) -> Result<u8> {
    /// Correction factor to raise minimum index to zero.
    const MAGIC_NUMBER: i32 = 62;

    let idx = uncorrected_offset(character) + MAGIC_NUMBER;
    data::DECODING_ARRAY
        .get(idx as usize)
        .copied()
        .and_then(|x| x)
        .ok_or_else(|| Error::Decode(character.into()))
}

#[inline]
fn uncorrected_offset(character: &str) -> i32 {
    let mut offset = 0;
    let mut increment = 32;

    character.bytes().for_each(|u| {
        match u {
            b'.' => offset += increment,
            b'-' => offset -= increment,
            _ => (),
        }
        increment /= 2;
    });

    offset
}

#[cfg(test)]
mod tests {
    #[test]
    fn char_to_code_works() {
        let sequence = "abcdefghijklmnopqrstuvwxyz0123456789";
        let pairs = sequence.bytes().zip(super::data::ENCODED_SEQUENCES);

        for (u, &code) in pairs {
            assert_eq!(super::encode_byte(u).unwrap(), code);
        }
    }
}
