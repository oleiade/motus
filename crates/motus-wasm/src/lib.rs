use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn memorable_password(
    word_count: usize,
    separator: Separator,
    capitalize: bool,
    scramble: bool,
) -> String {
    let mut rng = rand::thread_rng();
    motus::memorable_password(&mut rng, word_count, separator.into(), capitalize, scramble)
}

#[wasm_bindgen]
pub fn random_password(characters: u32, numbers: bool, symbols: bool) -> String {
    let mut rng = rand::thread_rng();
    motus::random_password(&mut rng, characters, numbers, symbols)
}

#[wasm_bindgen]
pub fn pin_password(numbers: u32) -> String {
    let mut rng = rand::thread_rng();
    motus::pin_password(&mut rng, numbers)
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Separator {
    Space,
    Comma,
    Hyphen,
    Period,
    Underscore,
    Numbers,
    NumbersAndSymbols,
}

#[allow(clippy::from_over_into)]
impl Into<motus::Separator> for Separator {
    fn into(self) -> motus::Separator {
        match self {
            Separator::Space => motus::Separator::Space,
            Separator::Comma => motus::Separator::Comma,
            Separator::Hyphen => motus::Separator::Hyphen,
            Separator::Period => motus::Separator::Period,
            Separator::Underscore => motus::Separator::Underscore,
            Separator::Numbers => motus::Separator::Numbers,
            Separator::NumbersAndSymbols => motus::Separator::NumbersAndSymbols,
        }
    }
}
