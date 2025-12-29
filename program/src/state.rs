//! State transition types

use {
    arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs},
    solana_program_error::ProgramError,
    solana_program_pack::{IsInitialized, Pack, Sealed},
    solana_pubkey::{Pubkey, PUBKEY_BYTES},
};

/// Mint data - simplified for MVP
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mint {
    /// The authority that can mint new tokens
    pub mint_authority: Pubkey,
    /// Total supply of tokens
    pub supply: u64,
    /// Number of base 10 digits to the right of the decimal place
    pub decimals: u8,
    /// Is `true` if this structure has been initialized
    pub is_initialized: bool,
}

impl Sealed for Mint {}
impl IsInitialized for Mint {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Mint {
    const LEN: usize = 82;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, 82];
        let (mint_authority, supply, decimals, is_initialized) = array_refs![src, 32, 8, 1, 1];
        let mint_authority = Pubkey::new_from_array(*mint_authority);
        let supply = u64::from_le_bytes(*supply);
        let decimals = decimals[0];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(Mint {
            mint_authority,
            supply,
            decimals,
            is_initialized,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 82];
        let (mint_authority_dst, supply_dst, decimals_dst, is_initialized_dst) =
            mut_array_refs![dst, 32, 8, 1, 1];
        let &Mint {
            ref mint_authority,
            supply,
            decimals,
            is_initialized,
        } = self;
        mint_authority_dst.copy_from_slice(mint_authority.as_ref());
        *supply_dst = supply.to_le_bytes();
        decimals_dst[0] = decimals;
        is_initialized_dst[0] = is_initialized as u8;
    }
}

/// Account data - simplified for MVP
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

impl Sealed for Account {}
impl IsInitialized for Account {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Account {
    const LEN: usize = 165;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, 165];
        let (mint, owner, amount, is_initialized) = array_refs![src, 32, 32, 8, 1];
        let mint = Pubkey::new_from_array(*mint);
        let owner = Pubkey::new_from_array(*owner);
        let amount = u64::from_le_bytes(*amount);
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(Account {
            mint,
            owner,
            amount,
            is_initialized,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 165];
        let (mint_dst, owner_dst, amount_dst, is_initialized_dst) =
            mut_array_refs![dst, 32, 32, 8, 1];
        let &Account {
            ref mint,
            ref owner,
            amount,
            is_initialized,
        } = self;
        mint_dst.copy_from_slice(mint.as_ref());
        owner_dst.copy_from_slice(owner.as_ref());
        *amount_dst = amount.to_le_bytes();
        is_initialized_dst[0] = is_initialized as u8;
    }
}
