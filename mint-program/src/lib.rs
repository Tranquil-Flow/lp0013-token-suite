//! Runtime-facing instruction semantics for the LP-0013 mint program.
//!
//! This crate contains a pure in-memory execution harness that mirrors the
//! intended LEZ/NSSA instruction behavior. The real account adapter can wrap
//! these deterministic transitions without changing semantics.

use admin_authority_core::{AccountId, TokenError};
use borsh::{BorshDeserialize, BorshSerialize};
use mint_core::{MintDefinition, TokenHolding};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Program-level instructions exposed by the token authority program.
#[derive(Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub enum Instruction {
    /// Initializes a new mint definition.
    CreateMint {
        mint: AccountId,
        authority: Option<AccountId>,
        decimals: u8,
    },

    /// Mints base units into a deterministic destination holding account.
    MintTo {
        mint: AccountId,
        signer: AccountId,
        destination_owner: AccountId,
        amount: u128,
    },

    /// Rotates or revokes the mint authority.
    SetMintAuthority {
        mint: AccountId,
        signer: AccountId,
        new_authority: Option<AccountId>,
    },
}

/// Program-level errors surfaced by instruction execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ProgramError {
    /// A create instruction targeted an already initialized mint.
    #[error("mint already exists")]
    MintAlreadyExists,

    /// An instruction referenced a mint that has not been initialized.
    #[error("mint not found")]
    MintNotFound,

    /// Core token/authority transition rejected the operation.
    #[error(transparent)]
    Token(#[from] TokenError),
}

/// In-memory state used to prove instruction semantics before LEZ integration.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProgramState {
    mints: BTreeMap<AccountId, MintDefinition>,
    holdings: BTreeMap<HoldingKey, TokenHolding>,
}

impl ProgramState {
    /// Executes one instruction atomically.
    pub fn execute(&mut self, instruction: Instruction) -> Result<(), ProgramError> {
        match instruction {
            Instruction::CreateMint {
                mint,
                authority,
                decimals,
            } => self.create_mint(mint, authority, decimals),
            Instruction::MintTo {
                mint,
                signer,
                destination_owner,
                amount,
            } => self.mint_to(mint, signer, destination_owner, amount),
            Instruction::SetMintAuthority {
                mint,
                signer,
                new_authority,
            } => self.set_mint_authority(mint, signer, new_authority),
        }
    }

    /// Reads a mint definition by id.
    pub fn mint(&self, mint: &AccountId) -> Option<&MintDefinition> {
        self.mints.get(mint)
    }

    /// Reads a holding account by mint and owner.
    pub fn holding(&self, mint: &AccountId, owner: &AccountId) -> Option<&TokenHolding> {
        self.holdings.get(&HoldingKey::new(*mint, *owner))
    }

    fn create_mint(
        &mut self,
        mint: AccountId,
        authority: Option<AccountId>,
        decimals: u8,
    ) -> Result<(), ProgramError> {
        if self.mints.contains_key(&mint) {
            return Err(ProgramError::MintAlreadyExists);
        }

        let definition = match authority {
            Some(authority) => MintDefinition::new_variable_supply(authority, decimals),
            None => MintDefinition::new_fixed_supply(decimals),
        };
        self.mints.insert(mint, definition);
        Ok(())
    }

    fn mint_to(
        &mut self,
        mint: AccountId,
        signer: AccountId,
        destination_owner: AccountId,
        amount: u128,
    ) -> Result<(), ProgramError> {
        let mut mint_definition = self
            .mints
            .get(&mint)
            .cloned()
            .ok_or(ProgramError::MintNotFound)?;
        let holding_key = HoldingKey::new(mint, destination_owner);
        let mut holding = self
            .holdings
            .get(&holding_key)
            .cloned()
            .unwrap_or_else(|| TokenHolding::new(destination_owner));

        mint_definition.mint_to(&signer, &mut holding, amount)?;

        self.mints.insert(mint, mint_definition);
        self.holdings.insert(holding_key, holding);
        Ok(())
    }

