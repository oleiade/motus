#![feature(iter_intersperse)]

use arboard::Clipboard;
use clap::{Parser, Subcommand};
use human_panic::setup_panic;
use rand::prelude::*;

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
        separator: motus::Separator,

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
        } => motus::memorable_password(
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
        } => motus::random_password(&mut rng, characters, numbers, symbols),
        Commands::Pin { numbers } => motus::pin_password(&mut rng, numbers),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_word_count() {
        assert!(validate_word_count("2").is_err());
        assert!(validate_word_count("3").is_ok());
        assert!(validate_word_count("15").is_ok());
        assert!(validate_word_count("16").is_err());
    }

    #[test]
    fn test_validate_character_count() {
        assert!(validate_character_count("7").is_err());
        assert!(validate_character_count("8").is_ok());
        assert!(validate_character_count("100").is_ok());
        assert!(validate_character_count("101").is_err());
    }

    #[test]
    fn test_validate_pin_length() {
        assert!(validate_pin_length("2").is_err());
        assert!(validate_pin_length("3").is_ok());
        assert!(validate_pin_length("12").is_ok());
        assert!(validate_pin_length("13").is_err());
    }
}
