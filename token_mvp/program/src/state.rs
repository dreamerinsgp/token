
use {
    arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs},
    solana_program_error::ProgramError,
    solana_program_option::COption,
    solana_program_pack::{IsInitialized, Pack, Sealed},
    solana_pubkey::Pubkey,
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
    /// For native accounts, the rent-exempt reserve amount
    pub is_native: COption<u64>,
    /// Optional delegate account that can transfer tokens on behalf of the owner
    pub delegate: COption<Pubkey>,
    /// The amount of tokens the delegate is approved for
    pub delegated_amount: u64,
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

impl Mint {
    /// Unpacks a Mint from a byte slice without checking initialization state
    pub fn unpack_unchecked(src: &[u8]) -> Result<Self, ProgramError> {
        <Self as Pack>::unpack_from_slice(src)
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

/// Helper function to pack a COption<u64> into a 36-byte slice
fn pack_coption_u64(src: &COption<u64>, dst: &mut [u8; 36]) {
    let (tag, body) = mut_array_refs![dst, 4, 32];
    match src {
        COption::Some(value) => {
            *tag = [1, 0, 0, 0];
            let value_bytes = value.to_le_bytes();
            body[..8].copy_from_slice(&value_bytes);
            // Zero out the rest
            for i in 8..32 {
                body[i] = 0;
            }
        }
        COption::None => {
            *tag = [0; 4];
            body.fill(0);
        }
    }
}

/// Helper function to unpack a COption<u64> from a 36-byte slice
fn unpack_coption_u64(src: &[u8; 36]) -> Result<COption<u64>, ProgramError> {
    let (tag, body) = array_refs![src, 4, 32];
    match *tag {
        [0, 0, 0, 0] => Ok(COption::None),
        [1, 0, 0, 0] => {
            let value_bytes: [u8; 8] = body[..8].try_into().map_err(|_| ProgramError::InvalidAccountData)?;
            Ok(COption::Some(u64::from_le_bytes(value_bytes)))
        }
        _ => Err(ProgramError::InvalidAccountData),
    }
}

impl Sealed for Account {}
impl Pack for Account {
    const LEN: usize = 181; // mint (32) + owner (32) + amount (8) + is_initialized (1) + is_native (36) + delegate (36) + delegated_amount (8)
    
    fn pack_into_slice(&self, dst: &mut [u8]) {
        // Pack fields in order: mint (32) + owner (32) + amount (8) + is_initialized (1) + is_native (36) + delegate (36) + delegated_amount (8)
        let dst = array_mut_ref![dst, 0, 181];
        
        // Pack mint (bytes 0-31)
        dst[0..32].copy_from_slice(self.mint.as_ref());
        
        // Pack owner (bytes 32-63)
        dst[32..64].copy_from_slice(self.owner.as_ref());
        
        // Pack amount (bytes 64-71)
        dst[64..72].copy_from_slice(&self.amount.to_le_bytes());
        
        // Pack is_initialized (byte 72)
        dst[72] = self.is_initialized as u8;
        
        // Pack is_native (bytes 73-108)
        let is_native_dst = array_mut_ref![dst, 73, 36];
        pack_coption_u64(&self.is_native, is_native_dst);
        
        // Pack delegate (bytes 109-144)
        let delegate_dst = array_mut_ref![dst, 109, 36];
        pack_coption_key(&self.delegate, delegate_dst);
        
        // Pack delegated_amount (bytes 145-152)
        dst[145..153].copy_from_slice(&self.delegated_amount.to_le_bytes());
    }
    
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let src = array_ref![src, 0, 181];
        
        // Unpack mint (bytes 0-31)
        let mint = Pubkey::new_from_array(*array_ref![src, 0, 32]);
        
        // Unpack owner (bytes 32-63)
        let owner = Pubkey::new_from_array(*array_ref![src, 32, 32]);
        
        // Unpack amount (bytes 64-71)
        let amount = u64::from_le_bytes(*array_ref![src, 64, 8]);
        
        // Unpack is_initialized (byte 72)
        let is_initialized = match src[72] {
            0 => false,
            1 => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        
        // Unpack is_native (bytes 73-108)
        let is_native_src = array_ref![src, 73, 36];
        let is_native = unpack_coption_u64(is_native_src)?;
        
        // Unpack delegate (bytes 109-144)
        let delegate_src = array_ref![src, 109, 36];
        let delegate = unpack_coption_key(delegate_src)?;
        
        // Unpack delegated_amount (bytes 145-152)
        let delegated_amount = u64::from_le_bytes(*array_ref![src, 145, 8]);
        
        Ok(Account {
            mint,
            owner,
            amount,
            is_initialized,
            is_native,
            delegate,
            delegated_amount,
        })
    }
}

impl Account {
    /// Unpacks an Account from a byte slice without checking initialization state
    pub fn unpack_unchecked(src: &[u8]) -> Result<Self, ProgramError> {
        <Self as Pack>::unpack_from_slice(src)
    }
}