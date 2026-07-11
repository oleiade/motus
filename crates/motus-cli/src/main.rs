use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[cfg(feature = "clipboard")]
use arboard::Clipboard;
use clap::{Parser, Subcommand, ValueEnum};
use colored::{ColoredString, Colorize};
use human_panic::setup_panic;
use rand::prelude::*;
use rand::rng as thread_rng;
use serde::Serialize;
use serde::ser::{SerializeStruct, Serializer};
use term_table::row::Row;
use term_table::table_cell::{Alignment, TableCell};
use term_table::{Table, TableStyle, row, rows};
use zxcvbn::zxcvbn;

/// Args is a struct representing the command line arguments
#[derive(Parser, Debug)]
#[command(name = "motus")]
#[command(version = "0.3.1")]
#[command(about = "A command-line tool to generate secure passwords")]
#[command(
    long_about = "Motus is a command-line tool for generating secure, random, and memorable passwords as well as PIN codes."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Disable automatic copying of generated password to clipboard
    #[cfg(feature = "clipboard")]
    #[arg(long)]
    no_clipboard: bool,

    /// Output the generated password in a specified format
    #[arg(short, long, default_value = "text", value_enum)]
    output: OutputFormat,

    /// Display a safety analysis along the generated password
    #[arg(long)]
    analyze: bool,

    /// Seed value for deterministic password generation (for testing purposes).
    ///
    /// A seeded password is fully determined by this value and is therefore
    /// predictable. This flag is compiled in only under the `insecure-seed`
    /// feature (used by the test suite) and is absent from release builds.
    #[cfg(feature = "insecure-seed")]
    #[arg(long)]
    seed: Option<u64>,
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

    /// Internal: hold the clipboard contents until they are pasted or a
    /// timeout elapses.
    ///
    /// On Linux, the clipboard drops its contents when the process that set
    /// them exits. `copy_to_clipboard` spawns this hidden subcommand as a
    /// short-lived background holder; the password travels over stdin so it
    /// never appears in the process arguments.
    #[cfg(all(target_os = "linux", feature = "clipboard"))]
    #[command(name = "hold-clipboard", hide = true)]
    HoldClipboard,
}

fn main() {
    // Enable human-readable panic messages
    setup_panic!();

    // Parse command line arguments
    let opts: Cli = Cli::parse();

    // On Linux the clipboard is held by a detached child process (see
    // copy_to_clipboard); this hidden subcommand is that child.
    #[cfg(all(target_os = "linux", feature = "clipboard"))]
    if matches!(opts.command, Commands::HoldClipboard) {
        if let Err(e) = hold_clipboard() {
            eprintln!("Error: failed to hold the clipboard: {e}");
            std::process::exit(1);
        }
        return;
    }

    // Initialize the randomness source.
    //
    // Release builds always use the operating system's CSPRNG. The optional
    // `insecure-seed` feature (enabled for the test suite only) allows seeding a
    // deterministic generator so tests can assert on reproducible output.
    #[cfg(feature = "insecure-seed")]
    let seed = opts.seed;
    #[cfg(not(feature = "insecure-seed"))]
    let seed: Option<u64> = None;

    let mut rng: Box<dyn Rng> = match seed {
        Some(seed) => Box::new(StdRng::seed_from_u64(seed)),
        None => Box::new(thread_rng()),
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
        #[cfg(all(target_os = "linux", feature = "clipboard"))]
        Commands::HoldClipboard => unreachable!("handled right after argument parsing"),
    };

    // Copy the password to the clipboard, excluding it from OS clipboard
    // history and cloud sync wherever the platform supports it.
    #[cfg(feature = "clipboard")]
    {
        if !opts.no_clipboard
            && let Err(e) = copy_to_clipboard(&password)
        {
            eprintln!(
                "Warning: Could not copy to clipboard: {e}. Use --no-clipboard to suppress this warning."
            );
        }
    }

    match opts.output {
        OutputFormat::Text => {
            if opts.analyze {
                let analysis = SecurityAnalysis::new(&password);
                analysis.display_report(TableStyle::extended(), 80)
            } else {
                println!("{password}");
            }
        }
        OutputFormat::Json => {
            let output = PasswordOutput {
                kind: match opts.command {
                    Commands::Memorable { .. } => PasswordKind::Memorable,
                    Commands::Random { .. } => PasswordKind::Random,
                    Commands::Pin { .. } => PasswordKind::Pin,
                    #[cfg(all(target_os = "linux", feature = "clipboard"))]
                    Commands::HoldClipboard => {
                        unreachable!("handled right after argument parsing")
                    }
                },
                password: &password,
                analysis: if opts.analyze {
                    Some(SecurityAnalysis::new(&password))
                } else {
                    None
                },
            };
            println!("{}", serde_json::to_string(&output).unwrap());
        }
    }
}

