
use {
    crate::error::TokenError,
    solana_program_error::ProgramError,
    solana_program_option::COption,
    solana_pubkey::{Pubkey, PUBKEY_BYTES},
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
        /// Optional authority to freeze token accounts
        freeze_authority: COption<Pubkey>,
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
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input.split_first().ok_or(TokenError::InvalidInstruction)?;
        Ok(match tag {
            0 => {
                // InitializeMint instruction
                let (&decimals, rest) = rest.split_first().ok_or(TokenError::InvalidInstruction)?;
                let (mint_authority, rest) = Self::unpack_pubkey(rest)?;
                let (freeze_authority, _rest) = Self::unpack_pubkey_option(rest)?;
                Self::InitializeMint {
                    decimals,
                    mint_authority,
                    freeze_authority,
                }
            }
            1 => Self::InitializeAccount,
            3 => {
                // Transfer instruction
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(TokenError::InvalidInstruction)?;
                Self::Transfer { amount }
            }
            7 => {
                // MintTo instruction
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(TokenError::InvalidInstruction)?;
                Self::MintTo { amount }
            }
            8 => {
                // Burn instruction
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(TokenError::InvalidInstruction)?;
                Self::Burn { amount }
            }
            _ => return Err(TokenError::InvalidInstruction.into()),
        })
    }

    /// Unpacks a Pubkey from the input slice
    fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
        if input.len() >= PUBKEY_BYTES {
            let (key, rest) = input.split_at(PUBKEY_BYTES);
            let pk = Pubkey::try_from(key).map_err(|_| TokenError::InvalidInstruction)?;
            Ok((pk, rest))
        } else {
            Err(TokenError::InvalidInstruction.into())
        }
    }

    /// Unpacks a COption<Pubkey> from the input slice
    fn unpack_pubkey_option(input: &[u8]) -> Result<(COption<Pubkey>, &[u8]), ProgramError> {
        match input.split_first() {
            Some((&0, rest)) => Ok((COption::None, rest)),
            Some((&1, rest)) if rest.len() >= PUBKEY_BYTES => {
                let (key, rest) = rest.split_at(PUBKEY_BYTES);
                let pk = Pubkey::try_from(key).map_err(|_| TokenError::InvalidInstruction)?;
                Ok((COption::Some(pk), rest))
            }
            _ => Err(TokenError::InvalidInstruction.into()),
        }
    }
}