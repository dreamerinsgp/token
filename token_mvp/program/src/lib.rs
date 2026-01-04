#![allow(clippy::arithmetic_side_effects)]
#![deny(missing_docs)]

//! Solana Program Library Token
//! 
//! An ERC20-like Token program for the Solana blockchain.

/// Error types for the token program.
pub mod error;
/// Instruction types for the token program.
pub mod instruction;
/// Instruction processor for the token program.
pub mod processor;
/// State types for the token program.
pub mod state;


#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

use solana_program_error::ProgramError;


/// Program ID - This should be set to your actual program ID
pub fn id() -> solana_pubkey::Pubkey {
    solana_pubkey::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
}



/// Check that the account is owned by the token program
pub fn check_id(account: &solana_pubkey::Pubkey) -> bool {
    account == &id()
}



/// Check that the account is owned by the token program
pub fn check_program_account(
    account: &solana_account_info::AccountInfo
) -> Result<(), ProgramError> {
    if account.owner != &id() {
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}