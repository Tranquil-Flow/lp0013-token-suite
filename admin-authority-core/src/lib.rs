//! Core authority model for LP-0013 token authority improvements.
//!
//! The model is deliberately small: one optional authority controls one
//! authority domain. `None` means the authority has been revoked and future
//! privileged actions must fail deterministically.

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Fixed-width account identifier used by LEZ/NSSA-style account references.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    BorshDeserialize,
    BorshSerialize,
    Deserialize,
    Serialize,
)]
pub struct AccountId([u8; 32]);

impl AccountId {
    /// Creates an account identifier from its canonical 32-byte representation.
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the canonical 32-byte representation.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Authority domains supported by the token authority extension.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    BorshDeserialize,
    BorshSerialize,
    Deserialize,
    Serialize,
)]
pub enum AuthorityType {
    /// Permission to mint additional supply.
    MintTokens,
}

/// Documented deterministic error codes for authority operations.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum TokenError {
    /// The requested action requires an authority, but it has been revoked.
    #[error("authority has been revoked")]
    AuthorityRevoked,

    /// The signer does not match the currently configured authority.
    #[error("signer is not the configured authority")]
    UnauthorizedAuthority,

    /// Minting or transfer amount must be non-zero.
    #[error("amount must be greater than zero")]
    ZeroAmount,

    /// Total minted supply would overflow its u128 representation.
    #[error("token supply overflow")]
    SupplyOverflow,

    /// Token account balance would overflow its u128 representation.
    #[error("token balance overflow")]
    BalanceOverflow,
}

/// Current authority state for one authority domain.
#[derive(Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct AuthorityInfo {
    authority_type: AuthorityType,
    current_authority: Option<AccountId>,
}

impl AuthorityInfo {
    /// Creates authority state at token initialization.
    ///
    /// Passing `None` creates a fixed-supply/revoked state immediately.
    pub const fn new(authority_type: AuthorityType, current_authority: Option<AccountId>) -> Self {
        Self {
            authority_type,
            current_authority,
        }
    }

    /// Returns the authority domain controlled by this record.
    pub const fn authority_type(&self) -> AuthorityType {
        self.authority_type
    }

    /// Returns the current authority, or `None` if it has been revoked.
    pub const fn current_authority(&self) -> Option<AccountId> {
        self.current_authority
    }

    /// Returns true when `signer` is the current non-revoked authority.
    pub fn authorizes(&self, signer: &AccountId) -> bool {
        self.current_authority == Some(*signer)
    }

    /// Requires `signer` to be the current authority.
    pub fn require_authority(&self, signer: &AccountId) -> Result<(), TokenError> {
        match self.current_authority {
            None => Err(TokenError::AuthorityRevoked),
            Some(current) if current == *signer => Ok(()),
            Some(_) => Err(TokenError::UnauthorizedAuthority),
        }
    }

    /// Atomically rotates the authority to `new_authority`.
    ///
    /// On failure, this method leaves prior state unchanged.
    pub fn rotate(
        &mut self,
        signer: &AccountId,
        new_authority: AccountId,
    ) -> Result<(), TokenError> {
        self.require_authority(signer)?;
        self.current_authority = Some(new_authority);
        Ok(())
    }

    /// Atomically revokes the authority.
    ///
    /// On success, the state becomes fixed/revoked. On failure, prior state is
    /// unchanged.
    pub fn revoke(&mut self, signer: &AccountId) -> Result<(), TokenError> {
        self.require_authority(signer)?;
        self.current_authority = None;
        Ok(())
    }
}

/// Returns the crate name for scaffold smoke checks.
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alice() -> AccountId {
        AccountId::new([0x11; 32])
    }

    fn bob() -> AccountId {
        AccountId::new([0x22; 32])
    }

    fn charlie() -> AccountId {
        AccountId::new([0x33; 32])
    }

    #[test]
    fn exposes_crate_name() {
        assert!(!super::crate_name().is_empty());
    }

    #[test]
    fn mint_authority_is_set_at_initialization_and_can_authorize_minting() {
        let authority = AuthorityInfo::new(AuthorityType::MintTokens, Some(alice()));

        assert_eq!(authority.authority_type(), AuthorityType::MintTokens);
        assert_eq!(authority.current_authority(), Some(alice()));
        assert!(authority.authorizes(&alice()));
        assert!(!authority.authorizes(&bob()));
    }

    #[test]
    fn authority_rotation_is_atomic_and_replaces_the_current_authority() {
        let mut authority = AuthorityInfo::new(AuthorityType::MintTokens, Some(alice()));

        authority
            .rotate(&alice(), bob())
            .expect("current authority can rotate");

        assert_eq!(authority.current_authority(), Some(bob()));
        assert!(!authority.authorizes(&alice()));
        assert!(authority.authorizes(&bob()));
    }

    #[test]
    fn unauthorized_rotation_leaves_prior_authority_unchanged() {
        let mut authority = AuthorityInfo::new(AuthorityType::MintTokens, Some(alice()));

        let err = authority
            .rotate(&charlie(), bob())
            .expect_err("non-authority cannot rotate");

        assert_eq!(err, TokenError::UnauthorizedAuthority);
        assert_eq!(authority.current_authority(), Some(alice()));
        assert!(authority.authorizes(&alice()));
        assert!(!authority.authorizes(&bob()));
    }

    #[test]
    fn authority_revocation_is_atomic_and_rejects_future_minting_deterministically() {
        let mut authority = AuthorityInfo::new(AuthorityType::MintTokens, Some(alice()));

        authority
            .revoke(&alice())
            .expect("current authority can revoke");

        assert_eq!(authority.current_authority(), None);
        assert!(!authority.authorizes(&alice()));
        assert_eq!(
            authority.require_authority(&alice()),
            Err(TokenError::AuthorityRevoked)
        );
        assert_eq!(
            authority.rotate(&alice(), bob()),
            Err(TokenError::AuthorityRevoked)
        );
        assert_eq!(authority.current_authority(), None);
    }

    #[test]
    fn revocation_by_non_authority_leaves_prior_authority_unchanged() {
        let mut authority = AuthorityInfo::new(AuthorityType::MintTokens, Some(alice()));

        let err = authority
            .revoke(&bob())
            .expect_err("non-authority cannot revoke");

        assert_eq!(err, TokenError::UnauthorizedAuthority);
        assert_eq!(authority.current_authority(), Some(alice()));
    }
}
