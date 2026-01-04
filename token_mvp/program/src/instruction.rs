
use {
    solana_pubkey::Pubkey,
};

/// Instructions supported by the token program.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenInstruction {
    /// Initialize a new mint
    InitializeMint {
        /// Number of base 10 digits to the right of the decimal place
        decimals: u8,
        /// Authority that can mint new tokens
        mint_authority: Pubkey,
    },
    /// Initialize a new token account
    InitializeAccount,
    /// Transfer tokens from one account to another
    Transfer {
        /// Amount of tokens to transfer
        amount: u64,
    },
    /// Mint new tokens to an account
    MintTo {
        /// Amount of tokens to mint
        amount: u64,
    },
    /// Burn tokens from an account
    Burn {
        /// Amount of tokens to burn
        amount: u64,
    },
}


impl TokenInstruction {
    /// Unpacks a byte buffer into a TokenInstruction
    pub fn unpack(_input: &[u8]) -> Result<Self, solana_program_error::ProgramError> {
        // TODO: Implement instruction unpacking
        Err(solana_program_error::ProgramError::InvalidInstructionData)
    }
}