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

// Variant ORDER must match the guest's `#[lez_program]` instruction order
// (`create_mint`, `create_holding`, `mint_to`, `set_mint_authority`), because
// risc0 serde serializes the variant *index* as the on-wire discriminant.
// Account args are NOT carried here (they travel in the message's account
// list); only scalar args become variant fields, so `CreateHolding` is a unit
// variant (the guest's `create_holding` takes no scalar args).
#[derive(Serialize)]
enum Instruction {
    CreateMint {
        decimals: u8,
        initial_authority: Option<[u8; 32]>,
    },
    CreateHolding,
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

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct OnChainTokenHolding {
    owner: [u8; 32],
    balance: u128,
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

    // Poll window is env-configurable. The public testnet can take minutes to
    // include (vs sub-second on localnet), so defaults are generous: 1500ms x
    // 160 = 4 min for success, half that for the expected-rejection case.
    let poll_ms: u64 = std::env::var("LP0013_POLL_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1500);
    let base: u32 = std::env::var("LP0013_POLL_ATTEMPTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(160);
    let poll = Duration::from_millis(poll_ms);
    let attempts = if expect_success { base } else { base / 2 };
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
        format!("did NOT confirm in {attempts}x{poll_ms}ms tx={hash_hex}")
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
    let program = Program::new(elf.clone()).map_err(|e| format!("parse program: {e:?}"))?;
    let program_id = program.id();

    let mint_seed = seed_from_str("lp0013:mint:v1");
    let mint_pda = compute_pda(&program_id, &[&mint_seed]);
    let holding_seed = seed_from_str("lp0013:holding:v1");
    let recipient_holding = compute_pda(&program_id, &[&holding_seed]);
    // Authority/signer account. Defaults to the localnet seeded account but is
    // overridable via LP0013_AUTHORITY so the same driver can run against the
    // public testnet with a faucet-funded account whose signing key lives in
    // the wallet pointed to by NSSA_WALLET_HOME_DIR.
    let authority_id = std::env::var("LP0013_AUTHORITY")
        .unwrap_or_else(|_| "2RHZhw9h534Zr3eq2RGhQete2Hh667foECzXPmSkGni2".to_string());
    let authority_pubkey = AccountId::from_str(&authority_id)
        .map_err(|e| format!("parse authority {authority_id}: {e:?}"))?;
    let authority_bytes: [u8; 32] = *authority_pubkey.value();

    println!("== LP-0013 live lifecycle ==");
    println!("program_id  = {program_id:?}");
    println!("mint_pda    = {mint_pda}");
    println!("authority   = {authority_pubkey}");
    println!("recipient   = {recipient_holding}");
    println!();

    // ---- Step 0: deploy the program, capturing an explorer-resolvable hash ----
    // The `wallet deploy-program` CLI discards the response (fire-and-forget), so
    // it can't surface a deploy tx hash. We replicate the same transaction here
    // and poll for inclusion. ProgramDeploymentTransaction has no signer and
    // affects no accounts, so the deploy itself charges no gas. A re-run on an
    // already-deployed program is skipped by the sequencer (ProgramAlreadyExists),
    // which `create_mint` below then independently confirms.
    let deploy_tx = nssa::ProgramDeploymentTransaction::new(
        nssa::program_deployment_transaction::Message::new(elf.clone()),
    );
    match wallet_core
        .sequencer_client
        .send_transaction(NSSATransaction::ProgramDeployment(deploy_tx))
        .await
    {
        Ok(h) => {
            let hh = format!("{h:?}");
            let poll_ms: u64 = std::env::var("LP0013_POLL_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1500);
            let attempts: u32 = std::env::var("LP0013_POLL_ATTEMPTS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(160);
            let mut confirmed = false;
            for _ in 0..attempts {
                tokio::time::sleep(Duration::from_millis(poll_ms)).await;
                if let Ok(Some(_)) = wallet_core.sequencer_client.get_transaction(h).await {
                    confirmed = true;
                    break;
                }
            }
            let status = if confirmed {
                "confirmed"
            } else {
                "submitted; not seen in poll window (lag or already deployed)"
            };
            println!("[0] deploy_program           {status} tx={hh}");
        }
        Err(e) => {
            println!("[0] deploy_program           submit error (may already be deployed): {e:?}")
        }
    }

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

    // Claim the recipient holding ONCE. This is the only instruction that
    // `init`s the holding PDA; the two mints below then mutate it repeatedly.
    let r2 = submit_signed(
        &wallet_core,
        &program,
        vec![recipient_holding, authority_pubkey],
        authority_pubkey,
        Instruction::CreateHolding,
        "create_holding",
        true,
    )
    .await;
    println!("[2] create_holding           {r2}");

    // Two mints into the SAME holding. With the corrected mutable-holding model
    // these accumulate (60 + 40 = 100) — variable supply is genuinely exercised
    // on chain. (The prior init-on-mint bug would have failed this second mint.)
    let r3 = submit_signed(
        &wallet_core,
        &program,
        vec![mint_pda, recipient_holding, authority_pubkey],
        authority_pubkey,
        Instruction::MintTo { amount: 60 },
        "mint_to(amount=60)",
        true,
    )
    .await;
    println!("[3] mint_to(60)              {r3}");

    let r4 = submit_signed(
        &wallet_core,
        &program,
        vec![mint_pda, recipient_holding, authority_pubkey],
        authority_pubkey,
        Instruction::MintTo { amount: 40 },
        "mint_to(amount=40) [accumulates -> 100]",
        true,
    )
    .await;
    println!("[4] mint_to(40)              {r4}");

    let r5 = submit_signed(
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
    println!("[5] set_mint_authority(None) {r5}");

    // Post-revoke mint into the EXISTING holding. Because the holding already
    // exists (mut, not init), nothing rejects this tx before the guest body
    // runs — so the rejection comes from `require_authority` (error 2008:
    // "authority has been revoked"), the guard the spec mandates. On localnet
    // the sequencer log surfaces that exact 2008 string; the public testnet
    // hides guest logs, so there it manifests as non-inclusion.
    let r6 = submit_signed(
        &wallet_core,
        &program,
        vec![mint_pda, recipient_holding, authority_pubkey],
        authority_pubkey,
        Instruction::MintTo { amount: 7 },
        "mint_to(post-revoke) -> expect guard rejection (2008)",
        false,
    )
    .await;
    println!("[6] mint_to(post-revoke)     {r6}");

    // ---- Readback: mint supply should be 100 (60+40), authority None ----
    let mint_account = wallet_core
        .get_account_public(mint_pda)
        .await
        .map_err(|e| format!("fetch mint PDA: {e:?}"))?;
    let mint_bytes: Vec<u8> = mint_account.data.clone().into();
    if mint_bytes.is_empty() {
        println!("[7] mint PDA is empty — UNEXPECTED");
    } else {
        match OnChainMintDefinition::try_from_slice(&mint_bytes) {
            Ok(state) => {
                let ok = state.supply == 100 && state.authority.current_authority.is_none();
                println!(
                    "[7] mint state = {state:?} {}",
                    if ok {
                        "(supply=100 via accumulation, authority revoked — OK)"
                    } else {
                        "(UNEXPECTED: supply/authority mismatch)"
                    }
                );
            }
            Err(e) => println!(
                "[7] mint PDA holds {} bytes but borsh decode failed: {e}",
                mint_bytes.len()
            ),
        }
    }

    // ---- Readback: holding balance should also be 100 (accumulated) ----
    let holding_account = wallet_core
        .get_account_public(recipient_holding)
        .await
        .map_err(|e| format!("fetch holding PDA: {e:?}"))?;
    let holding_bytes: Vec<u8> = holding_account.data.clone().into();
    if holding_bytes.is_empty() {
        println!("[8] holding PDA is empty — UNEXPECTED (create_holding should have claimed it)");
    } else {
        match OnChainTokenHolding::try_from_slice(&holding_bytes) {
            Ok(state) => {
                let ok = state.balance == 100;
                println!(
                    "[8] holding state = {state:?} {}",
                    if ok {
                        "(balance=100 from two accumulating mints — OK)"
                    } else {
                        "(UNEXPECTED: balance mismatch)"
                    }
                );
            }
            Err(e) => println!(
                "[8] holding PDA holds {} bytes but borsh decode failed: {e}",
                holding_bytes.len()
            ),
        }
    }
    Ok(())
}
