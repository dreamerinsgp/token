
use {
    arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs},
    solana_program_error::ProgramError,
    solana_program_option::COption,
    solana_program_pack::{IsInitialized, Pack, Sealed},
    solana_pubkey::{Pubkey, PUBKEY_BYTES},
};

/// Mint data.
#[derive(Debug, Clone, PartialEq)]
pub struct Mint {
    /// Optional authority that can mint new tokens
    pub mint_authority: COption<Pubkey>,
    /// Total supply of tokens
    pub supply: u64,
    /// Number of base 10 digits to the right of the decimal place
    pub decimals: u8,
    /// Is `true` if this structure has been initialized
    pub is_initialized: bool,
    /// Optional authority to freeze token accounts
    pub freeze_authority: COption<Pubkey>,
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
    
    /// Unpacks a Mint from a byte slice without checking initialization state
    pub fn unpack_unchecked(src: &[u8]) -> Result<Self, ProgramError> {
        Self::unpack_from_slice(src)
    }
    
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 82];
        let (
            mint_authority_dst,
            supply_dst,
            decimals_dst,
            is_initialized_dst,
            freeze_authority_dst,
        ) = mut_array_refs![dst, 36, 8, 1, 1, 36];
        let &Mint {
            ref mint_authority,
            supply,
            decimals,
            is_initialized,
            ref freeze_authority,
        } = self;
        pack_coption_key(mint_authority, mint_authority_dst);
        *supply_dst = supply.to_le_bytes();
        decimals_dst[0] = decimals;
        is_initialized_dst[0] = is_initialized as u8;
        pack_coption_key(freeze_authority, freeze_authority_dst);
    }
    
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let src = array_ref![src, 0, 82];
        let (mint_authority, supply, decimals, is_initialized, freeze_authority) =
            array_refs![src, 36, 8, 1, 1, 36];
        let mint_authority = unpack_coption_key(mint_authority)?;
        let supply = u64::from_le_bytes(*supply);
        let decimals = decimals[0];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let freeze_authority = unpack_coption_key(freeze_authority)?;
        Ok(Mint {
            mint_authority,
            supply,
            decimals,
            is_initialized,
            freeze_authority,
        })
    }
}

/// Helper function to pack a COption<Pubkey> into a 36-byte slice
fn pack_coption_key(src: &COption<Pubkey>, dst: &mut [u8; 36]) {
    let (tag, body) = mut_array_refs![dst, 4, 32];
    match src {
        COption::Some(key) => {
            *tag = [1, 0, 0, 0];
            body.copy_from_slice(key.as_ref());
        }
        COption::None => {
            *tag = [0; 4];
        }
    }
}

/// Helper function to unpack a COption<Pubkey> from a 36-byte slice
fn unpack_coption_key(src: &[u8; 36]) -> Result<COption<Pubkey>, ProgramError> {
    let (tag, body) = array_refs![src, 4, 32];
    match *tag {
        [0, 0, 0, 0] => Ok(COption::None),
        [1, 0, 0, 0] => Ok(COption::Some(Pubkey::new_from_array(*body))),
        _ => Err(ProgramError::InvalidAccountData),
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