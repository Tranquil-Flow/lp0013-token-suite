//! LP-0013 live lifecycle driver — signs and submits real SPEL txs.
//!
//! Bypasses the IDL-driven CLI (which can't serialize `Option<T>` args in
//! the current SPEL revision) by passing a strongly-typed Instruction enum
//! to `Message::try_new`, which calls `Program::serialize_instruction`
//! internally to match the guest's `risc0_zkvm::serde::Deserializer`.
//!
//! Run with:
//!
//!   export NSSA_WALLET_HOME_DIR=.../whistleblower/.scaffold/wallet
//!   export LP0013_PROGRAM_BIN=.../admin_authority_spike.bin
//!   cargo run -p admin_authority_spike-examples --bin live_lifecycle

use borsh::{BorshDeserialize, BorshSerialize};
use common::transaction::NSSATransaction;
use nssa::{
    program::Program,
    public_transaction::{Message, WitnessSet},
    AccountId, PublicTransaction,
};
use sequencer_service_rpc::RpcClient;
use serde::Serialize;
use spel_framework::pda::{compute_pda, seed_from_str};
use std::fs;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use wallet::WalletCore;

#[derive(Serialize)]
enum Instruction {
    CreateMint {
        decimals: u8,
        initial_authority: Option<[u8; 32]>,
    },
    MintTo {
        amount: u128,
    },
    SetMintAuthority {
        new_authority: Option<[u8; 32]>,
    },
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct OnChainAuthorityInfo {
    authority_type: u8,
    current_authority: Option<[u8; 32]>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct OnChainMintDefinition {
    authority: OnChainAuthorityInfo,
    supply: u128,
    decimals: u8,
}

async fn submit_signed(
    wallet_core: &WalletCore,
    program: &Program,
    accounts: Vec<AccountId>,
    signer: AccountId,
    instruction: Instruction,
    label: &str,
    expect_success: bool,
) -> String {
    let nonces = match wallet_core.get_accounts_nonces(vec![signer]).await {
        Ok(n) => n,
        Err(e) => return format!("{label}: fetch nonce: {e:?}"),
    };
    let signing_key = match wallet_core.get_account_public_signing_key(signer) {
        Some(k) => k,
        None => return format!("{label}: no signing key for {signer}"),
    };

    let message = match Message::try_new(program.id(), accounts, nonces, instruction) {
        Ok(m) => m,
        Err(e) => return format!("{label}: build message: {e:?}"),
    };
    let witness_set = WitnessSet::for_message(&message, &[&signing_key]);
    let tx = PublicTransaction::new(message, witness_set);
    let hash = match wallet_core
        .sequencer_client
        .send_transaction(NSSATransaction::Public(tx))
        .await
    {
        Ok(h) => h,
        Err(e) => return format!("{label}: submit: {e:?}"),
    };
    let hash_hex = format!("{hash:?}");

    let poll = Duration::from_millis(750);
    let attempts = if expect_success { 40 } else { 20 };
    for _ in 0..attempts {
        tokio::time::sleep(poll).await;
        if let Ok(Some(_)) = wallet_core.sequencer_client.get_transaction(hash).await {
            return if expect_success {
                format!("confirmed tx={hash_hex}")
            } else {
                format!("UNEXPECTED confirm tx={hash_hex}")
            };
        }
    }
    if expect_success {
        format!("did NOT confirm in {attempts}*0.75s tx={hash_hex}")
    } else {
        format!("rejected as expected (no inclusion) tx={hash_hex}")
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let wallet_core = Arc::new(
        WalletCore::from_env()
            .map_err(|e| format!("WalletCore::from_env failed (NSSA_WALLET_HOME_DIR?): {e:?}"))?,
    );

    let elf_path = std::env::var("LP0013_PROGRAM_BIN")
        .map_err(|_| "set LP0013_PROGRAM_BIN to admin_authority_spike.bin".to_string())?;
    let elf = fs::read(&elf_path).map_err(|e| format!("read program ELF {elf_path}: {e}"))?;
    let program = Program::new(elf).map_err(|e| format!("parse program: {e:?}"))?;
    let program_id = program.id();

    let mint_seed = seed_from_str("lp0013:mint:v1");
    let mint_pda = compute_pda(&program_id, &[&mint_seed]);
    let holding_seed = seed_from_str("lp0013:holding:v1");
    let recipient_holding = compute_pda(&program_id, &[&holding_seed]);
    let authority_pubkey = AccountId::from_str("2RHZhw9h534Zr3eq2RGhQete2Hh667foECzXPmSkGni2")
        .map_err(|e| format!("parse authority: {e:?}"))?;
    let authority_bytes: [u8; 32] = *authority_pubkey.value();

    println!("== LP-0013 live lifecycle ==");
    println!("program_id  = {program_id:?}");
    println!("mint_pda    = {mint_pda}");
    println!("authority   = {authority_pubkey}");
    println!("recipient   = {recipient_holding}");
    println!();

    let r1 = submit_signed(
        &wallet_core,
        &program,
        vec![mint_pda, authority_pubkey],
        authority_pubkey,
        Instruction::CreateMint {
            decimals: 6,
            initial_authority: Some(authority_bytes),
        },
        "create_mint(decimals=6, Some(authority))",
        true,
    )
    .await;
    println!("[1] create_mint              {r1}");

    let r2 = submit_signed(
        &wallet_core,
        &program,
        vec![mint_pda, recipient_holding, authority_pubkey],
        authority_pubkey,
        Instruction::MintTo { amount: 100 },
        "mint_to(amount=100)",
        true,
    )
    .await;
    println!("[2] mint_to(100)             {r2}");

    let r3 = submit_signed(
        &wallet_core,
        &program,
        vec![mint_pda, authority_pubkey],
        authority_pubkey,
        Instruction::SetMintAuthority {
            new_authority: None,
        },
        "set_mint_authority(None)",
        true,
    )
    .await;
    println!("[3] set_mint_authority(None) {r3}");

    let r4 = submit_signed(
        &wallet_core,
        &program,
        vec![mint_pda, recipient_holding, authority_pubkey],
        authority_pubkey,
        Instruction::MintTo { amount: 7 },
        "mint_to(post-revoke)",
        false,
    )
    .await;
    println!("[4] mint_to(post-revoke)     {r4}");

    let account = wallet_core
        .get_account_public(mint_pda)
        .await
        .map_err(|e| format!("fetch mint PDA: {e:?}"))?;
    let bytes: Vec<u8> = account.data.clone().into();
    if bytes.is_empty() {
        println!("[5] mint PDA is empty — UNEXPECTED");
    } else {
        match OnChainMintDefinition::try_from_slice(&bytes) {
            Ok(state) => println!("[5] mint state = {state:?}"),
            Err(e) => println!(
                "[5] mint PDA holds {} bytes but borsh decode failed: {e}",
                bytes.len()
            ),
        }
    }
    Ok(())
}

