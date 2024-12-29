use crate::args::Args;
use clap::Parser;

#[test]
fn test_default_args() {
    let args = Args::parse_from(["program"]);
    assert_eq!(args.address, "0.0.0.0:8080");
}

#[test]
fn test_custom_address() {
    let args = Args::parse_from(["program", "--address", "127.0.0.1:9000"]);
    assert_eq!(args.address, "127.0.0.1:9000");
}
