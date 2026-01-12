use {
    crate::{
        check_program_account,
        error::TokenError,
        instruction::TokenInstruction,
        state::Mint,
    },
    solana_account_info::{next_account_info, AccountInfo},
    solana_program_error::ProgramResult,
    solana_program_option::COption,
    solana_program_pack::Pack,
    solana_pubkey::Pubkey,
    solana_rent::Rent,
    solana_sysvar::Sysvar,
};

/// Program state handler.
pub struct Processor {}

impl Processor {
    /// Processes an InitializeMint instruction
    pub fn process_initialize_mint(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        decimals: u8,
        mint_authority: Pubkey,
        freeze_authority: COption<Pubkey>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_info = next_account_info(account_info_iter)?;
        let mint_data_len = mint_info.data_len();
        
        // Get rent sysvar
        let rent_sysvar_info = next_account_info(account_info_iter)?;
        let rent = Rent::from_account_info(rent_sysvar_info)?;
        
        // Validate account is owned by this program
        check_program_account(mint_info)?;
        
        // Unpack mint account data
        let mut mint = Mint::unpack_unchecked(&mint_info.data.borrow())?;
        
        // Validate mint is not already initialized
        if mint.is_initialized {
            return Err(TokenError::AlreadyInitialized.into());
        }
        
        // Validate rent exemption
        if !rent.is_exempt(mint_info.lamports(), mint_data_len) {
            return Err(TokenError::NotRentExempt.into());
        }
        
        // Validate account data length matches expected size
        if mint_data_len != Mint::LEN {
            return Err(solana_program_error::ProgramError::InvalidAccountData);
        }
        
        // Set mint fields
        mint.mint_authority = COption::Some(mint_authority);
        mint.decimals = decimals;
        mint.supply = 0;
        mint.is_initialized = true;
        mint.freeze_authority = freeze_authority;
        
        // Pack and save mint data
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;
        
        Ok(())
    }

    /// Main instruction processing router
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = TokenInstruction::unpack(instruction_data)?;
        
        match instruction {
            TokenInstruction::InitializeMint {
                decimals,
                mint_authority,
                freeze_authority,
            } => {
                Self::process_initialize_mint(
                    program_id,
                    accounts,
                    decimals,
                    mint_authority,
                    freeze_authority,
                )
            }
            TokenInstruction::InitializeAccount => {
                // TODO: Implement InitializeAccount handler
                Err(TokenError::NotInitialized.into())
            }
            TokenInstruction::Transfer { .. } => {
                // TODO: Implement Transfer handler
                Err(TokenError::NotInitialized.into())
            }
            TokenInstruction::MintTo { .. } => {
                // TODO: Implement MintTo handler
                Err(TokenError::NotInitialized.into())
            }
            TokenInstruction::Burn { .. } => {
                // TODO: Implement Burn handler
                Err(TokenError::NotInitialized.into())
            }
        }
    }
}
