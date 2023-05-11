use assert_cmd::Command;

#[test]
fn test_memorable_command_default_behavior() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 memorable`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("memorable")
        .assert()
        .success()
        .stdout("chokehold nativity dolly ominous throat\n");
}

#[test]
fn test_memorable_command_custom_word_count() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 memorable --words 7`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("memorable")
        .arg("--words")
        .arg("7")
        .assert()
        .success()
        .stdout("chokehold native dollop omen thrive pungent woozy\n");
}

#[test]
fn test_memorable_command_custom_separator() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 memorable --separator " "`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("memorable")
        .arg("--separator")
        .arg("numbers-and-symbols")
        .assert()
        .success()
        .stdout("chokehold(nativity9dolly2ominous(throat\n");
}

#[test]
fn test_memorable_command_capitalize() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 memorable --capitalize`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("memorable")
        .arg("--capitalize")
        .assert()
        .success()
        .stdout("Chokehold Nativity Dolly Ominous Throat\n");
}

#[test]
fn test_memorable_command_no_full_words() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 memorable --no-full-words`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("memorable")
        .arg("--no-full-words")
        .assert()
        .success()
        .stdout("lhodheokc inayittv loydl uoimson tohatr\n");
}

#[test]
fn test_memorable_command_all_options() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 memorable --words 7 --separator numbers-and-symbols --capitalize --no-full-words`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("memorable")
        .arg("--words")
        .arg("7")
        .arg("--separator")
        .arg("numbers-and-symbols")
        .arg("--capitalize")
        .arg("--no-full-words")
        .assert()
        .success()
        .stdout("Lhodheokc2Tnaevi)Loopld!Meno7Etvrhi$Uptgnne^Ozoyw\n");
}

#[test]
fn test_memorable_command_too_little_words() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 memorable --words 2`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("memorable")
        .arg("--words")
        .arg("2")
        .assert()
        .failure();
}

#[test]
fn test_memorable_command_too_many_words() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 memorable --words 16`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("memorable")
        .arg("--words")
        .arg("16")
        .assert()
        .failure();
}

#[test]
fn test_memorable_command_unknown_separator() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 memorable --separator "foo"`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("memorable")
        .arg("--separator")
        .arg("foo")
        .assert()
        .failure();
}

#[test]
fn test_random_command_default_behavior() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 random`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("random")
        .assert()
        .success()
        .stdout("mHYvjgQAKBHBIRYdpPAI\n");
}

#[test]
fn test_random_command_specified_characters_count() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 random --characters 10`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("random")
        .arg("--characters")
        .arg("10")
        .assert()
        .success()
        .stdout("mHYvjgQAKB\n");
}

#[test]
fn test_random_command_numbers() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 random --numbers`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("random")
        .arg("--numbers")
        .assert()
        .success()
        .stdout("mH9vj1Q57B6BIRYdpPAI\n");
}

#[test]
fn test_random_command_symbols() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 random --symbols`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("random")
        .arg("--symbols")
        .assert()
        .success()
        .stdout("mH)vj@Q^*B&BIRYdpPAI\n");
}

#[test]
fn test_random_command_all_options() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 random --characters 10 --numbers --symbols`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("random")
        .arg("--characters")
        .arg("10")
        .arg("--numbers")
        .arg("--symbols")
        .assert()
        .success()
        .stdout("mH)vj1Q^7B\n");
}

#[test]
fn test_random_command_too_little_characters() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 random --characters 2`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("random")
        .arg("--characters")
        .arg("2")
        .assert()
        .failure();
}

#[test]
fn test_random_command_too_many_characters() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 random --characters 101`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("random")
        .arg("--characters")
        .arg("101")
        .assert()
        .failure();
}

#[test]
fn test_pin_command_default_behavior() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 pin`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("pin")
        .assert()
        .success()
        .stdout("5564047\n");
}

#[test]
fn test_pin_command_numbers() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 pin --numbers`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("pin")
        .arg("--numbers")
        .arg("9")
        .assert()
        .success()
        .stdout("556404781\n");
}

#[test]
fn test_pin_command_too_little_numbers() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 pin --numbers 2`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("pin")
        .arg("--numbers")
        .arg("2")
        .assert()
        .failure();
}

#[test]
fn test_pin_command_too_many_numbers() {
    let mut cmd = Command::cargo_bin("motus").unwrap();

    // `motus --seed 42 pin --numbers 9`
    cmd.arg("--no-clipboard")
        .arg("--seed")
        .arg("42")
        .arg("pin")
        .arg("--numbers")
        .arg("13")
        .assert()
        .failure();
}
