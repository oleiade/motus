#![feature(iter_intersperse)]

use std::sync::Arc;

use clap::{Parser, Subcommand, ValueEnum};
// use copypasta::{ClipboardContext, ClipboardProvider};
use arboard::Clipboard;
use human_panic::setup_panic;
use lazy_static::lazy_static;
use rand::distributions::{Uniform, WeightedIndex};
use rand::prelude::*;
use rand::seq::SliceRandom;

/// Args is a struct representing the command line arguments
#[derive(Parser, Debug)]
#[command(name = "motus")]
#[command(version = "0.1.0")]
#[command(about = "A command-line tool to generate secure passwords")]
#[command(
    long_about = "Motus is a command-line tool for generating secure, random, and memorable passwords as well as PIN codes."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Disable automatic copying of generated password to clipboard
    #[arg(long)]
    no_clipboard: bool,

    /// Seed value for deterministic password generation (for testing purposes)
    #[arg(long)]
    seed: Option<u64>, // Set the randomness source with an unsigned 64-bit integer for reproducible passwords
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(name = "memorable")]
    #[command(about = "Generate a human-friendly memorable password")]
    #[command(
        long_about = "Generate a memorable password using a combination of words and configurable separators, with optional capitalization and the choice to use unrecognizable words."
    )]
    Memorable {
        /// Specify the number of words in the generated password
        #[arg(short, long, default_value = "5", value_parser = validate_word_count)]
        words: u32,

        /// Choose the separator for words in the generated password
        #[arg(short, long, default_value = "space", value_enum)]
        separator: Separator,

        /// Enable capitalization of each word in the generated password
        #[arg(short, long)]
        capitalize: bool,

        /// Enable the use of unrecognizable words in the generated password
        #[arg(long)]
        no_full_words: bool,
    },

    #[command(name = "random")]
    #[command(about = "Generate a random password with specified complexity")]
    #[command(
        long_about = "Generate a random password with a configurable number of characters, including optional numbers and symbols for increased complexity."
    )]
    Random {
        /// Specify the number of characters in the generated password
        #[arg(short, long, default_value = "20", value_parser = validate_character_count)]
        characters: u32,

        /// Enable the inclusion of numbers in the generated password
        #[arg(short, long)]
        numbers: bool,

        /// Enable the inclusion of symbols in the generated password
        #[arg(short, long)]
        symbols: bool,
    },

    #[command(name = "pin")]
    #[command(about = "Generate a random numeric PIN code")]
    #[command(
        long_about = "Generate a random numeric Personal Identification Number (PIN) code with a configurable length."
    )]
    Pin {
        /// Specify the number of digits in the generated PIN code
        #[arg(short, long, default_value = "7", value_parser = validate_pin_length)]
        numbers: u32,
    },
}

/// Separator is an enum of possible separators for words in the password
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Separator {
    Space,
    Comma,
    Hyphen,
    Period,
    Underscore,
    Numbers,
    NumbersAndSymbols,
}

/// Include the word list file content as a static string
const WORDS: &str = include_str!("../wordlist.txt");

// Use lazy_static to create an indexed version of the list
lazy_static! {
    static ref WORDS_LIST: Arc<Vec<&'static str>> = {
        let words = WORDS
            .lines()
            .filter(|l| l.len() >= 4)
            .collect::<Vec<&str>>();
        Arc::new(words)
    };
}

fn main() {
    // Enable human-readable panic messages
    setup_panic!();

    // Parse command line arguments
    let opts: Cli = Cli::parse();

    // Initialize the randomness source
    // If a seed is provided, use it to seed the randomness source
    // Otherwise, use the main thread's randomness source
    let mut rng: Box<dyn RngCore> = match opts.seed {
        Some(seed) => Box::new(StdRng::seed_from_u64(seed)),
        None => Box::new(rand::thread_rng()),
    };

    let password = match opts.command {
        Commands::Memorable {
            words,
            separator,
            capitalize,
            no_full_words,
        } => memorable_password(
            &mut rng,
            words as usize,
            separator,
            capitalize,
            no_full_words,
        ),
        Commands::Random {
            characters,
            numbers,
            symbols,
        } => random_password(&mut rng, characters, numbers, symbols),
        Commands::Pin { numbers } => pin_password(&mut rng, numbers),
    };

    // Copy the password to the clipboard
    if !opts.no_clipboard {
        let mut clipboard =
            Clipboard::new().expect("unable to interact with your system's clipboard");
        clipboard
            .set_text(&password)
            .expect("unable to set clipboard contents");
    }

    print!("{}", password);
}

