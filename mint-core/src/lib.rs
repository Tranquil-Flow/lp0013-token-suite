//! Pure mint state transitions for LP-0013.
//!
//! This crate contains runtime-agnostic state updates for fixed and variable
//! supply tokens. LEZ/NSSA account plumbing belongs in `mint-program`; this
//! crate only decides whether a transition is valid and applies it atomically.

use admin_authority_core::{AccountId, AuthorityInfo, AuthorityType, TokenError};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Mint definition state controlled by the mint program.
#[derive(Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct MintDefinition {
    authority: AuthorityInfo,
    supply: u128,
    decimals: u8,
}

impl MintDefinition {
    /// Creates a variable-supply mint with an active mint authority.
    pub const fn new_variable_supply(authority: AccountId, decimals: u8) -> Self {
        Self {
            authority: AuthorityInfo::new(AuthorityType::MintTokens, Some(authority)),
            supply: 0,
            decimals,
        }
    }

    /// Creates a fixed-supply mint by starting in the revoked authority state.
    pub const fn new_fixed_supply(decimals: u8) -> Self {
        Self {
            authority: AuthorityInfo::new(AuthorityType::MintTokens, None),
            supply: 0,
            decimals,
        }
    }

    /// Returns the authority domain used by this mint.
    pub const fn authority_type(&self) -> AuthorityType {
        self.authority.authority_type()
    }

    /// Returns total minted supply in base units.
    pub const fn supply(&self) -> u128 {
        self.supply
    }

    /// Returns display decimal places.
    pub const fn decimals(&self) -> u8 {
        self.decimals
    }

    /// Returns the current mint authority, or `None` once fixed/revoked.
    pub const fn current_authority(&self) -> Option<AccountId> {
        self.authority.current_authority()
    }

    /// Atomically mints `amount` base units into `destination`.
    pub fn mint_to(
        &mut self,
        signer: &AccountId,
        destination: &mut TokenHolding,
        amount: u128,
    ) -> Result<(), TokenError> {
        if amount == 0 {
            return Err(TokenError::ZeroAmount);
        }

        self.authority.require_authority(signer)?;

        let next_supply = self
            .supply
            .checked_add(amount)
            .ok_or(TokenError::SupplyOverflow)?;
        let next_balance = destination
            .balance
            .checked_add(amount)
            .ok_or(TokenError::BalanceOverflow)?;

        self.supply = next_supply;
        destination.balance = next_balance;
        Ok(())
    }

    /// Atomically rotates the mint authority.
    pub fn rotate_authority(
        &mut self,
        signer: &AccountId,
        new_authority: AccountId,
    ) -> Result<(), TokenError> {
        self.authority.rotate(signer, new_authority)
    }

    /// Atomically revokes the mint authority, fixing supply permanently.
    pub fn revoke_authority(&mut self, signer: &AccountId) -> Result<(), TokenError> {
        self.authority.revoke(signer)
    }
}

/// Token holding state for a destination owner.
#[derive(Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct TokenHolding {
    owner: AccountId,
    balance: u128,
}

impl TokenHolding {
    /// Creates an empty holding account for `owner`.
    pub const fn new(owner: AccountId) -> Self {
        Self { owner, balance: 0 }
    }

    /// Returns the holding owner.
    pub const fn owner(&self) -> AccountId {
        self.owner
    }

    /// Returns the base-unit token balance.
    pub const fn balance(&self) -> u128 {
        self.balance
    }
}

/// Returns the crate name for scaffold smoke checks.
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;
    use admin_authority_core::{AccountId, AuthorityType, TokenError};

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
    fn variable_supply_mint_requires_current_authority_and_updates_supply_and_balance() {
        let mut mint = MintDefinition::new_variable_supply(authority(), 6);
        let mut holding = TokenHolding::new(owner());

        mint.mint_to(&authority(), &mut holding, 1_000)
            .expect("authority can mint");

        assert_eq!(mint.authority_type(), AuthorityType::MintTokens);
        assert_eq!(mint.supply(), 1_000);
        assert_eq!(mint.decimals(), 6);
        assert_eq!(holding.owner(), owner());
        assert_eq!(holding.balance(), 1_000);
    }

    #[test]
    fn minting_with_wrong_authority_leaves_state_unchanged() {
        let mut mint = MintDefinition::new_variable_supply(authority(), 6);
        let mut holding = TokenHolding::new(owner());

        let err = mint
            .mint_to(&next_authority(), &mut holding, 1_000)
            .expect_err("wrong authority cannot mint");

        assert_eq!(err, TokenError::UnauthorizedAuthority);
        assert_eq!(mint.supply(), 0);
        assert_eq!(holding.balance(), 0);
    }

    #[test]
    fn fixed_supply_mint_rejects_minting_deterministically() {
        let mut mint = MintDefinition::new_fixed_supply(6);
        let mut holding = TokenHolding::new(owner());

        let err = mint
            .mint_to(&authority(), &mut holding, 1)
            .expect_err("fixed supply mint cannot mint more tokens");

        assert_eq!(err, TokenError::AuthorityRevoked);
        assert_eq!(mint.supply(), 0);
        assert_eq!(holding.balance(), 0);
    }

    #[test]
    fn zero_amount_mint_is_rejected_without_state_change() {
        let mut mint = MintDefinition::new_variable_supply(authority(), 6);
        let mut holding = TokenHolding::new(owner());

        let err = mint
            .mint_to(&authority(), &mut holding, 0)
            .expect_err("zero amount is invalid");

        assert_eq!(err, TokenError::ZeroAmount);
        assert_eq!(mint.supply(), 0);
        assert_eq!(holding.balance(), 0);
    }

    #[test]
    fn supply_overflow_is_rejected_without_balance_change() {
        let mut mint = MintDefinition::new_variable_supply(authority(), 6);
        let mut holding = TokenHolding::new(owner());

        mint.mint_to(&authority(), &mut holding, u128::MAX)
            .expect("first max mint fits exactly");
        let err = mint
            .mint_to(&authority(), &mut holding, 1)
            .expect_err("second mint would overflow supply");

        assert_eq!(err, TokenError::SupplyOverflow);
        assert_eq!(mint.supply(), u128::MAX);
        assert_eq!(holding.balance(), u128::MAX);
    }

    #[test]
    fn authority_rotation_and_revocation_delegate_to_core_lifecycle() {
        let mut mint = MintDefinition::new_variable_supply(authority(), 6);
        let mut holding = TokenHolding::new(owner());

        mint.rotate_authority(&authority(), next_authority())
            .expect("current authority can rotate");
        assert_eq!(
            mint.mint_to(&authority(), &mut holding, 1),
            Err(TokenError::UnauthorizedAuthority)
        );

        mint.mint_to(&next_authority(), &mut holding, 1)
            .expect("new authority can mint");
        mint.revoke_authority(&next_authority())
            .expect("new authority can revoke");

        assert_eq!(
            mint.mint_to(&next_authority(), &mut holding, 1),
            Err(TokenError::AuthorityRevoked)
        );
        assert_eq!(mint.supply(), 1);
        assert_eq!(holding.balance(), 1);
    }
}
