use std::sync::LazyLock;

use clap::ValueEnum;
use itertools::Itertools;
use rand::distr::Uniform;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;

// WORDS_LIST is a list of words to use for generating memorable passwords, which
// we directly embed in the executable.
//
// It is lazily initialized to avoid the cost of reading the wordlist from disk if it is not used
// in a given run of the program.
static WORDS_LIST: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    include_str!("../wordlist.txt")
        .lines()
        .filter(|l| l.len() >= 4)
        .collect::<Vec<&str>>()
});

/// Generates a memorable password with the given options.
///
/// This function creates a memorable password by choosing random words,
/// optionally scrambling them and/or capitalizing them, and then joining them
/// with the specified separator.
///
/// # Arguments
///
/// * `rng` - A mutable reference to a random number generator that implements the `Rng` trait
/// * `word_count` - The number of words to include in the password
/// * `separator` - The type of separator to use between words (see `Separator` enum)
/// * `capitalize` - Whether to capitalize the first letter of each word
/// * `scramble` - Whether to scramble the characters of each word
///
/// # Example
///
/// ```
/// use rand::thread_rng;
/// use motus::{Separator, memorable_password};
///
/// let rng = &mut thread_rng();
/// let word_count = 3;
/// let separator = Separator::Hyphen;
/// let capitalize = true;
/// let scramble = false;
///
/// let password = memorable_password(rng, word_count, separator, capitalize, scramble);
/// println!("Generated password: {}", password);
/// ```
///
/// # Panics
///
/// The function may panic in the event a word from the list the crate embeds were to contain
/// non-UTF-8 characters.
///
/// # Returns
///
/// A `String` containing the generated memorable password
#[allow(unstable_name_collisions)] // using itertools::intersperse_with until it is stabilized
pub fn memorable_password<R: Rng>(
    rng: &mut R,
    word_count: usize,
    separator: Separator,
    capitalize: bool,
    scramble: bool,
) -> String {
    // Get the random words and format them
    let formatted_words: Vec<String> = get_random_words(rng, word_count)
        .into_iter()
        .map(|word| {
            let mut word = word.to_string();

            // Scramble the word if requested
            if scramble {
                let mut bytes = word.clone().into_bytes();
                bytes.shuffle(rng);
                word = String::from_utf8(bytes).expect("random words should be valid UTF-8");
            }

            // Capitalize the word if requested
            if capitalize && let Some(first_letter) = word.get_mut(0..1) {
                first_letter.make_ascii_uppercase();
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
            .map(String::to_string)
            .intersperse_with(|| rng.random_range(0..10).to_string())
            .collect(),
        Separator::NumbersAndSymbols => {
            let numbers_and_symbols: Vec<char> = SYMBOL_CHARS
                .iter()
                .chain(NUMBER_CHARS.iter())
                .copied()
                .collect();
            formatted_words
                .iter()
                .map(String::to_string)
                .intersperse_with(|| {
                    numbers_and_symbols
                        .choose(rng)
                        .expect("numbers and symbols should have a length >= 1")
                        .to_string()
                })
                .collect()
        }
        Separator::None => formatted_words.join(""),
    }
}

/// Enum representing the various separators used to join words in a memorable password.
///
/// The `Separator` enum provides options for different types of separators that can be used
/// when generating a memorable password. These separators are used to join the words together
/// in the final password.
///
/// # Variants
///
/// * `Space` - Use a space character (' ') as the separator
/// * `Comma` - Use a comma character (',') as the separator
/// * `Hyphen` - Use a hyphen character ('-') as the separator
/// * `Period` - Use a period character ('.') as the separator
/// * `Underscore` - Use an underscore character ('_') as the separator
/// * `Numbers` - Use random numbers (0-9) as separators between words
/// * `NumbersAndSymbols` - Use a mix of random numbers (0-9) and symbols from the `SYMBOL_CHARS` const as separators between words
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Separator {
    Space,
    Comma,
    Hyphen,
    Period,
    Underscore,
    Numbers,
    NumbersAndSymbols,
    None,
}

/// Generates a random password with a specified length and optional inclusion of numbers and symbols.
///
/// This function creates a random password with the desired number of characters.
/// The generated password can include letters, numbers, and symbols based on the provided boolean flags.
///
/// # Arguments
///
/// * `rng: &mut R` - A mutable reference to a random number generator implementing the `Rng` trait
/// * `characters: u32` - The number of characters desired for the password
/// * `numbers: bool` - A flag indicating whether numbers should be included in the password
/// * `symbols: bool` - A flag indicating whether symbols should be included in the password
///
/// # Panics
///
/// The function may panic in the event that the provided `characters` argument is 0.
///
/// # Returns
///
/// * `String` - The generated random password
///
/// # Examples
///
/// ```
/// use rand::thread_rng;
/// use motus::random_password;
///
/// let mut rng = thread_rng();
/// let password = random_password(&mut rng, 12, true, true);
/// assert_eq!(password.len(), 12);
/// ```
pub fn random_password<R: Rng>(
    rng: &mut R,
    characters: u32,
    numbers: bool,
    symbols: bool,
) -> String {
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
        (true, false) | (false, true) => vec![8, 2],

        // Otherwise we want to make sure that we apply the following distribution: 100% letters.
        (false, false) => vec![10],
    };

    let dist_set = WeightedIndex::new(weights).expect("weights should be valid");
    let mut password = String::with_capacity(characters as usize);

    for _ in 0..characters {
        let selected_set = available_sets
            .get(dist_set.sample(rng))
            .expect("index should be valid");
        let dist_char =
            Uniform::new(0, selected_set.len()).expect("failed to create uniform distribution");
        let index = dist_char.sample(rng);
        password.push(selected_set[index]);
    }

    password
}

/// Generates a random numeric PIN with a specified length.
///
/// This function creates a random PIN with the desired number of digits.
///
/// # Arguments
///
/// * `rng: &mut R` - A mutable reference to a random number generator implementing the `Rng` trait
/// * `numbers: u32` - The number of digits desired for the PIN
///
/// # Returns
///
/// * `String` - The generated random numeric PIN
///
/// # Examples
///
/// ```
/// use rand::thread_rng;
/// use motus::pin_password;
///
/// let mut rng = thread_rng();
/// let pin = pin_password(&mut rng, 4);
/// assert_eq!(pin.len(), 4);
/// assert!(pin.chars().all(|c| c.is_digit(10)));
/// ```
pub fn pin_password<R: Rng>(rng: &mut R, numbers: u32) -> String {
    (0..numbers)
        .map(|_| NUMBER_CHARS[rng.random_range(0..NUMBER_CHARS.len())])
        .collect()
}

// LETTER_CHARS is a list of letters that can be used in passwords
const LETTER_CHARS: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

// NUMBER_CHARS is a list of numbers that can be used in passwords
const NUMBER_CHARS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

// SYMBOL_CHARS is a list of symbols that can be used in passwords
const SYMBOL_CHARS: &[char] = &['!', '@', '#', '$', '%', '^', '&', '*', '(', ')'];

// get_random_words returns a vector of n random words from the word list
fn get_random_words<R: Rng>(rng: &mut R, n: usize) -> Vec<&'static str> {
    WORDS_LIST.choose_multiple(rng, n).copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memorable_password() {
        let seed = 42; // Fixed seed for predictable randomness
        let mut rng = StdRng::seed_from_u64(seed);

        let password = memorable_password(&mut rng, 4, Separator::Space, false, false);
        assert_eq!(password, "choking natural dolly ominous");

        let password = memorable_password(&mut rng, 4, Separator::Comma, false, false);
        assert_eq!(password, "thrive,punctured,wool,hardcover");

        let password = memorable_password(&mut rng, 4, Separator::Hyphen, true, false);
        assert_eq!(password, "Violate-Applause-Preorder-Headstone");

        let password = memorable_password(&mut rng, 4, Separator::Numbers, true, true);
        assert_eq!(password, "Taunnfoi8Causerl9Ocrrwab5Stpwe");

        let password = memorable_password(&mut rng, 4, Separator::None, false, false);
        assert_eq!(password, "molecularthirstinggroundrubber");
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
        assert!(
            password_numbers_symbols
                .chars()
                .any(|c| NUMBER_CHARS.contains(&c) || SYMBOL_CHARS.contains(&c))
        );
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
    fn test_get_random_words() {
        let seed = 42; // Fixed seed for predictable randomness
        let mut rng = StdRng::seed_from_u64(seed);

        let words = get_random_words(&mut rng, 5);

        // Note that the expected word list is fixed as we provide a fixed
        // random seed. If you change the seed, you should change the expected
        // word list.
        assert_eq!(
            words,
            vec!["chokehold", "nativity", "dolly", "ominous", "throat"]
        );
    }
}