/// Copies the given text to the system clipboard.
///
/// Where the platform supports it, the entry is excluded from the OS clipboard
/// history and cloud clipboard so a freshly generated password is not retained
/// locally or synced to other devices after it has been used.
#[cfg(all(feature = "clipboard", not(target_os = "linux")))]
fn copy_to_clipboard(text: &str) -> Result<(), arboard::Error> {
    #[cfg(target_os = "macos")]
    use arboard::SetExtApple;
    #[cfg(target_os = "windows")]
    use arboard::SetExtWindows;

    let mut clipboard = Clipboard::new()?;
    let set = clipboard.set();

    // On Windows, `exclude_from_monitoring` also keeps the entry out of the
    // cloud clipboard and the clipboard history list.
    #[cfg(target_os = "windows")]
    let set = set.exclude_from_monitoring();
    #[cfg(target_os = "macos")]
    let set = set.exclude_from_history();

    set.text(text)
}

/// How long the detached holder process keeps the password on the clipboard
/// before letting it clear.
#[cfg(all(feature = "clipboard", target_os = "linux"))]
const CLIPBOARD_HOLD_TTL: std::time::Duration = std::time::Duration::from_secs(120);

/// Copies the given text to the system clipboard.
///
/// X11 and Wayland clipboards drop their contents when the process that set
/// them exits, so the password is handed over stdin to a detached copy of
/// this binary (the hidden `hold-clipboard` subcommand) that keeps it
/// available until it is pasted or `CLIPBOARD_HOLD_TTL` elapses. Letting the
/// holder exit at the deadline doubles as an automatic clipboard clear.
#[cfg(all(feature = "clipboard", target_os = "linux"))]
fn copy_to_clipboard(text: &str) -> Result<(), arboard::Error> {
    if spawn_clipboard_holder(text).is_ok() {
        return Ok(());
    }

    // Fallback: set the clipboard from this process. The contents survive
    // only if a clipboard manager is running, which beats not copying at all.
    Clipboard::new()?.set().text(text)
}

#[cfg(all(feature = "clipboard", target_os = "linux"))]
fn spawn_clipboard_holder(text: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new(std::env::current_exe()?)
        .arg("hold-clipboard")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    child
        .stdin
        .take()
        .expect("child stdin should be piped")
        .write_all(text.as_bytes())
}

/// Entry point of the hidden `hold-clipboard` subcommand: reads the password
/// from stdin and owns the clipboard until it is pasted or the TTL elapses.
#[cfg(all(feature = "clipboard", target_os = "linux"))]
fn hold_clipboard() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;

    use arboard::SetExtLinux;

    let mut password = String::new();
    std::io::stdin().read_to_string(&mut password)?;

    let deadline = std::time::Instant::now() + CLIPBOARD_HOLD_TTL;
    Clipboard::new()?
        .set()
        .wait_until(deadline)
        .text(password)?;
    Ok(())
}

#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Serialize)]
struct PasswordOutput<'a> {
    kind: PasswordKind,
    password: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    analysis: Option<SecurityAnalysis<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum PasswordKind {
    Memorable,
    Random,
    Pin,
}

