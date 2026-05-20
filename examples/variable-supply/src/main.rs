use admin_authority_core::AccountId;
use mint_sdk::TokenClient;

fn main() -> Result<(), String> {
    let mint = account(0x01);
    let authority = account(0xA1);
    let next_authority = account(0xB2);
    let holder = account(0xC3);
    let mut client = TokenClient::default();

    println!("LP-0013 variable supply authority example");

    client
        .create_variable_mint(mint, authority, 6)
        .map_err(|err| err.to_string())?;
    println!("created variable mint decimals=6");

    client
        .mint_to(mint, authority, holder, 100)
        .map_err(|err| err.to_string())?;
    println!(
        "minted initial supply={}",
        client.supply(mint).unwrap_or_default()
    );

    client
        .rotate_authority(mint, authority, next_authority)
        .map_err(|err| err.to_string())?;
    println!("authority rotated");

    let old_err = client
        .mint_to(mint, authority, holder, 1)
        .expect_err("old authority must not mint after rotation");
    println!("old authority rejected: {old_err}");

    client
        .mint_to(mint, next_authority, holder, 25)
        .map_err(|err| err.to_string())?;
    println!(
        "new authority minted supply={} balance={}",
        client.supply(mint).unwrap_or_default(),
        client.balance(mint, holder).unwrap_or_default()
    );

    client
        .revoke_authority(mint, next_authority)
        .map_err(|err| err.to_string())?;
    println!("authority revoked");

    let revoked_err = client
        .mint_to(mint, next_authority, holder, 1)
        .expect_err("revoked authority must not mint");
    println!("post-revoke mint rejected: {revoked_err}");

    Ok(())
}

const fn account(byte: u8) -> AccountId {
    AccountId::new([byte; 32])
}
