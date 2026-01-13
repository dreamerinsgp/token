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

    /// Processes a Transfer instruction
    pub fn process_transfer(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let _program_id = program_id; // Suppress unused warning for now
        let account_info_iter = &mut accounts.iter();
        
        // Extract accounts in order:
        // 0. [writable] The source account
        // 1. [writable] The destination account
        // 2. [signer] The source account's owner/delegate
        let source_info = next_account_info(account_info_iter)?;
        let destination_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        
        // Validate accounts are owned by this program
        check_program_account(source_info)?;
        check_program_account(destination_info)?;
        
        // Validate accounts are writable
        if !source_info.is_writable {
            return Err(TokenError::InvalidOwner.into());
        }
        if !destination_info.is_writable {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Unpack source account
        let mut source_account = Account::unpack(&source_info.data.borrow())?;
        
        // Validate source account is initialized
        if !source_account.is_initialized {
            return Err(TokenError::NotInitialized.into());
        }
        
        // Unpack destination account
        let mut destination_account = Account::unpack(&destination_info.data.borrow())?;
        
        // Validate destination account is initialized
        if !destination_account.is_initialized {
            return Err(TokenError::NotInitialized.into());
        }
        
        // Validate mint match
        if source_account.mint != destination_account.mint {
            return Err(TokenError::MintMismatch.into());
        }
        
        // Validate authority is the owner (for now, we don't support delegate)
        if source_account.owner != *authority_info.key {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Validate authority is a signer
        if !authority_info.is_signer {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Check for self-transfer (no-op)
        if source_info.key == destination_info.key {
            msg!("Self-transfer detected, no-op");
            return Ok(());
        }
        
        // Validate sufficient balance
        if source_account.amount < amount {
            return Err(TokenError::InsufficientFunds.into());
        }
        
        // Perform transfer
        source_account.amount = source_account.amount
            .checked_sub(amount)
            .ok_or(TokenError::Overflow)?;
        
        destination_account.amount = destination_account.amount
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;
        
        // Log debug information
        msg!("=== Transfer Debug Info ===");
        msg!("[source_account]");
        msg!("  - key: {}", source_info.key);
        msg!("  - owner: {}", source_account.owner);
        msg!("  - mint: {}", source_account.mint);
        msg!("  - amount before: {} (will subtract {})", 
             source_account.amount + amount, amount);
        msg!("  - amount after: {}", source_account.amount);
        
        msg!("[destination_account]");
        msg!("  - key: {}", destination_info.key);
        msg!("  - owner: {}", destination_account.owner);
        msg!("  - mint: {}", destination_account.mint);
        msg!("  - amount before: {} (will add {})", 
             destination_account.amount - amount, amount);
        msg!("  - amount after: {}", destination_account.amount);
        
        msg!("[authority]");
        msg!("  - key: {} (must be source owner)", authority_info.key);
        msg!("  - is_signer: {}", authority_info.is_signer);
        
        // Pack and save account data
        Account::pack(source_account, &mut source_info.data.borrow_mut())?;
        Account::pack(destination_account, &mut destination_info.data.borrow_mut())?;
        
        msg!("✅ Transfer completed successfully");
        msg!("========================================");
        
        Ok(())
    }

    /// Processes a MintTo instruction
    pub fn process_mint_to(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let _program_id = program_id; // Suppress unused warning for now
        let account_info_iter = &mut accounts.iter();
        
        // Extract accounts in order:
        // 0. [writable] The mint
        // 1. [writable] The account to mint tokens to
        // 2. [signer] The mint's minting authority
        let mint_info = next_account_info(account_info_iter)?;
        let destination_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        
        // Validate accounts are owned by this program
        check_program_account(mint_info)?;
        check_program_account(destination_info)?;
        
        // Validate accounts are writable
        if !mint_info.is_writable {
            return Err(TokenError::InvalidOwner.into());
        }
        if !destination_info.is_writable {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Unpack mint account
        let mut mint = Mint::unpack(&mint_info.data.borrow())?;
        
        // Validate mint is initialized
        if !mint.is_initialized {
            return Err(TokenError::InvalidMint.into());
        }
        
        // Validate mint authority exists and matches
        match mint.mint_authority {
            COption::None => return Err(TokenError::InvalidMint.into()),
            COption::Some(auth) => {
                if auth != *authority_info.key {
                    return Err(TokenError::InvalidOwner.into());
                }
            }
        }
        
        // Validate authority is a signer
        if !authority_info.is_signer {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Unpack destination account
        let mut destination_account = Account::unpack(&destination_info.data.borrow())?;
        
        // Validate destination account is initialized
        if !destination_account.is_initialized {
            return Err(TokenError::NotInitialized.into());
        }
        
        // Validate mint match
        if destination_account.mint != *mint_info.key {
            return Err(TokenError::MintMismatch.into());
        }
        
        // Perform minting: update supply and balance
        mint.supply = mint.supply
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;
        
        destination_account.amount = destination_account.amount
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;
        
        // Log debug information
        msg!("=== MintTo Debug Info ===");
        msg!("[mint_info] Mint Account:");
        msg!("  - key: {}", mint_info.key);
        msg!("  - supply before: {} (will add {})", mint.supply - amount, amount);
        msg!("  - supply after: {}", mint.supply);
        msg!("  - mint_authority: {:?}", mint.mint_authority);
        
        msg!("[destination_info] Destination Account:");
        msg!("  - key: {}", destination_info.key);
        msg!("  - owner: {}", destination_account.owner);
        msg!("  - mint: {}", destination_account.mint);
        msg!("  - amount before: {} (will add {})", destination_account.amount - amount, amount);
        msg!("  - amount after: {}", destination_account.amount);
        
        msg!("[authority_info] Mint Authority:");
        msg!("  - key: {} (must be mint authority)", authority_info.key);
        msg!("  - is_signer: {}", authority_info.is_signer);
        
        // Pack and save account data
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;
        Account::pack(destination_account, &mut destination_info.data.borrow_mut())?;
        
        msg!("✅ MintTo completed successfully");
        msg!("========================================");
        
        Ok(())
    }

    /// Main instruction processing router
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction: TokenInstruction = TokenInstruction::unpack(instruction_data)?;
        
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
            TokenInstruction::Transfer { amount } => {
                Self::process_transfer(program_id, accounts, amount)
            }
            TokenInstruction::MintTo { amount } => {
                Self::process_mint_to(program_id, accounts, amount)
            }
            TokenInstruction::Burn { .. } => {
                // TODO: Implement Burn handler
                Err(TokenError::NotInitialized.into())
            }
        }
    }
}
