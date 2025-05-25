use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Port for the HTTP server
    #[arg(long, default_value_t = 9001)]
    port: u16,
}

#[test]
fn defaults() {
    let cli = Cli::parse_from(["test"]);
    assert_eq!(cli.port, 9001);
}

#[test]
fn custom_port() {
    let cli = Cli::parse_from(["test", "--port", "8000"]);
    assert_eq!(cli.port, 8000);
}
