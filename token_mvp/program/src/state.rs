
use {
    solana_program_pack::{IsInitialized, Pack, Sealed},
    solana_pubkey::Pubkey,
};

/// Mint data.
#[derive(Debug, Clone, PartialEq)]
pub struct Mint {
    /// Authority that can mint new tokens
    pub mint_authority: Pubkey,
    /// Total supply of tokens
    pub supply: u64,
    /// Number of base 10 digits to the right of the decimal place
    pub decimals: u8,
    /// Is `true` if this structure has been initialized
    pub is_initialized: bool,
}

/// Account data.
#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    /// The mint associated with this account
    pub mint: Pubkey,
    /// The owner of this account
    pub owner: Pubkey,
    /// The amount of tokens this account holds
    pub amount: u64,
    /// Is `true` if this structure has been initialized
    pub is_initialized: bool,
}


impl IsInitialized for Mint {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for Account {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for Mint {}
impl Pack for Mint {
    const LEN: usize = 82;
    
    fn pack_into_slice(&self, _dst: &mut [u8]) {
        // TODO: Implement packing
    }
    
    fn unpack_from_slice(_src: &[u8]) -> Result<Self, solana_program_error::ProgramError> {
        // TODO: Implement unpacking
        Err(solana_program_error::ProgramError::InvalidAccountData)
    }
}

impl Sealed for Account {}
impl Pack for Account {
    const LEN: usize = 165;
    
    fn pack_into_slice(&self, _dst: &mut [u8]) {
        // TODO: Implement packing
    }
    
    fn unpack_from_slice(_src: &[u8]) -> Result<Self, solana_program_error::ProgramError> {
        // TODO: Implement unpacking
        Err(solana_program_error::ProgramError::InvalidAccountData)
    }
}