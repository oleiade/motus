use assert_cmd::Command;

// Helper function to create a command with appropriate clipboard flags
fn motus_command() -> Command {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // Only add --no-clipboard if the clipboard feature is enabled
    #[cfg(feature = "clipboard")]
    cmd.arg("--no-clipboard");

    cmd
}

#[test]
fn test_memorable_command_default_behavior() {
    let mut cmd = motus_command();

    // `motus --seed 42 memorable`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("memorable")
        .assert()
        .success()
        .stdout("chokehold nativity dolly ominous throat\n");
}

#[test]
fn test_memorable_command_custom_word_count() {
    let mut cmd = motus_command();

    // `motus --seed 42 memorable --words 7`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("memorable");
    cmd.arg("--words");
    cmd.arg("7")
        .assert()
        .success()
        .stdout("chokehold native dollop omen thrive pungent woozy\n");
}

#[test]
fn test_memorable_command_custom_separator() {
    let mut cmd = motus_command();

    // `motus --seed 42 memorable --separator " "`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("memorable");
    cmd.arg("--separator");
    cmd.arg("numbers-and-symbols")
        .assert()
        .success()
        .stdout("chokehold2nativity9dolly(ominous9throat\n");
}

#[test]
fn test_memorable_command_capitalize() {
    let mut cmd = motus_command();

    // `motus --seed 42 memorable --capitalize`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("memorable");
    cmd.arg("--capitalize")
        .assert()
        .success()
        .stdout("Chokehold Nativity Dolly Ominous Throat\n");
}

#[test]
fn test_memorable_command_no_full_words() {
    let mut cmd = motus_command();

    // `motus --seed 42 memorable --no-full-words`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("memorable");
    cmd.arg("--no-full-words")
        .assert()
        .success()
        .stdout("edhhookcl tyaitniv dlloy mnosiuo htator\n");
}

#[test]
fn test_memorable_command_all_options() {
    let mut cmd = motus_command();

    // `motus --seed 42 memorable --words 7 --separator numbers-and-symbols --capitalize --no-full-words`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("memorable");
    cmd.arg("--words");
    cmd.arg("7");
    cmd.arg("--separator");
    cmd.arg("numbers-and-symbols");
    cmd.arg("--capitalize");
    cmd.arg("--no-full-words")
        .assert()
        .success()
        .stdout("Lhkheoodc6Aivnte2Odopll#Mnoe)Thervi!Npetnug6Yzowo\n");
}

#[test]
fn test_memorable_command_too_little_words() {
    let mut cmd = motus_command();

    // `motus --seed 42 memorable --words 2`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("memorable");
    cmd.arg("--words");
    cmd.arg("2").assert().failure();
}

#[test]
fn test_memorable_command_too_many_words() {
    let mut cmd = motus_command();

    // `motus --seed 42 memorable --words 16`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("memorable");
    cmd.arg("--words");
    cmd.arg("16").assert().failure();
}

#[test]
fn test_memorable_command_unknown_separator() {
    let mut cmd = motus_command();

    // `motus --seed 42 memorable --separator "foo"`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("memorable");
    cmd.arg("--separator");
    cmd.arg("foo").assert().failure();
}

#[test]
fn test_memorable_command_json_output() {
    let mut cmd = motus_command();

    // motus --seed 42 memorable
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("--output");
    cmd.arg("json");
    cmd.arg("memorable");

    let output = cmd.output().expect("failed to execute process");

    let json = String::from_utf8(output.stdout)
        .expect("unable to parse json output; reason: invalid utf-8");

    use assert_json::assert_json;

    assert_json!(json.as_str(), {
        "kind": "memorable",
        "password": "chokehold nativity dolly ominous throat",
    });
}

#[test]
fn test_memorable_command_analyze_json_output() {
    let mut cmd = motus_command();

    // motus --seed 42 memorable
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("--analyze");
    cmd.arg("--output");
    cmd.arg("json");
    cmd.arg("memorable");

    let output = cmd.output().expect("failed to execute process");

    let json = String::from_utf8(output.stdout)
        .expect("unable to parse json output; reason: invalid utf-8");
    println!("JSON: {json}");

    use assert_json::assert_json;

    assert_json!(json.as_str(), {
        "kind": "memorable",
        "password": "chokehold nativity dolly ominous throat",
        "analysis": {
            "strength": "very strong",
            "guesses": "10^19",
            "crack_times": {
                "10/s": "centuries",
                "100/h": "centuries",
                "10^10/s": "57 years",
                "10^4/s": "centuries"
            },
        },
    });
}

#[test]
fn test_random_command_default_behavior() {
    let mut cmd = motus_command();

    // `motus --seed 42 random`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("random")
        .assert()
        .success()
        .stdout("BCHvbvMSgaWAuhBlaBcH\n");
}

#[test]
fn test_random_command_specified_characters_count() {
    let mut cmd = motus_command();

    // `motus --seed 42 random --characters 10`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("random");
    cmd.arg("--characters");
    cmd.arg("10").assert().success().stdout("BCHvbvMSga\n");
}

