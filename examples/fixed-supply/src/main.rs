use admin_authority_core::AccountId;
use mint_sdk::TokenClient;

fn main() -> Result<(), String> {
    let mint = account(0x02);
    let signer = account(0xA1);
    let holder = account(0xC3);
    let mut client = TokenClient::default();

    println!("LP-0013 fixed supply authority example");

    client
        .create_fixed_mint(mint, 6)
        .map_err(|err| err.to_string())?;
    println!("created fixed mint decimals=6");
    println!(
        "current authority={:?}",
        client.current_authority(mint).unwrap_or(None)
    );

    let err = client
        .mint_to(mint, signer, holder, 1)
        .expect_err("fixed supply mint must reject future minting");
    println!("mint rejected because authority is revoked: {err}");

    Ok(())
}

const fn account(byte: u8) -> AccountId {
    AccountId::new([byte; 32])
}
