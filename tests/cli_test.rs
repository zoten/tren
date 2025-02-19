#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn successful_cli_invocation() {
        // CARGO_BIN_EXE_<name> env set by cargo on tests
        let binary = option_env!("CARGO_BIN_EXE_tren").unwrap();
        println!("{:?}", binary);
        let output = Command::new(binary)
            .arg("src/tests/cli.csv")
            .output()
            .expect("failed to execute process");

        assert!(output.status.success(), "Process exited abnormally");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("amount"), "Output did not contain `amount`");
        // not exhaustive of course but "good enough": the file has a 1.00009 - 5 digits after decimals
        // that by assumptions will be accepted and then rounded
        assert!(
            stdout.contains("1.0001"),
            "Output seems not rounded to the 4th decimal"
        );
    }

    #[test]
    fn unsuccessful_cli_invocation() {
        // Cargo sets the CARGO_BIN_EXE_<name> environment variable for tests.
        let binary = option_env!("CARGO_BIN_EXE_tren").unwrap();
        println!("{:?}", binary);
        let output = Command::new(binary)
            .arg("src/tests/random_nonexistent_file0989072839743.csv")
            .output()
            .expect("failed to execute process");

        assert!(
            !output.status.success(),
            "Process exited normally, but should have failed"
        );
    }
}
