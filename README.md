<p align="center"><img src="logo.png" alt="motus logo"/></p>
<h1 align="center">Dead simple password generator</h3>

<p align="center">
    <a href="http://github.com/oleiade/motus/releases"><img src="https://img.shields.io/github/release/oleiade/motus.svg" alt="release"></a>
    <a href="http://www.gnu.org/licenses/agpl-3.0"><img src="https://img.shields.io/badge/license-AGPL-blue.svg" alt="AGPL License"></a>
    <a href="https://github.com/oleiade/motus/actions/workflows/build.yml"><img src="https://github.com/oleiade/motus/actions/workflows/build.yml/badge.svg" alt="Build status"></a>
</p>

Motus is a command-line application that makes generating secure passwords a breeze.

Inspired by the user experience of the 1Password password generator, motus focuses on providing a simple and elegant user interface with sane defaults and comprehensive options. By default, motus copies the generated password to your clipboard, making it even more convenient to use.

<p align="center">
  <img src="static/motus-demo.gif" alt="motus demo" />
</p>

## Features

- **Simple and elegant user interface**: motus is designed to be easy to use and understand, and makes it difficult to generate insecure passwords.
- Generate **secure memorable passwords**: motus uses the [EFF's wordlist](https://www.eff.org/deeplinks/2016/07/new-wordlists-random-passphrases) to generate secure and memorable passwords
- Generate **random passwords** with optional number and symbol inclusion
- Generate **PINs** with customizable length
- **Security analysis**: the `--analyze` option provides a security analysis of the generated password, ensuring optimal password strength.
- **Sane defaults**
- **Clipboard** integration for easy password usage
- Flexible **customization** options

## Installation

### on macOS

Using Homebrew:

```bash
brew tap oleiade/tap
brew install motus
```

### on Debian/Ubuntu Linux

Add the repository and install motus:

```bash
# Download and install the repository's GPG key
curl -fsSL https://oleiade.github.io/deb/oleiade-archive-keyring.gpg | \
gpg --dearmor | \
sudo tee /usr/share/keyrings/oleiade-archive-keyring.gpg > /dev/null

# Add the repository to your system's sources
echo "deb [signed-by=/usr/share/keyrings/oleiade-archive-keyring.gpg] https://oleiade.github.io/deb stable main" | \
sudo tee /etc/apt/sources.list.d/oleiade.list > /dev/null

# Update your sources
sudo apt update

# Install motus
sudo apt install motus
```

### using Cargo

Alternatively, you can install using Cargo:

```bash
# Install with clipboard support (default)
cargo install motus

# Install without clipboard support (for headless/server environments)
cargo install motus --no-default-features
```


## Usage

```bash
> motus --help
Motus is a command-line tool for generating secure, random, and memorable passwords as well as PIN codes.

Usage: motus [OPTIONS] <COMMAND>

Commands:
  memorable
          Generate a human-friendly memorable password
  random
          Generate a random password with specified complexity
  pin
          Generate a random numeric PIN code
  help
          Print this message or the help of the given subcommand(s)

Options:
      --no-clipboard
          Disable automatic copying of generated password to clipboard

  -o, --output <OUTPUT>
          Output the generated password in a specified format

          [default: text]
          [possible values: text, json]

      --analyze
          Display a safety analysis along the generated password

      --seed <SEED>
          Seed value for deterministic password generation (for testing purposes)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Generate a memorable password

```bash
> motus memorable
fossil abreast overplant commute dish

# Or customize the password generation
> motus memorable --words 7 --separator numbers-and-symbols --capitalize
Goes$Stood3Paving(Tipoff$Settle*Flip3Scone
```

### Generate a random password

```bash
> motus random
UDrZrJJTYElWeOFHZmfp

# Or customize the password generation
> motus random --characters 42 --numbers --symbols
6HdwMjKQPYE3scIBlCps&1Ir5R8lQ85eIVtF!fpUSD
```

### Generate a PIN

```bash
> motus pin
1234421

# Or customize the size of the PIN
> motus pin --numbers 9
347751411
```

### Miscellaneous

#### Generate a password and analyze its security

![motus --analyze](static/motus-demo-report.gif)

##### Generate a password and output the result in JSON format

```bash
> motus --output json random
{"kind": "memorable", "password": "6HdwMjKQPYE3scIBlCps&1Ir5R8lQ85eIVtF!fpUSD"}
```

## Headless and SSH Usage

### Clipboard Behavior

For headless environments (SSH sessions, Docker containers, servers), you have two options:

**Option 1: Compile without clipboard support (recommended for servers)**
```bash
# Install headless version with no clipboard dependencies
cargo install motus --no-default-features

# Use normally - no clipboard code or warnings
motus memorable --words 5
```

**Option 2: Use runtime flag with standard version**
In environments without X11/Wayland, the standard version will show warnings when clipboard access fails:

- The password is still generated and displayed
- A warning message is shown on stderr explaining the clipboard failure
- Use `--no-clipboard` to suppress the warning for automated scripts

```bash
# Suppress clipboard warnings for server usage
motus --no-clipboard memorable --words 5
```

### Output Streams

Motus follows Unix conventions for output streams:
- **stdout**: Contains only the generated password (or JSON output)
- **stderr**: Contains warnings and error messages

This design ensures passwords can be reliably captured by scripts even when warnings occur.

**Script usage example:**
```bash
#!/bin/bash
# Capture password to variable, ignore clipboard warnings
PASSWORD=$(motus memorable --words 4 2>/dev/null)

# Or capture warnings separately for logging
motus memorable --words 4 2>warnings.log | some-other-tool

# Pipe password directly to other tools
motus --no-clipboard random --characters 20 | pbcopy
```

## Contributing

We welcome contributions to the project. Feel free to submit issues, suggest new features, or create pull requests to help improve motus.

## License

motus is distributed under the [AGPL-3.0 license](https://github.com/oleiade/motus/blob/master/LICENSE).

## Why the name?

motus used to be a tv game that I would call the ancestor of Wordle. Players had to guess words of a given size, and would pick up balls from a cup to decide how each round would move along. They would make that very distinct move to scramble the balls around every time, with a very distinct sound. When starting this project, I thought of the process of generating passwords as this comforting and satisfying act of diving into a huge cup full of numbered balls, and the childish feeling of it. This project is named in memory of Motus.