#[test]
fn test_random_command_numbers() {
    let mut cmd = motus_command();

    // `motus --seed 42 random --numbers`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("random");
    cmd.arg("--numbers")
        .assert()
        .success()
        .stdout("BC640vMSga9A3h52aBcH\n");
}

#[test]
fn test_random_command_symbols() {
    let mut cmd = motus_command();

    // `motus --seed 42 random --symbols`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("random");
    cmd.arg("--symbols")
        .assert()
        .success()
        .stdout("BC&%!vMSga)A$h^#aBcH\n");
}

#[test]
fn test_random_command_all_options() {
    let mut cmd = motus_command();

    // `motus --seed 42 random --characters 10 --numbers --symbols`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("random");
    cmd.arg("--characters");
    cmd.arg("10");
    cmd.arg("--numbers");
    cmd.arg("--symbols")
        .assert()
        .success()
        .stdout("BC6%!vMSga\n");
}

#[test]
fn test_random_command_too_little_characters() {
    let mut cmd = motus_command();

    // `motus --seed 42 random --characters 2`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("random");
    cmd.arg("--characters");
    cmd.arg("2").assert().failure();
}

#[test]
fn test_random_command_too_many_characters() {
    let mut cmd = motus_command();

    // `motus --seed 42 random --characters 101`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("random");
    cmd.arg("--characters");
    cmd.arg("101").assert().failure();
}

#[test]
fn test_random_command_json_output() {
    let mut cmd = motus_command();

    // motus --seed 42 memorable
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("--output");
    cmd.arg("json");
    cmd.arg("random");

    let output = cmd.output().expect("failed to execute process");

    let json = String::from_utf8(output.stdout)
        .expect("unable to parse json output; reason: invalid utf-8");

    use assert_json::assert_json;

    assert_json!(json.as_str(), {
        "kind": "random",
        "password": "BCHvbvMSgaWAuhBlaBcH",
    });
}

#[test]
fn test_random_command_analyze_json_output() {
    let mut cmd = motus_command();

    // motus --seed 42 memorable
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("--analyze");
    cmd.arg("--output");
    cmd.arg("json");
    cmd.arg("random");

    let output = cmd.output().expect("failed to execute process");

    let json = String::from_utf8(output.stdout)
        .expect("unable to parse json output; reason: invalid utf-8");

    use assert_json::assert_json;

    assert_json!(json.as_str(), {
        "kind": "random",
        "password": "BCHvbvMSgaWAuhBlaBcH",
        "analysis": {
            "strength": "very strong",
            "guesses": "10^19",
            "crack_times": {
                "10/s": "centuries",
                "100/h": "centuries",
                "10^10/s": "57 years",
                "10^4/s": "centuries"
            },
        },
    });
}

#[test]
fn test_pin_command_default_behavior() {
    let mut cmd = motus_command();

    // `motus --seed 42 pin`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("pin").assert().success().stdout("1525869\n");
}

#[test]
fn test_pin_command_numbers() {
    let mut cmd = motus_command();

    // `motus --seed 42 pin --numbers`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("pin");
    cmd.arg("--numbers");
    cmd.arg("9").assert().success().stdout("152586949\n");
}

#[test]
fn test_pin_command_too_little_numbers() {
    let mut cmd = motus_command();

    // `motus --seed 42 pin --numbers 2`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("pin");
    cmd.arg("--numbers");
    cmd.arg("2").assert().failure();
}

#[test]
fn test_pin_command_too_many_numbers() {
    let mut cmd = motus_command();

    // `motus --seed 42 pin --numbers 9`
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("pin");
    cmd.arg("--numbers");
    cmd.arg("13").assert().failure();
}

#[test]
fn test_pin_command_json_output() {
    let mut cmd = motus_command();

    // motus --seed 42 memorable
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("--output");
    cmd.arg("json");
    cmd.arg("pin");

    let output = cmd.output().expect("failed to execute process");

    let json = String::from_utf8(output.stdout)
        .expect("unable to parse json output; reason: invalid utf-8");

    use assert_json::assert_json;

    assert_json!(json.as_str(), {
        "kind": "pin",
        "password": "1525869",
    });
}

#[test]
fn test_pin_command_analyze_json_output() {
    let mut cmd = motus_command();

    // motus --seed 42 memorable
    cmd.arg("--seed");
    cmd.arg("42");
    cmd.arg("--analyze");
    cmd.arg("--output");
    cmd.arg("json");
    cmd.arg("pin");

    let output = cmd.output().expect("failed to execute process");

    let json = String::from_utf8(output.stdout)
        .expect("unable to parse json output; reason: invalid utf-8");

    use assert_json::assert_json;

    assert_json!(json.as_str(), {
        "kind": "pin",
        "password": "1525869",
        "analysis": {
            "strength": "reasonable",
            "guesses": "10^6",
            "crack_times": {
                "10/s": "1 day",
                "100/h": "1 year",
                "10^10/s": "less than a second",
                "10^4/s": "2 minutes"
            },
        },
    });
}
