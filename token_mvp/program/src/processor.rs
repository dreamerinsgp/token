use {
    crate::{
        check_program_account,
        error::TokenError,
        instruction::TokenInstruction,
        state::{Account, Mint},
    },
    solana_account_info::{next_account_info, AccountInfo},
    solana_msg::msg,
    solana_program_error::ProgramResult,
    solana_program_option::COption,
    solana_program_pack::Pack,
    solana_pubkey::Pubkey,
    solana_rent::Rent,
    solana_sysvar::SysvarSerialize,
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
        
        // ========== 日志：显示 mint_info 和 mint 的详细信息 ==========
        msg!("=== InitializeMint Debug Info ===");
        msg!("[mint_info] Account Information:");
        msg!("  - key: {}", mint_info.key);
        msg!("  - lamports: {} (rent-exempt balance)", mint_info.lamports());
        msg!("  - data_len(): {} bytes (expected: {} bytes)", mint_info.data_len(), Mint::LEN);
        msg!("  - owner: {} (must be token program)", mint_info.owner);
        msg!("  - executable: {}", mint_info.executable);
        msg!("  - is_signer: {}", mint_info.is_signer);
        msg!("  - is_writable: {} (must be true)", mint_info.is_writable);
        
        msg!("[mint] Struct Contents (before packing):");
        match mint.mint_authority {
            COption::Some(auth) => msg!("  - mint_authority: Some({})", auth),
            COption::None => msg!("  - mint_authority: None"),
        }
        msg!("  - supply: {} (always starts at 0)", mint.supply);
        msg!("  - decimals: {} (decimal places)", mint.decimals);
        msg!("  - is_initialized: {} (will be set to true)", mint.is_initialized);
        match mint.freeze_authority {
            COption::Some(auth) => msg!("  - freeze_authority: Some({})", auth),
            COption::None => msg!("  - freeze_authority: None (no freeze capability)"),
        }
        
        // Pack and save mint data
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;
        
        msg!("[After Pack] mint_info.data now contains serialized Mint struct (82 bytes)");
        msg!("✅ InitializeMint completed successfully");
        msg!("========================================");
        
        Ok(())
    }

    /// Processes an InitializeAccount instruction
    pub fn process_initialize_account(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let _program_id = program_id; // Suppress unused warning for now
        let account_info_iter = &mut accounts.iter();
        
        // Extract accounts in order:
        // 0. [writable] The account to initialize
        // 1. [] The mint this account will be associated with
        // 2. [] The new account's owner/multisignature
        // 3. [] Rent sysvar
        let account_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;
        let rent_sysvar_info = next_account_info(account_info_iter)?;
        
        let account_data_len = account_info.data_len();
        
        // Get rent sysvar
        let rent = Rent::from_account_info(rent_sysvar_info)?;
        
        // Validate account is owned by this program
        check_program_account(account_info)?;
        
        // Validate mint account exists and is initialized
        let mint = Mint::unpack(&mint_info.data.borrow())?;
        if !mint.is_initialized {
            return Err(TokenError::InvalidMint.into());
        }
        
        // Unpack account data (may be uninitialized)
        let mut account = Account::unpack_unchecked(&account_info.data.borrow())?;
        
        // Validate account is not already initialized
        if account.is_initialized {
            return Err(TokenError::AlreadyInitialized.into());
        }
        
        // Validate rent exemption
        if !rent.is_exempt(account_info.lamports(), account_data_len) {
            return Err(TokenError::NotRentExempt.into());
        }
        
        // Validate account data length matches expected size
        if account_data_len != Account::LEN {
            return Err(solana_program_error::ProgramError::InvalidAccountData);
        }
        
        // Set account fields
        account.mint = *mint_info.key;
        account.owner = *owner_info.key;
        account.amount = 0; // Initialize with zero balance
        account.is_initialized = true;
        
        // Log debug information
        msg!("=== InitializeAccount Debug Info ===");
        msg!("[account_info] Account Information:");
        msg!("  - key: {}", account_info.key);
        msg!("  - lamports: {} (rent-exempt balance)", account_info.lamports());
        msg!("  - data_len(): {} bytes (expected: {} bytes)", account_info.data_len(), Account::LEN);
        msg!("  - owner: {} (must be token program)", account_info.owner);
        msg!("  - is_writable: {} (must be true)", account_info.is_writable);
        
        msg!("[mint_info] Mint Account:");
        msg!("  - key: {}", mint_info.key);
        msg!("  - is_initialized: {}", mint.is_initialized);
        
        msg!("[owner_info] Owner Account:");
        msg!("  - key: {}", owner_info.key);
        
        msg!("[account] Struct Contents (before packing):");
        msg!("  - mint: {} (associated mint)", account.mint);
        msg!("  - owner: {} (account owner)", account.owner);
        msg!("  - amount: {} (initial balance, always 0)", account.amount);
        msg!("  - is_initialized: {} (will be set to true)", account.is_initialized);
        
        // Pack and save account data
        Account::pack(account, &mut account_info.data.borrow_mut())?;
        
        msg!("[After Pack] account_info.data now contains serialized Account struct (73 bytes)");
        msg!("✅ InitializeAccount completed successfully");
        msg!("========================================");
        
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
                Self::process_initialize_account(program_id, accounts)
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
