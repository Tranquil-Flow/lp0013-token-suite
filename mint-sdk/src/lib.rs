//! Host-side SDK for LP-0013 token authority flows.
//!
//! This SDK gives examples and CLIs a small, stable API over the program
//! instruction surface. A later LEZ-backed client can preserve this API while
//! replacing the in-memory transport.

use admin_authority_core::AccountId;
use mint_program::{Instruction, ProgramError, ProgramState};

/// Evaluator-facing token client for create/mint/rotate/revoke/query flows.
#[derive(Clone, Debug, Default)]
pub struct TokenClient {
    program: ProgramState,
}

impl TokenClient {
    /// Creates a variable-supply mint controlled by `authority`.
    pub fn create_variable_mint(
        &mut self,
        mint: AccountId,
        authority: AccountId,
        decimals: u8,
    ) -> Result<(), ProgramError> {
        self.program.execute(Instruction::CreateMint {
            mint,
            authority: Some(authority),
            decimals,
        })
    }

    /// Creates a fixed-supply mint by starting with revoked authority.
    pub fn create_fixed_mint(&mut self, mint: AccountId, decimals: u8) -> Result<(), ProgramError> {
        self.program.execute(Instruction::CreateMint {
            mint,
            authority: None,
            decimals,
        })
    }

    /// Mints base units to the owner's holding account.
    pub fn mint_to(
        &mut self,
        mint: AccountId,
        signer: AccountId,
        destination_owner: AccountId,
        amount: u128,
    ) -> Result<(), ProgramError> {
        self.program.execute(Instruction::MintTo {
            mint,
            signer,
            destination_owner,
            amount,
        })
    }

    /// Rotates mint authority to `new_authority`.
    pub fn rotate_authority(
        &mut self,
        mint: AccountId,
        signer: AccountId,
        new_authority: AccountId,
    ) -> Result<(), ProgramError> {
        self.program.execute(Instruction::SetMintAuthority {
            mint,
            signer,
            new_authority: Some(new_authority),
        })
    }

    /// Revokes mint authority, fixing future supply.
    pub fn revoke_authority(
        &mut self,
        mint: AccountId,
        signer: AccountId,
    ) -> Result<(), ProgramError> {
        self.program.execute(Instruction::SetMintAuthority {
            mint,
            signer,
            new_authority: None,
        })
    }

    /// Returns current supply if the mint exists.
    pub fn supply(&self, mint: AccountId) -> Option<u128> {
        self.program.mint(&mint).map(|mint| mint.supply())
    }

    /// Returns current authority state if the mint exists.
    pub fn current_authority(&self, mint: AccountId) -> Option<Option<AccountId>> {
        self.program
            .mint(&mint)
            .map(|mint| mint.current_authority())
    }

    /// Returns holding balance if the holding account exists.
    pub fn balance(&self, mint: AccountId, owner: AccountId) -> Option<u128> {
        self.program
            .holding(&mint, &owner)
            .map(|holding| holding.balance())
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
    use mint_program::ProgramError;

    fn mint() -> AccountId {
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

    #[test]
    fn exposes_crate_name() {
        assert!(!super::crate_name().is_empty());
    }

    #[test]
    fn client_drives_variable_supply_authority_lifecycle() {
        let mut client = TokenClient::default();

        client
            .create_variable_mint(mint(), authority(), 6)
            .expect("create variable mint");
        client
            .mint_to(mint(), authority(), owner(), 100)
            .expect("mint by authority");
        client
            .rotate_authority(mint(), authority(), next_authority())
            .expect("rotate authority");

        assert_eq!(
            client.mint_to(mint(), authority(), owner(), 1),
            Err(ProgramError::Token(TokenError::UnauthorizedAuthority))
        );

        client
            .mint_to(mint(), next_authority(), owner(), 25)
            .expect("new authority mints");
        client
            .revoke_authority(mint(), next_authority())
            .expect("revoke authority");

        assert_eq!(
            client.mint_to(mint(), next_authority(), owner(), 1),
            Err(ProgramError::Token(TokenError::AuthorityRevoked))
        );
        assert_eq!(client.supply(mint()), Some(125));
        assert_eq!(client.balance(mint(), owner()), Some(125));
        assert_eq!(client.current_authority(mint()), Some(None));
    }

    #[test]
    fn client_can_create_fixed_supply_mint() {
        let mut client = TokenClient::default();

        client
            .create_fixed_mint(mint(), 6)
            .expect("create fixed mint");

        assert_eq!(client.current_authority(mint()), Some(None));
        assert_eq!(
            client.mint_to(mint(), authority(), owner(), 1),
            Err(ProgramError::Token(TokenError::AuthorityRevoked))
        );
    }
}
