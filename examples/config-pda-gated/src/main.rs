use admin_authority_core::AccountId;
use mint_sdk::TokenClient;

fn main() -> Result<(), String> {
    let mint = account(0x03);
    let config = account(0x44);
    let unauthorized_config = account(0x45);
    let holder = account(0xC3);
    let config_pda = derive_config_pda(b"lp-0013:mint-authority", &config);
    let wrong_pda = derive_config_pda(b"lp-0013:mint-authority", &unauthorized_config);
    let mut client = TokenClient::default();

    println!("LP-0013 RFP-001 config PDA gated authority example");
    println!(
        "config pda gate derived prefix={:02x?}",
        &config_pda.as_bytes()[..4]
    );

    client
        .create_variable_mint(mint, config_pda, 6)
        .map_err(|err| err.to_string())?;

    let unauthorized_err = client
        .mint_to(mint, wrong_pda, holder, 1)
        .expect_err("wrong config-derived PDA must not satisfy gate");
    println!("unauthorized config rejected: {unauthorized_err}");

    client
        .mint_to(mint, config_pda, holder, 64)
        .map_err(|err| err.to_string())?;
    println!(
        "authorized config minted supply={} balance={}",
        client.supply(mint).unwrap_or_default(),
        client.balance(mint, holder).unwrap_or_default()
    );

    client
        .revoke_authority(mint, config_pda)
        .map_err(|err| err.to_string())?;
    println!("gate revoked");

    let revoked_err = client
        .mint_to(mint, config_pda, holder, 1)
        .expect_err("revoked gate must not mint");
    println!("post-revoke config mint rejected: {revoked_err}");

    Ok(())
}

fn derive_config_pda(domain: &[u8], config: &AccountId) -> AccountId {
    let mut out = [0u8; 32];
    for (idx, byte) in domain.iter().chain(config.as_bytes().iter()).enumerate() {
        let slot = idx % out.len();
        out[slot] = out[slot].wrapping_add(*byte).rotate_left((idx % 8) as u32)
            ^ (idx as u8).wrapping_mul(31);
    }
    AccountId::new(out)
}

const fn account(byte: u8) -> AccountId {
    AccountId::new([byte; 32])
}
