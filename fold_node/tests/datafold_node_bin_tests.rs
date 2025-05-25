use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Port for the P2P network
    #[arg(long, default_value_t = 9000)]
    port: u16,

    /// Port for the TCP server
    #[arg(long, default_value_t = 9000)]
    tcp_port: u16,
}

#[test]
fn defaults() {
    let cli = Cli::parse_from(["test"]);
    assert_eq!(cli.port, 9000);
    assert_eq!(cli.tcp_port, 9000);
}

#[test]
fn custom_values() {
    let cli = Cli::parse_from(["test", "--port", "8000", "--tcp-port", "8001"]);
    assert_eq!(cli.port, 8000);
    assert_eq!(cli.tcp_port, 8001);
}