/// memorable_password generates a password with the given number of words, separated by the given
/// separator.
///
/// If capitalize is true, each word will be capitalized.
/// If scramble is true, the words will be scrambled before being joined.
fn memorable_password<R: Rng>(
    rng: &mut R,
    word_count: usize,
    separator: Separator,
    capitalize: bool,
    scramble: bool,
) -> String {
    // Get the random words and format them
    let formatted_words: Vec<String> = get_random_words(rng, word_count)
        .iter()
        .map(|word| {
            let mut word = word.to_string();

            // Scramble the word if requested
            if scramble {
                let mut bytes = word.to_string().into_bytes();
                bytes.shuffle(rng);
                word =
                    String::from_utf8(bytes.to_vec()).expect("random words should be valid UTF-8");
            }

            // Capitalize the word if requested
            if capitalize {
                if let Some(first_letter) = word.get_mut(0..1) {
                    first_letter.make_ascii_uppercase();
                }
            }
            word
        })
        .collect();

    // Join the formatted words with the separator
    match separator {
        Separator::Space => formatted_words.join(" "),
        Separator::Comma => formatted_words.join(","),
        Separator::Hyphen => formatted_words.join("-"),
        Separator::Period => formatted_words.join("."),
        Separator::Underscore => formatted_words.join("_"),
        Separator::Numbers => formatted_words
            .iter()
            .map(|s| s.to_string())
            .intersperse_with(|| rng.gen_range(0..10).to_string())
            .collect(),
        Separator::NumbersAndSymbols => {
            let numbers_and_symbols: Vec<char> = SYMBOL_CHARS
                .iter()
                .chain(NUMBER_CHARS.iter())
                .cloned()
                .collect();
            formatted_words
                .iter()
                .map(|s| s.to_string())
                .intersperse_with(|| {
                    numbers_and_symbols
                        .choose(rng)
                        .expect("numbers and symbols should have a length >= 1")
                        .to_string()
                })
                .collect()
        }
    }
}

/// random_password generates a password with the given number of characters, using the given
/// character sets.
fn random_password<R: Rng>(rng: &mut R, characters: u32, numbers: bool, symbols: bool) -> String {
    let mut available_sets = vec![LETTER_CHARS];

    if numbers {
        available_sets.push(NUMBER_CHARS);
    }

    if symbols {
        available_sets.push(SYMBOL_CHARS);
    }

    let weights: Vec<u32> = match (numbers, symbols) {
        // If numbers and symbols are both true, we want to make sure that
        // we apply the following distribution: 70% letters, 20% numbers, 10% symbols.
        (true, true) => vec![7, 2, 1],

        // If either numbers or symbols is true, but not the other, we want
        // to make sure that we apply the following distribution: 80% letters, 20% numbers.
        (true, false) => vec![8, 2],
        (false, true) => vec![8, 2],

        // Otherwise we want to make sure that we apply the following distribution: 100% letters.
        (false, false) => vec![10],
    };

    let dist_set = WeightedIndex::new(&weights).expect("weights should be valid");
    let mut password = String::with_capacity(characters as usize);

    for _ in 0..characters {
        let selected_set = available_sets
            .get(dist_set.sample(rng))
            .expect("index should be valid");
        let dist_char = Uniform::from(0..selected_set.len());
        let index = dist_char.sample(rng);
        password.push(selected_set[index]);
    }

    password
}

/// pin_password generates a PIN with the given number of numbers.
fn pin_password<R: Rng>(rng: &mut R, numbers: u32) -> String {
    (0..numbers)
        .map(|_| NUMBER_CHARS[rng.gen_range(0..NUMBER_CHARS.len())])
        .collect()
}

/// LETTER_CHARS is a list of letters that can be used in passwords
const LETTER_CHARS: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

/// NUMBER_CHARS is a list of numbers that can be used in passwords
const NUMBER_CHARS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

/// SYMBOL_CHARS is a list of symbols that can be used in passwords
const SYMBOL_CHARS: &[char] = &['!', '@', '#', '$', '%', '^', '&', '*', '(', ')'];

/// validate_word_count parses the given string as a u32 and returns an error if it is not between
/// 3 and 15.
fn validate_word_count(s: &str) -> Result<u32, String> {
    match s.parse::<u32>() {
        Ok(n) if (3..16).contains(&n) => Ok(n),
        Ok(_) => Err("The number of words must be between 4 and 15".to_string()),
        Err(_) => Err("The number of words must be an integer".to_string()),
    }
}

/// validate_character_count parses the given string as a u32 and returns an error if it is not between
/// 8 and 100.
fn validate_character_count(s: &str) -> Result<u32, String> {
    match s.parse::<u32>() {
        Ok(n) if (8..101).contains(&n) => Ok(n),
        Ok(_) => Err("The number of words must be between 8 and 100".to_string()),
        Err(_) => Err("The number of words must be an integer".to_string()),
    }
}

