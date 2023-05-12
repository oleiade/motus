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
        struct_serializer.serialize_field("guesses", &self.entropy.guesses_log10())?;
        struct_serializer.serialize_field("crack_times", &crack_times)?;
        struct_serializer.end()
    }
}

impl<'a> SecurityAnalysis<'a> {
    fn new(password: &'a str) -> Self {
        let entropy = zxcvbn(password, &[]).expect("unable to analyze password's safety");
        Self { password, entropy }
    }

    fn display_report(&self, table_style: TableStyle, max_width: usize) {
        self.display_password_table(table_style, max_width);
        self.display_analysis_table(table_style, max_width);
        self.display_crack_times_table(table_style, max_width);
    }

    fn display_password_table(&self, table_style: TableStyle, max_width: usize) {
        let mut table = Table::new();
        table.max_column_width = max_width;
        table.style = table_style;

        table.add_row(Row::new(vec![TableCell::new_with_alignment(
            "Generated Password".bold(),
            1,
            Alignment::Left,
        )]));

        table.add_row(Row::new(vec![TableCell::new(self.password)]));

        println!("{}", table.render());
    }

    fn display_analysis_table(&self, table_style: TableStyle, max_width: usize) {
        let mut table = Table::new();
        table.max_column_width = max_width;
        table.style = table_style;

        table.add_row(Row::new(vec![TableCell::new_with_alignment(
            "Security Analysis",
            2,
            Alignment::Left,
        )]));

        table.add_row(Row::new(vec![
            TableCell::new("Strength".bold()),
            TableCell::new_with_alignment(
                PasswordStrength::from(self.entropy.score()).to_string(),
                1,
                Alignment::Left,
            ),
        ]));

        table.add_row(Row::new(vec![
            TableCell::new("Guesses".bold()),
            TableCell::new_with_alignment(
                format!("10^{:.0}", self.entropy.guesses_log10()),
                1,
                Alignment::Left,
            ),
        ]));

        println!("{}", table.render());
    }

    fn display_crack_times_table(&self, table_style: TableStyle, max_width: usize) {
        let mut table = Table::new();
        table.max_column_width = max_width;
        table.style = table_style;

        table.add_row(Row::new(vec![TableCell::new_with_alignment(
            "Crack time estimations",
            2,
            Alignment::Left,
        )]));

        table.add_row(Row::new(vec![
            TableCell::new("100 attempts/hour".bold()),
            TableCell::new_with_alignment(
                format!(
                    "{}",
                    self.entropy.crack_times().online_throttling_100_per_hour()
                ),
                1,
                Alignment::Left,
            ),
        ]));

        table.add_row(Row::new(vec![
            TableCell::new("10 attempts/second".bold()),
            TableCell::new_with_alignment(
                format!(
                    "{}",
                    self.entropy
                        .crack_times()
                        .online_no_throttling_10_per_second()
                ),
                1,
                Alignment::Left,
            ),
        ]));

        table.add_row(Row::new(vec![
            TableCell::new("10^4 attempts/second".bold()),
            TableCell::new_with_alignment(
                format!(
                    "{}",
                    self.entropy
                        .crack_times()
                        .offline_slow_hashing_1e4_per_second()
                ),
                1,
                Alignment::Left,
            ),
        ]));

        table.add_row(Row::new(vec![
            TableCell::new("10^10 attempts/second".bold()),
            TableCell::new_with_alignment(
                format!(
                    "{}",
                    self.entropy
                        .crack_times()
                        .offline_fast_hashing_1e10_per_second()
                ),
                1,
                Alignment::Left,
            ),
        ]));

        println!("{}", table.render());
    }
}
