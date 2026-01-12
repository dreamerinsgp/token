
use {
    solana_program_error::ProgramError,
    thiserror::Error,
};

/// Errors that may be returned by the token program.
#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenError {
    /// Account already initialized
    #[error("Account already initialized")]
    AlreadyInitialized,
    
    /// Account not initialized
    #[error("Account not initialized")]
    NotInitialized,
    
    /// Insufficient funds
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    /// Invalid mint
    #[error("Invalid mint")]
    InvalidMint,
    
    /// Mint mismatch
    #[error("Mint mismatch")]
    MintMismatch,
    
    /// Invalid owner
    #[error("Invalid owner")]
    InvalidOwner,
    
    /// Overflow
    #[error("Overflow")]
    Overflow,
    
    /// Not rent exempt
    #[error("Not rent exempt")]
    NotRentExempt,
    
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
}

impl From<TokenError> for ProgramError {
    fn from(e: TokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}