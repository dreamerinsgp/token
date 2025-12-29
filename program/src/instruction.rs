//! Instruction types

use {
    crate::error::TokenError,
    solana_program_error::ProgramError,
    solana_pubkey::Pubkey,
    std::mem::size_of,
};

/// Instructions supported by the minimal token program
#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum TokenInstruction {
    /// Initializes a new mint
    ///
    /// Accounts expected:
    ///   0. `[writable]` The mint to initialize
    ///   1. `[]` Rent sysvar
    InitializeMint {
        /// Number of base 10 digits to the right of the decimal place
        decimals: u8,
        /// The authority to mint tokens
        mint_authority: Pubkey,
    },
    /// Initializes a new account to hold tokens
    ///
    /// Accounts expected:
    ///   0. `[writable]` The account to initialize
    ///   1. `[]` The mint this account will be associated with
    ///   2. `[]` The new account's owner
    ///   3. `[]` Rent sysvar
    InitializeAccount,
    /// Transfers tokens from one account to another
    ///
    /// Accounts expected:
    ///   0. `[writable]` The source account
    ///   1. `[writable]` The destination account
    ///   2. `[signer]` The source account's owner
    Transfer { amount: u64 },
    /// Mints tokens to an account
    ///
    /// Accounts expected:
    ///   0. `[writable]` The mint
    ///   1. `[writable]` The destination account
    ///   2. `[signer]` The mint authority
    MintTo { amount: u64 },
    /// Burns tokens from an account
    ///
    /// Accounts expected:
    ///   0. `[writable]` The source account
    ///   1. `[writable]` The mint
    ///   2. `[signer]` The account owner
    Burn { amount: u64 },
}

impl TokenInstruction {
    /// Unpacks a byte buffer into a TokenInstruction
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        if input.is_empty() {
            return Err(TokenError::InvalidInstruction.into());
        }

        let (&instruction_id, rest) = input
            .split_first()
            .ok_or_else(|| TokenError::InvalidInstruction)?;

        match instruction_id {
            0 => {
                if rest.len() < 1 + PUBKEY_BYTES {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let decimals = rest[0];
                let mint_authority = Pubkey::new_from_array(
                    rest[1..1 + PUBKEY_BYTES]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );
                Ok(TokenInstruction::InitializeMint {
                    decimals,
                    mint_authority,
                })
            }
            1 => Ok(TokenInstruction::InitializeAccount),
            2 => {
                if rest.len() < size_of::<u64>() {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let amount = u64::from_le_bytes(
                    rest[0..8]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );
                Ok(TokenInstruction::Transfer { amount })
            }
            3 => {
                if rest.len() < size_of::<u64>() {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let amount = u64::from_le_bytes(
                    rest[0..8]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );
                Ok(TokenInstruction::MintTo { amount })
            }
            4 => {
                if rest.len() < size_of::<u64>() {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let amount = u64::from_le_bytes(
                    rest[0..8]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );
                Ok(TokenInstruction::Burn { amount })
            }
            _ => Err(TokenError::InvalidInstruction.into()),
        }
    }
}

use solana_pubkey::PUBKEY_BYTES;
use std::convert::TryInto;
