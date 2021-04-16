use std::fmt::Display;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hashbrown::HashMap;

type Code = &'static str;
type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
enum Error {
    // Encode(char),
    Decode(String),
    // Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Error::Encode(u) => write!(f, "unable to encode value: {:?}", u),
            Error::Decode(code) => write!(f, "unable to decode sequence: {:?}", code),
            // Error::Io(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

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

    pub static DECODING_ARRAY_2: &[Option<u8>] = &[
        None,
        Some(b'E'),
        Some(b'T'),
        Some(b'I'),
        Some(b'A'),
        Some(b'N'),
        Some(b'M'),
        Some(b'S'),
        Some(b'U'),
        Some(b'R'),
        Some(b'W'),
        Some(b'D'),
        Some(b'K'),
        Some(b'G'),
        Some(b'O'),
        Some(b'H'),
        Some(b'V'),
        Some(b'F'),
        None,
        Some(b'L'),
        None,
        Some(b'P'),
        Some(b'J'),
        Some(b'B'),
        Some(b'X'),
        Some(b'C'),
        Some(b'Y'),
        Some(b'Z'),
        Some(b'Q'),
        None,
        None,
        Some(b'5'),
        Some(b'4'),
        None,
        Some(b'3'),
        None,
        None,
        None,
        Some(b'2'),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(b'1'),
        Some(b'6'),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(b'7'),
        None,
        None,
        None,
        Some(b'8'),
        None,
        Some(b'9'),
        Some(b'0'),
    ];
}

/// The original decoding method involved a hashmap.
struct CharacterDecoder {
    map: HashMap<Code, char>,
}

impl CharacterDecoder {
    fn new() -> Self {
        // Looking back, I can't for the life of me figure
        // why I did this in two iterations.
        let letters = &data::ENCODED_SEQUENCES[..26];
        let letters = letters
            .iter()
            .copied()
            .zip("ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars());

        let numbers = &data::ENCODED_SEQUENCES[26..];
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
    let mut increment = 1 << 5;

    character.bytes().for_each(|u| {
        match u {
            b'.' => offset += increment,
            b'-' => offset -= increment,
            _ => (),
        }
        increment >>= 1;
    });

    offset
}

#[inline]
fn decode_character_heap(character: &str) -> Result<u8> {
    let idx = character_index(character);
    data::DECODING_ARRAY_2
        .get(idx as usize)
        .copied()
        .and_then(|x| x)
        .ok_or_else(|| Error::Decode(character.into()))
}

#[inline]
fn character_index(character: &str) -> i32 {
    character.bytes().fold(0, |idx, u| match u {
        b'.' => idx * 2 + 1,
        b'-' => idx * 2 + 2,
        _ => idx,
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    let decoder = CharacterDecoder::new();
    let sequences: Vec<_> = data::ENCODED_SEQUENCES
        .iter()
        .copied()
        .cycle()
        .take(1000)
        .collect();

    c.bench_function("map", |b| {
        b.iter(|| {
            for &character in &sequences {
                black_box(decoder.decode(character).unwrap());
            }
        })
    });

    c.bench_function("flat tree", |b| {
        b.iter(|| {
            for &character in &sequences {
                black_box(decode_character(character).unwrap());
            }
        })
    });

    c.bench_function("ftree 2.0", |b| {
        b.iter(|| {
            for &character in &sequences {
                black_box(decode_character_heap(character).unwrap());
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
