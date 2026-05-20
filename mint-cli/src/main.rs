use admin_authority_core::AccountId;
use clap::{Parser, Subcommand};
use mint_sdk::TokenClient;

#[derive(Debug, Parser)]
#[command(name = "mint-cli")]
#[command(about = "LP-0013 token authority demo CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run an offline variable-supply authority lifecycle demo.
    DemoVariable,
    /// Run an offline fixed-supply / revoked-authority demo.
    DemoFixed,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::DemoVariable => demo_variable(),
        Command::DemoFixed => demo_fixed(),
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn demo_variable() -> Result<(), String> {
    let mint = account(0x01);
    let authority = account(0xA1);
    let next_authority = account(0xB2);
    let owner = account(0xC3);
    let mut client = TokenClient::default();

    client
        .create_variable_mint(mint, authority, 6)
        .map_err(|err| err.to_string())?;
    println!("created variable mint decimals=6");

    client
        .mint_to(mint, authority, owner, 100)
        .map_err(|err| err.to_string())?;
    println!(
        "minted amount=100 supply={} balance={}",
        client.supply(mint).unwrap_or_default(),
        client.balance(mint, owner).unwrap_or_default()
    );

    client
        .rotate_authority(mint, authority, next_authority)
        .map_err(|err| err.to_string())?;
    println!("rotated authority");

    let old_authority_err = client
        .mint_to(mint, authority, owner, 1)
        .expect_err("old authority must be rejected");
    println!("old authority rejected: {old_authority_err}");

    client
        .mint_to(mint, next_authority, owner, 25)
        .map_err(|err| err.to_string())?;
    println!(
        "minted amount=25 supply={} balance={}",
        client.supply(mint).unwrap_or_default(),
        client.balance(mint, owner).unwrap_or_default()
    );

    client
        .revoke_authority(mint, next_authority)
        .map_err(|err| err.to_string())?;
    println!("revoked authority");

    let revoked_err = client
        .mint_to(mint, next_authority, owner, 1)
        .expect_err("revoked authority must reject minting");
    println!("post-revoke mint rejected: {revoked_err}");
    Ok(())
}

fn demo_fixed() -> Result<(), String> {
    let mint = account(0x02);
    let authority = account(0xA1);
    let owner = account(0xC3);
    let mut client = TokenClient::default();

    client
        .create_fixed_mint(mint, 6)
        .map_err(|err| err.to_string())?;
    println!("created fixed mint decimals=6");

    let err = client
        .mint_to(mint, authority, owner, 1)
        .expect_err("fixed mint must reject minting");
    println!("mint rejected: {err}");
    Ok(())
}

const fn account(byte: u8) -> AccountId {
    AccountId::new([byte; 32])
}