/// validate_ping_length parses the given string as a u32 and returns an error if it is not between
/// 3 and 12.
fn validate_pin_length(s: &str) -> Result<u32, String> {
    match s.parse::<u32>() {
        Ok(n) if (3..13).contains(&n) => Ok(n),
        Ok(_) => Err("The number of words must be between 3 and 12".to_string()),
        Err(_) => Err("The number of words must be an integer".to_string()),
    }
}

/// get_random_words returns a vector of n random words from the word list
fn get_random_words<R: Rng>(rng: &mut R, n: usize) -> Vec<&'static str> {
    WORDS_LIST.choose_multiple(rng, n).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memorable_password() {
        let seed = 42; // Fixed seed for predictable randomness
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let password = memorable_password(&mut rng, 4, Separator::Space, false, false);
        assert_eq!(password, "chairmen manacle discouraging metallurgy");

        let password = memorable_password(&mut rng, 4, Separator::Comma, false, false);
        assert_eq!(password, "sweetheart,woad,headstock,vouchers");

        let password = memorable_password(&mut rng, 4, Separator::Hyphen, true, false);
        assert_eq!(password, "Anomic-Parallelepiped-Hiring-Forcefully");

        let password = memorable_password(&mut rng, 4, Separator::Numbers, true, true);
        assert_eq!(password, "Sveratoitre3Rgionitndoca4Erpitrs0Ipntanocet");
    }

    #[test]
    fn test_random_password_length() {
        let mut rng = StdRng::seed_from_u64(0);
        let length = 12;
        let password = random_password(&mut rng, length, true, true);
        assert_eq!(password.len(), length as usize);
    }

    #[test]
    fn test_random_password_content() {
        let mut rng = StdRng::seed_from_u64(0);
        let length = 12;

        let password_letters = random_password(&mut rng, length, false, false);
        assert!(password_letters.chars().all(|c| LETTER_CHARS.contains(&c)));

        let password_numbers = random_password(&mut rng, length, true, false);
        assert!(password_numbers.chars().any(|c| NUMBER_CHARS.contains(&c)));

        let password_symbols = random_password(&mut rng, length, false, true);
        assert!(password_symbols.chars().any(|c| SYMBOL_CHARS.contains(&c)));

        let password_numbers_symbols = random_password(&mut rng, length, true, true);
        assert!(password_numbers_symbols
            .chars()
            .any(|c| NUMBER_CHARS.contains(&c) || SYMBOL_CHARS.contains(&c)));
    }

    #[test]
    fn test_random_password_different_seeds() {
        let mut rng1 = StdRng::seed_from_u64(0);
        let mut rng2 = StdRng::seed_from_u64(1);
        let length = 12;
        let password1 = random_password(&mut rng1, length, true, true);
        let password2 = random_password(&mut rng2, length, true, true);
        assert_ne!(password1, password2);
    }

    #[test]
    fn test_pin_password_length() {
        let mut rng = StdRng::seed_from_u64(0);
        let pin_length = 6;
        let pin = pin_password(&mut rng, pin_length);
        assert_eq!(pin.len(), pin_length as usize);
    }

    #[test]
    fn test_pin_password_content() {
        let mut rng = StdRng::seed_from_u64(0);
        let pin_length = 6;
        let pin = pin_password(&mut rng, pin_length);
        assert!(pin.chars().all(|c| NUMBER_CHARS.contains(&c)));
    }

    #[test]
    fn test_pin_password_different_seeds() {
        let mut rng1 = StdRng::seed_from_u64(0);
        let mut rng2 = StdRng::seed_from_u64(1);
        let pin_length = 6;
        let pin1 = pin_password(&mut rng1, pin_length);
        let pin2 = pin_password(&mut rng2, pin_length);
        assert_ne!(pin1, pin2);
    }

    #[test]
    fn test_validate_word_count() {
        assert!(validate_word_count("2").is_err());
        assert!(validate_word_count("3").is_ok());
        assert!(validate_word_count("15").is_ok());
        assert!(validate_word_count("16").is_err());
    }

    #[test]
    fn test_get_random_words() {
        let seed = 42; // Fixed seed for predictable randomness
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let words = get_random_words(&mut rng, 5);

        // Note that the expected word list is fixed as we provide a fixed
        // random seed. If you change the seed, you should change the expected
        // word list.
        assert_eq!(
            words,
            vec![
                "chairmen",
                "mammy",
                "discouraging",
                "metallurgy",
                "petticoats"
            ]
        );
    }
}