    fn set_mint_authority(
        &mut self,
        mint: AccountId,
        signer: AccountId,
        new_authority: Option<AccountId>,
    ) -> Result<(), ProgramError> {
        let mut mint_definition = self
            .mints
            .get(&mint)
            .cloned()
            .ok_or(ProgramError::MintNotFound)?;

        match new_authority {
            Some(new_authority) => mint_definition.rotate_authority(&signer, new_authority)?,
            None => mint_definition.revoke_authority(&signer)?,
        }

        self.mints.insert(mint, mint_definition);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct HoldingKey {
    mint: AccountId,
    owner: AccountId,
}

impl HoldingKey {
    const fn new(mint: AccountId, owner: AccountId) -> Self {
        Self { mint, owner }
    }
}

/// Returns the crate name for scaffold smoke checks.
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;
    use admin_authority_core::{AccountId, TokenError};

    fn mint_id() -> AccountId {
        AccountId::new([0x01; 32])
    }

    fn authority() -> AccountId {
        AccountId::new([0xA1; 32])
    }

    fn next_authority() -> AccountId {
        AccountId::new([0xB2; 32])
    }

    fn owner() -> AccountId {
        AccountId::new([0xC3; 32])
    }

    fn stranger() -> AccountId {
        AccountId::new([0xD4; 32])
    }

    #[test]
    fn exposes_crate_name() {
        assert!(!super::crate_name().is_empty());
    }

    #[test]
    fn create_variable_mint_initializes_state_once() {
        let mut runtime = ProgramState::default();

        runtime
            .execute(Instruction::CreateMint {
                mint: mint_id(),
                authority: Some(authority()),
                decimals: 6,
            })
            .expect("mint can be created");

        let mint = runtime.mint(&mint_id()).expect("mint exists");
        assert_eq!(mint.current_authority(), Some(authority()));
        assert_eq!(mint.supply(), 0);
        assert_eq!(mint.decimals(), 6);

        let duplicate = runtime
            .execute(Instruction::CreateMint {
                mint: mint_id(),
                authority: Some(authority()),
                decimals: 6,
            })
            .expect_err("duplicate create rejected");
        assert_eq!(duplicate, ProgramError::MintAlreadyExists);
    }

    #[test]
    fn create_fixed_mint_starts_revoked_and_rejects_minting() {
        let mut runtime = ProgramState::default();

        runtime
            .execute(Instruction::CreateMint {
                mint: mint_id(),
                authority: None,
                decimals: 6,
            })
            .expect("fixed mint can be created");

        let err = runtime
            .execute(Instruction::MintTo {
                mint: mint_id(),
                signer: authority(),
                destination_owner: owner(),
                amount: 1,
            })
            .expect_err("fixed mint rejects future supply");

        assert_eq!(err, ProgramError::Token(TokenError::AuthorityRevoked));
        assert_eq!(runtime.mint(&mint_id()).expect("mint exists").supply(), 0);
        assert!(runtime.holding(&mint_id(), &owner()).is_none());
    }

    #[test]
    fn mint_to_creates_destination_holding_and_accumulates_supply() {
        let mut runtime = ProgramState::default();
        runtime
            .execute(Instruction::CreateMint {
                mint: mint_id(),
                authority: Some(authority()),
                decimals: 6,
            })
            .expect("mint can be created");

        runtime
            .execute(Instruction::MintTo {
                mint: mint_id(),
                signer: authority(),
                destination_owner: owner(),
                amount: 42,
            })
            .expect("authority can mint");

        assert_eq!(runtime.mint(&mint_id()).expect("mint exists").supply(), 42);
        assert_eq!(
            runtime
                .holding(&mint_id(), &owner())
                .expect("holding created")
                .balance(),
            42
        );
    }

    #[test]
    fn variable_supply_allows_repeated_minting_to_same_holding() {
        // This is the contract the ON-CHAIN SPEL program must match (weboko LP-0013
        // reliability point): a variable-supply mint can mint repeatedly to the SAME
        // holding and the balance accumulates — it must NOT be a single-use,
        // init-only holding. After revocation, a further mint to that same existing
        // holding is rejected by the authority guard (AuthorityRevoked), NOT by an
        // AccountAlreadyInitialized / init side effect.
        let mut runtime = ProgramState::default();
        runtime
            .execute(Instruction::CreateMint {
                mint: mint_id(),
                authority: Some(authority()),
                decimals: 6,
            })
            .expect("mint can be created");

        runtime
            .execute(Instruction::MintTo {
                mint: mint_id(),
                signer: authority(),
                destination_owner: owner(),
                amount: 50,
            })
            .expect("first mint to a fresh holding");
        runtime
            .execute(Instruction::MintTo {
                mint: mint_id(),
                signer: authority(),
                destination_owner: owner(),
                amount: 50,
            })
            .expect("second mint to the SAME holding accumulates (variable supply)");

        assert_eq!(runtime.mint(&mint_id()).expect("mint exists").supply(), 100);
        assert_eq!(
            runtime
                .holding(&mint_id(), &owner())
                .expect("holding exists")
                .balance(),
            100,
            "repeated minting to one holding must accumulate, not fail"
        );

        runtime
            .execute(Instruction::SetMintAuthority {
                mint: mint_id(),
                signer: authority(),
                new_authority: None,
            })
            .expect("authority revokes");

        // The holding ALREADY exists here, so a correct rejection must come from the
        // authority guard, not from re-initializing an existing account.
        assert_eq!(
            runtime.execute(Instruction::MintTo {
                mint: mint_id(),
                signer: authority(),
                destination_owner: owner(),
                amount: 1,
            }),
            Err(ProgramError::Token(TokenError::AuthorityRevoked))
        );
        assert_eq!(
            runtime
                .holding(&mint_id(), &owner())
                .expect("holding exists")
                .balance(),
            100,
            "rejected post-revoke mint must not alter the existing holding"
        );
    }

    #[test]
    fn wrong_signer_or_missing_mint_leave_state_unchanged() {
        let mut runtime = ProgramState::default();
        runtime
            .execute(Instruction::CreateMint {
                mint: mint_id(),
                authority: Some(authority()),
                decimals: 6,
            })
            .expect("mint can be created");

        assert_eq!(
            runtime.execute(Instruction::MintTo {
                mint: mint_id(),
                signer: stranger(),
                destination_owner: owner(),
                amount: 5,
            }),
            Err(ProgramError::Token(TokenError::UnauthorizedAuthority))
        );
        assert_eq!(runtime.mint(&mint_id()).expect("mint exists").supply(), 0);
        assert!(runtime.holding(&mint_id(), &owner()).is_none());

        assert_eq!(
            runtime.execute(Instruction::MintTo {
                mint: AccountId::new([0x99; 32]),
                signer: authority(),
                destination_owner: owner(),
                amount: 5,
            }),
            Err(ProgramError::MintNotFound)
        );
    }

    #[test]
    fn rotate_then_revoke_authority_controls_future_minting() {
        let mut runtime = ProgramState::default();
        runtime
            .execute(Instruction::CreateMint {
                mint: mint_id(),
                authority: Some(authority()),
                decimals: 6,
            })
            .expect("mint can be created");

        runtime
            .execute(Instruction::SetMintAuthority {
                mint: mint_id(),
                signer: authority(),
                new_authority: Some(next_authority()),
            })
            .expect("authority rotates");

        assert_eq!(
            runtime.execute(Instruction::MintTo {
                mint: mint_id(),
                signer: authority(),
                destination_owner: owner(),
                amount: 1,
            }),
            Err(ProgramError::Token(TokenError::UnauthorizedAuthority))
        );

        runtime
            .execute(Instruction::MintTo {
                mint: mint_id(),
                signer: next_authority(),
                destination_owner: owner(),
                amount: 1,
            })
            .expect("new authority can mint");

        runtime
            .execute(Instruction::SetMintAuthority {
                mint: mint_id(),
                signer: next_authority(),
                new_authority: None,
            })
            .expect("authority revokes");

        assert_eq!(
            runtime.execute(Instruction::MintTo {
                mint: mint_id(),
                signer: next_authority(),
                destination_owner: owner(),
                amount: 1,
            }),
            Err(ProgramError::Token(TokenError::AuthorityRevoked))
        );
        assert_eq!(runtime.mint(&mint_id()).expect("mint exists").supply(), 1);
    }
}