impl Display for PasswordKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordKind::Memorable => write!(f, "memorable"),
            PasswordKind::Random => write!(f, "random"),
            PasswordKind::Pin => write!(f, "pin"),
        }
    }
}

struct SecurityAnalysis<'a> {
    password: &'a str,
    entropy: zxcvbn::Entropy,
}

impl Serialize for SecurityAnalysis<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut crack_times = HashMap::new();
        crack_times.insert(
            "100/h",
            self.entropy
                .crack_times()
                .online_throttling_100_per_hour()
                .to_string(),
        );

        crack_times.insert(
            "10/s",
            self.entropy
                .crack_times()
                .online_no_throttling_10_per_second()
                .to_string(),
        );

        crack_times.insert(
            "10^4/s",
            self.entropy
                .crack_times()
                .offline_slow_hashing_1e4_per_second()
                .to_string(),
        );

        crack_times.insert(
            "10^10/s",
            self.entropy
                .crack_times()
                .offline_fast_hashing_1e10_per_second()
                .to_string(),
        );

        let mut struct_serializer = serializer.serialize_struct("SecurityAnalysis", 3)?;
        struct_serializer.serialize_field(
            "strength",
            &PasswordStrength::from(self.entropy.score()).to_string(),
        )?;
        struct_serializer.serialize_field(
            "guesses",
            format!("10^{:.0}", &self.entropy.guesses_log10()).as_str(),
        )?;
        struct_serializer.serialize_field("crack_times", &crack_times)?;
        struct_serializer.end()
    }
}

impl<'a> SecurityAnalysis<'a> {
    fn new(password: &'a str) -> Self {
        let entropy = zxcvbn(password, &[]);
        Self { password, entropy }
    }

    fn display_report(&self, table_style: TableStyle, max_width: usize) {
        self.display_password_table(table_style, max_width);
        self.display_analysis_table(table_style, max_width);
        self.display_crack_times_table(table_style, max_width);
    }

    fn display_password_table(&self, table_style: TableStyle, max_width: usize) {
        let table = Table::builder()
            .max_column_width(max_width)
            .style(table_style)
            .rows(rows![
                row![
                    TableCell::builder("Generated Password".bold())
                        .alignment(Alignment::Left)
                        .build(),
                ],
                row![
                    TableCell::builder(self.password)
                        .alignment(Alignment::Left)
                        .build(),
                ]
            ])
            .build();

        println!("{}", table.render());
    }

    fn display_analysis_table(&self, table_style: TableStyle, max_width: usize) {
        let table = Table::builder()
            .max_column_width(max_width)
            .style(table_style)
            .rows(rows![
                row![
                    TableCell::builder("Security Analysis")
                        .alignment(Alignment::Left)
                        .build(),
                ],
                row![
                    TableCell::builder("Strength".bold())
                        .alignment(Alignment::Left)
                        .build(),
                    TableCell::builder(
                        PasswordStrength::from(self.entropy.score()).to_colored_string()
                    )
                    .alignment(Alignment::Left)
                    .build(),
                ],
                row![
                    TableCell::builder("Guesses".bold())
                        .alignment(Alignment::Left)
                        .build(),
                    TableCell::builder(format!("10^{:.0}", self.entropy.guesses_log10()))
                        .alignment(Alignment::Left)
                        .build(),
                ],
            ])
            .build();

        println!("{}", table.render());
    }

