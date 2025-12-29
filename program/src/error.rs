//! Error types

use {
    solana_program_error::{ProgramError, ToStr},
    thiserror::Error,
};

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum TokenError {
    /// Account/mint already initialized
    #[error("Already initialized")]
    AlreadyInitialized,
    /// Account/mint not initialized
    #[error("Not initialized")]
    NotInitialized,
    /// Insufficient funds for the operation requested
    #[error("Insufficient funds")]
    InsufficientFunds,
    /// Invalid Mint
    #[error("Invalid Mint")]
    InvalidMint,
    /// Account not associated with this Mint
    #[error("Mint mismatch")]
    MintMismatch,
    /// Owner does not match
    #[error("Invalid owner")]
    InvalidOwner,
    /// Operation overflowed
    #[error("Overflow")]
    Overflow,
    /// Lamport balance below rent-exempt threshold
    #[error("Not rent exempt")]
    NotRentExempt,
}

impl From<TokenError> for ProgramError {
    fn from(e: TokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl ToStr for TokenError {
    fn to_str(&self) -> &'static str {
        match self {
            TokenError::AlreadyInitialized => "Error: Already initialized",
            TokenError::NotInitialized => "Error: Not initialized",
            TokenError::InsufficientFunds => "Error: Insufficient funds",
            TokenError::InvalidMint => "Error: Invalid Mint",
            TokenError::MintMismatch => "Error: Mint mismatch",
            TokenError::InvalidOwner => "Error: Invalid owner",
            TokenError::Overflow => "Error: Overflow",
            TokenError::NotRentExempt => "Error: Not rent exempt",
        }
    }
}
