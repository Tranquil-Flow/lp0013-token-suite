#![no_main]

use nssa_core::account::Data;
use spel_framework::prelude::*;

risc0_zkvm::guest::entry!(main);

/// LP-0013 admin-authority program — SPEL surface mirroring the offline
/// `mint-program` instruction set: `create_mint`, `mint_to`, and
/// `set_mint_authority`. Account shapes match the offline core crates so the
/// SPEL-generated IDL is interchangeable with the hand-written fallback for
/// LP-0017 compatibility patterns.
#[lez_program]
mod admin_authority {
    #[allow(unused_imports)]
    use super::*;

    /// On-chain authority state. `current_authority = None` represents a
    /// revoked / fixed-supply mint after rotation-to-none.
    #[derive(BorshSerialize, BorshDeserialize)]
    #[account_type]
    pub struct AuthorityInfo {
        /// The kind of authority this slot grants. The offline suite ships
        /// `MintTokens` as the only variant; future variants extend without
        /// breaking IDL clients.
        pub authority_type: u8,
        /// `Some(account_id)` for an active authority; `None` for revoked.
        pub current_authority: Option<[u8; 32]>,
    }

    /// On-chain mint state — the authoritative supply + authority record.
    #[derive(BorshSerialize, BorshDeserialize)]
    #[account_type]
    pub struct MintDefinition {
        pub authority: AuthorityInfo,
        pub supply: u128,
        pub decimals: u8,
    }

    /// On-chain holder balance keyed by `(mint, owner)`.
    #[derive(BorshSerialize, BorshDeserialize)]
    #[account_type]
    pub struct TokenHolding {
        pub owner: [u8; 32],
        pub balance: u128,
    }

    /// Initialize a new mint. Pass `initial_authority = None` for a
    /// fixed-supply mint (no further minting allowed). Pass `Some(authority)`
    /// for a variable-supply mint that may rotate or revoke later.
    #[instruction]
    pub fn create_mint(
        #[account(init, pda = literal("lp0013:mint:v1"))] mint: AccountWithMetadata,
        #[account(signer)] payer: AccountWithMetadata,
        decimals: u8,
        initial_authority: Option<[u8; 32]>,
    ) -> SpelResult {
        let mut acc = mint.account.clone();
        let state = MintDefinition {
            authority: AuthorityInfo {
                authority_type: 0, // MintTokens
                current_authority: initial_authority,
            },
            supply: 0,
            decimals,
        };
        let bytes = borsh::to_vec(&state)
            .map_err(|e| SpelError::custom(1001, format!("borsh error: {e}")))?;
        acc.data =
            Data::try_from(bytes).map_err(|_| SpelError::custom(1002, "mint state too big"))?;
        Ok(SpelOutput::execute(vec![acc, payer.account], vec![]))
    }

    /// Mint `amount` units to `recipient_holding` using the same authority,
    /// zero-amount, and overflow rules as the offline `mint-core` model. The
    /// holding account is a program-claimed PDA so the LEZ framework permits
    /// the program to mutate its on-chain state. `init` claims the PDA on
    /// first mint; subsequent mints reuse the same PDA via `mut`.
    #[instruction]
    pub fn mint_to(
        #[account(mut, pda = literal("lp0013:mint:v1"))] mint: AccountWithMetadata,
        #[account(init, pda = literal("lp0013:holding:v1"))] recipient_holding: AccountWithMetadata,
        #[account(signer)] authority: AccountWithMetadata,
        amount: u128,
    ) -> SpelResult {
        if amount == 0 {
            return Err(SpelError::custom(2003, "amount must be greater than zero"));
        }

        let mut mint_acc = mint.account.clone();
        let mut holding_acc = recipient_holding.account.clone();
        let mut mint_state = decode_mint(&mint_acc.data)?;
        require_authority(&mint_state, authority.account_id.value())?;

        let mut holding_state = if holding_acc.data.as_ref().is_empty() {
            TokenHolding {
                owner: *recipient_holding.account_id.value(),
                balance: 0,
            }
        } else {
            decode_holding(&holding_acc.data)?
        };

        let next_supply = mint_state
            .supply
            .checked_add(amount)
            .ok_or_else(|| SpelError::custom(2004, "token supply overflow"))?;
        let next_balance = holding_state
            .balance
            .checked_add(amount)
            .ok_or_else(|| SpelError::custom(2005, "token balance overflow"))?;

        mint_state.supply = next_supply;
        holding_state.balance = next_balance;
        mint_acc.data = encode_data(&mint_state)?;
        holding_acc.data = encode_data(&holding_state)?;

        Ok(SpelOutput::execute(
            vec![mint_acc, holding_acc, authority.account],
            vec![],
        ))
    }

    /// Rotate or revoke the mint authority using the same signer checks as the
    /// offline `mint-core` model. `new_authority = None` fixes supply by
    /// clearing the mint authority.
    #[instruction]
    pub fn set_mint_authority(
        #[account(mut, pda = literal("lp0013:mint:v1"))] mint: AccountWithMetadata,
        #[account(signer)] current_authority: AccountWithMetadata,
        new_authority: Option<[u8; 32]>,
    ) -> SpelResult {
        let mut mint_acc = mint.account.clone();
        let mut mint_state = decode_mint(&mint_acc.data)?;
        require_authority(&mint_state, current_authority.account_id.value())?;
        mint_state.authority.current_authority = new_authority;
        mint_acc.data = encode_data(&mint_state)?;

        Ok(SpelOutput::execute(
            vec![mint_acc, current_authority.account],
            vec![],
        ))
    }

    fn decode_mint(data: &Data) -> Result<MintDefinition, SpelError> {
        if data.as_ref().is_empty() {
            return Err(SpelError::custom(2000, "mint account is not initialized"));
        }
        MintDefinition::try_from_slice(data.as_ref())
            .map_err(|e| SpelError::custom(2001, format!("decode mint state: {e}")))
    }

    fn decode_holding(data: &Data) -> Result<TokenHolding, SpelError> {
        TokenHolding::try_from_slice(data.as_ref())
            .map_err(|e| SpelError::custom(2002, format!("decode token holding: {e}")))
    }

    fn encode_data<T: BorshSerialize>(value: &T) -> Result<Data, SpelError> {
        let bytes = borsh::to_vec(value)
            .map_err(|e| SpelError::custom(2006, format!("borsh encode: {e}")))?;
        Data::try_from(bytes).map_err(|_| SpelError::custom(2007, "account state too big"))
    }

    fn require_authority(mint: &MintDefinition, signer: &[u8; 32]) -> Result<(), SpelError> {
        match mint.authority.current_authority {
            None => Err(SpelError::custom(2008, "authority has been revoked")),
            Some(current) if &current == signer => Ok(()),
            Some(_) => Err(SpelError::custom(
                2009,
                "signer is not the configured authority",
            )),
        }
    }
}