    fn display_crack_times_table(&self, table_style: TableStyle, max_width: usize) {
        let table = Table::builder()
            .max_column_width(max_width)
            .style(table_style)
            .rows(rows![
                row![
                    TableCell::builder("Crack time estimations")
                        .alignment(Alignment::Left)
                        .build(),
                ],
                row![
                    TableCell::builder("100 attempts/hour".bold())
                        .alignment(Alignment::Left)
                        .build(),
                    TableCell::builder(
                        self.entropy
                            .crack_times()
                            .online_throttling_100_per_hour()
                            .to_string()
                    )
                    .alignment(Alignment::Left)
                    .build(),
                ],
                row![
                    TableCell::builder("10 attempts/second".bold())
                        .alignment(Alignment::Left)
                        .build(),
                    TableCell::builder(
                        self.entropy
                            .crack_times()
                            .online_no_throttling_10_per_second()
                            .to_string()
                    )
                    .alignment(Alignment::Left)
                    .build(),
                ],
                row![
                    TableCell::builder("10^4 attempts/second".bold())
                        .alignment(Alignment::Left)
                        .build(),
                    TableCell::builder(
                        self.entropy
                            .crack_times()
                            .offline_slow_hashing_1e4_per_second()
                            .to_string()
                    )
                    .alignment(Alignment::Left)
                    .build(),
                ],
                row![
                    TableCell::builder("10^10 attempts/second".bold())
                        .alignment(Alignment::Left)
                        .build(),
                    TableCell::builder(
                        self.entropy
                            .crack_times()
                            .offline_fast_hashing_1e10_per_second()
                            .to_string()
                    )
                    .alignment(Alignment::Left)
                    .build(),
                ],
            ])
            .build();

        println!("{}", table.render());
    }
}

enum PasswordStrength {
    VeryWeak,
    Weak,
    Reasonable,
    Strong,
    VeryStrong,
}

impl From<u8> for PasswordStrength {
    fn from(score: u8) -> Self {
        match score {
            0 => PasswordStrength::VeryWeak,
            1 => PasswordStrength::Weak,
            2 => PasswordStrength::Reasonable,
            3 => PasswordStrength::Strong,
            4 => PasswordStrength::VeryStrong,
            _ => unreachable!("internal error: invalid password strength"),
        }
    }
}

impl From<zxcvbn::Score> for PasswordStrength {
    fn from(score: zxcvbn::Score) -> Self {
        // Convert Score to u8 and then use the existing From<u8> implementation
        let score_u8 = score as u8;
        PasswordStrength::from(score_u8)
    }
}

impl PasswordStrength {
    fn to_colored_string(&self) -> ColoredString {
        match self {
            PasswordStrength::VeryWeak => self.to_string().red(),
            PasswordStrength::Weak => self.to_string().bright_red(),
            PasswordStrength::Reasonable => self.to_string().yellow(),
            PasswordStrength::Strong => self.to_string().bright_green(),
            PasswordStrength::VeryStrong => self.to_string().green(),
        }
    }
}

impl Display for PasswordStrength {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let strength = match self {
            PasswordStrength::VeryWeak => "very weak",
            PasswordStrength::Weak => "weak",
            PasswordStrength::Reasonable => "reasonable",
            PasswordStrength::Strong => "strong",
            PasswordStrength::VeryStrong => "very strong",
        };

        write!(f, "{strength}")
    }
}

/// validate_word_count parses the given string as a u32 and returns an error if it is not between
/// 3 and 15.
fn validate_word_count(s: &str) -> Result<u32, String> {
    match s.parse::<u32>() {
        Ok(n) if (3..16).contains(&n) => Ok(n),
        Ok(_) => Err("The number of words must be between 3 and 15".to_string()),
        Err(_) => Err("The number of words must be an integer".to_string()),
    }
}

/// validate_character_count parses the given string as a u32 and returns an error if it is not between
/// 8 and 100.
fn validate_character_count(s: &str) -> Result<u32, String> {
    match s.parse::<u32>() {
        Ok(n) if (8..101).contains(&n) => Ok(n),
        Ok(_) => Err("The number of characters must be between 8 and 100".to_string()),
        Err(_) => Err("The number of characters must be an integer".to_string()),
    }
}

/// validate_ping_length parses the given string as a u32 and returns an error if it is not between
/// 3 and 12.
fn validate_pin_length(s: &str) -> Result<u32, String> {
    match s.parse::<u32>() {
        Ok(n) if (3..13).contains(&n) => Ok(n),
        Ok(_) => Err("The number of digits must be between 3 and 12".to_string()),
        Err(_) => Err("The number of digits must be an integer".to_string()),
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
