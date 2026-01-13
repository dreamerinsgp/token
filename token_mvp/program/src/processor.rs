use {
    crate::{
        check_program_account,
        error::TokenError,
        instruction::TokenInstruction,
        native_mint,
        state::{Account, Mint},
    },
    solana_account_info::{next_account_info, AccountInfo},
    solana_msg::msg,
    solana_program_error::{ProgramError, ProgramResult},
    solana_program_option::COption,
    solana_program_pack::Pack,
    solana_pubkey::Pubkey,
    solana_rent::Rent,
    solana_sysvar::SysvarSerialize,
};
use std::str::FromStr;

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
        
        // Check if this is a native mint account
        let is_native_mint = mint_info.key == &native_mint::id();
        
        // Set account fields
        account.mint = *mint_info.key;
        account.owner = *owner_info.key;
        account.is_initialized = true;
        account.delegate = COption::None;
        account.delegated_amount = 0;
        account.is_frozen = false;
        
        // For native accounts, set is_native and calculate initial balance
        if is_native_mint {
            let rent_exempt_reserve = rent.minimum_balance(account_data_len);
            account.is_native = COption::Some(rent_exempt_reserve);
            account.amount = account_info
                .lamports()
                .checked_sub(rent_exempt_reserve)
                .ok_or(TokenError::Overflow)?;
        } else {
            account.is_native = COption::None;
            account.amount = 0; // Initialize with zero balance
        }
        
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
        
        // Check if source account is frozen
        if source_account.is_frozen {
            return Err(TokenError::AccountFrozen.into());
        }
        
        // Unpack destination account
        let mut destination_account = Account::unpack(&destination_info.data.borrow())?;
        
        // Validate destination account is initialized
        if !destination_account.is_initialized {
            return Err(TokenError::NotInitialized.into());
        }
        
        // Check if destination account is frozen
        if destination_account.is_frozen {
            return Err(TokenError::AccountFrozen.into());
        }
        
        // Validate mint match
        if source_account.mint != destination_account.mint {
            return Err(TokenError::MintMismatch.into());
        }
        
        // Check for self-transfer (no-op)
        let self_transfer = source_info.key == destination_info.key;
        
        // Validate authority: check if it's the owner or a delegate
        match source_account.delegate {
            COption::Some(ref delegate) if delegate == authority_info.key => {
                // Authority is a delegate
                // Validate delegate is a signer
                if !authority_info.is_signer {
                    return Err(TokenError::InvalidOwner.into());
                }
                
                // Validate sufficient delegated amount
                if source_account.delegated_amount < amount {
                    return Err(TokenError::InsufficientFunds.into());
                }
                
                // Decrease delegated amount if not self-transfer
                if !self_transfer {
                    source_account.delegated_amount = source_account
                        .delegated_amount
                        .checked_sub(amount)
                        .ok_or(TokenError::Overflow)?;
                    
                    // If delegated amount becomes 0, clear delegate
                    if source_account.delegated_amount == 0 {
                        source_account.delegate = COption::None;
                    }
                }
            }
            _ => {
                // Authority must be the owner
                if source_account.owner != *authority_info.key {
                    return Err(TokenError::InvalidOwner.into());
                }
                
                // Validate owner is a signer
                if !authority_info.is_signer {
                    return Err(TokenError::InvalidOwner.into());
                }
            }
        }
        
        // Check for self-transfer (no-op)
        if self_transfer {
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
        match source_account.delegate {
            COption::Some(ref delegate) if delegate == authority_info.key => {
                msg!("  - key: {} (delegate)", authority_info.key);
                msg!("  - delegated_amount before: {} (will subtract {})", 
                     source_account.delegated_amount + amount, amount);
                msg!("  - delegated_amount after: {}", source_account.delegated_amount);
            }
            _ => {
                msg!("  - key: {} (must be source owner)", authority_info.key);
            }
        }
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

    /// Processes a Burn instruction
    pub fn process_burn(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let _program_id = program_id; // Suppress unused warning for now
        let account_info_iter = &mut accounts.iter();
        
        // Extract accounts in order:
        // 0. [writable] The account to burn from
        // 1. [writable] The token mint
        // 2. [signer] The account's owner/delegate
        let account_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        
        // Validate accounts are owned by this program
        check_program_account(account_info)?;
        check_program_account(mint_info)?;
        
        // Validate accounts are writable
        if !account_info.is_writable {
            return Err(TokenError::InvalidOwner.into());
        }
        if !mint_info.is_writable {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Unpack account
        let mut account = Account::unpack(&account_info.data.borrow())?;
        
        // Validate account is initialized
        if !account.is_initialized {
            return Err(TokenError::NotInitialized.into());
        }
        
        // Check if account is frozen
        if account.is_frozen {
            return Err(TokenError::AccountFrozen.into());
        }
        
        // Unpack mint account
        let mut mint = Mint::unpack(&mint_info.data.borrow())?;
        
        // Validate mint is initialized
        if !mint.is_initialized {
            return Err(TokenError::InvalidMint.into());
        }
        
        // Validate mint match
        if account.mint != *mint_info.key {
            return Err(TokenError::MintMismatch.into());
        }
        
        // Validate authority is the owner (for now, we don't support delegate)
        if account.owner != *authority_info.key {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Validate authority is a signer
        if !authority_info.is_signer {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Validate sufficient balance
        if account.amount < amount {
            return Err(TokenError::InsufficientFunds.into());
        }
        
        // Perform burn: decrease supply and balance
        account.amount = account.amount
            .checked_sub(amount)
            .ok_or(TokenError::Overflow)?;
        
        mint.supply = mint.supply
            .checked_sub(amount)
            .ok_or(TokenError::Overflow)?;
        
        // Log debug information
        msg!("=== Burn Debug Info ===");
        msg!("[account_info] Account to Burn From:");
        msg!("  - key: {}", account_info.key);
        msg!("  - owner: {}", account.owner);
        msg!("  - mint: {}", account.mint);
        msg!("  - amount before: {} (will subtract {})", account.amount + amount, amount);
        msg!("  - amount after: {}", account.amount);
        
        msg!("[mint_info] Mint Account:");
        msg!("  - key: {}", mint_info.key);
        msg!("  - supply before: {} (will subtract {})", mint.supply + amount, amount);
        msg!("  - supply after: {}", mint.supply);
        
        msg!("[authority_info] Account Owner:");
        msg!("  - key: {} (must be account owner)", authority_info.key);
        msg!("  - is_signer: {}", authority_info.is_signer);
        
        // Pack and save account data
        Account::pack(account, &mut account_info.data.borrow_mut())?;
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;
        
        msg!("✅ Burn completed successfully");
        msg!("========================================");
        
        Ok(())
    }

    /// Processes a SyncNative instruction
    /// 
    /// SyncNative synchronizes the token account balance with the underlying SOL balance
    /// for native (wrapped SOL) accounts. For native accounts, the token balance equals
    /// the account's lamports minus the rent-exempt reserve stored in is_native field.
    /// 
    /// Accounts:
    /// - [writable] The native account to sync
    pub fn process_sync_native(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        
        // Extract account:
        // 0. [writable] The native account to sync
        let native_account_info = next_account_info(account_info_iter)?;
        
        // Validate account is owned by this program
        check_program_account(native_account_info)?;
        
        // Unpack account
        let mut native_account = Account::unpack(&native_account_info.data.borrow())?;
        
        // Check if this is a native account by checking is_native field
        // Only native accounts have is_native set to Some(rent_exempt_reserve)
        if let COption::Some(rent_exempt_reserve) = native_account.is_native {
            // Calculate new amount: lamports - rent_exempt_reserve
            let new_amount = native_account_info
                .lamports()
                .checked_sub(rent_exempt_reserve)
                .ok_or(TokenError::Overflow)?;
            
            // Validate: new amount should not be less than current amount
            // This prevents the balance from decreasing unexpectedly (e.g., if SOL was withdrawn)
            // The balance can only increase when SOL is sent directly to the account
            if new_amount < native_account.amount {
                return Err(TokenError::InvalidState.into());
            }
            
            //Native accoutn with 1 SOL ;
            //Rent is 0.0001 SOl
            //token Balance : 0.999SOl 
            // Alice->1.5 SOL
            //token Balance: 0.999SOL
            // 1.5 SOL - 0.0001SOL = 1.499 OSL. 

            // Update account balance
            native_account.amount = new_amount;
            
            // Log debug information
            msg!("=== SyncNative Debug Info ===");
            msg!("[native_account_info] Account Information:");
            msg!("  - key: {}", native_account_info.key);
            msg!("  - lamports: {}", native_account_info.lamports());
            msg!("  - rent_exempt_reserve: {} (from is_native field)", rent_exempt_reserve);
            msg!("  - amount before: {}", native_account.amount);
            msg!("  - amount after: {} (lamports - rent_exempt_reserve)", new_amount);
            
            // Pack and save account data
            Account::pack(native_account, &mut native_account_info.data.borrow_mut())?;
            
            msg!("✅ SyncNative completed successfully");
            msg!("========================================");
        } else {
            // Not a native account, return error
            // Only accounts initialized with native mint have is_native set
            return Err(TokenError::NonNativeNotSupported.into());
        }
        
        Ok(())
    }

    /// Processes an Approve instruction
    /// 
    /// Approve grants a delegate authority to transfer tokens on behalf of the account owner.
    /// The delegate can transfer up to the approved amount.
    /// 
    /// Accounts:
    /// - [writable] The source account
    /// - [] The delegate
    /// - [signer] The source account owner
    pub fn process_approve(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        
        // Extract accounts in order:
        // 0. [writable] The source account
        // 1. [] The delegate
        // 2. [signer] The source account owner
        let source_account_info = next_account_info(account_info_iter)?;
        let delegate_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;
        
        // Validate account is owned by this program
        check_program_account(source_account_info)?;
        
        // Validate account is writable
        if !source_account_info.is_writable {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Unpack source account
        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        
        // Validate account is initialized
        if !source_account.is_initialized {
            return Err(TokenError::NotInitialized.into());
        }
        
        // Check if account is frozen
        if source_account.is_frozen {
            return Err(TokenError::AccountFrozen.into());
        }
        
        // Validate owner is the account owner
        if source_account.owner != *owner_info.key {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Validate owner is a signer
        if !owner_info.is_signer {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Set delegate and delegated amount
        source_account.delegate = COption::Some(*delegate_info.key);
        source_account.delegated_amount = amount;
        
        // Log debug information
        msg!("=== Approve Debug Info ===");
        msg!("[source_account_info] Account Information:");
        msg!("  - key: {}", source_account_info.key);
        msg!("  - owner: {}", source_account.owner);
        msg!("  - mint: {}", source_account.mint);
        msg!("  - amount: {}", source_account.amount);
        
        msg!("[delegate_info] Delegate:");
        msg!("  - key: {} (will be approved)", delegate_info.key);
        
        msg!("[owner_info] Account Owner:");
        msg!("  - key: {} (must be account owner)", owner_info.key);
        msg!("  - is_signer: {}", owner_info.is_signer);
        
        msg!("[approval] Approval Details:");
        msg!("  - delegate: {}", delegate_info.key);
        msg!("  - delegated_amount: {} (approved amount)", amount);
        
        // Pack and save account data
        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;
        
        msg!("✅ Approve completed successfully");
        msg!("========================================");
        
        Ok(())
    }

    /// Processes a CloseAccount instruction
    /// 
    /// CloseAccount closes a token account by transferring all remaining SOL (lamports)
    /// to a destination account. This allows users to reclaim rent-exempt SOL that was
    /// locked in the account.
    /// 
    /// For non-native accounts, the account must have zero token balance.
    /// For native accounts, the remaining SOL (after rent-exempt reserve) can be transferred.
    /// 
    /// Accounts:
    /// - [writable] The account to close
    /// - [writable] The destination account to receive remaining SOL
    /// - [signer] The account's owner (or close authority if set)
    pub fn process_close_account(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        
        // Extract accounts in order:
        // 0. [writable] The account to close
        // 1. [writable] The destination account to receive remaining SOL
        // 2. [signer] The account's owner
        let source_account_info = next_account_info(account_info_iter)?;
        let destination_account_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        
        // Validate source and destination are different accounts
        if source_account_info.key == destination_account_info.key {
            return Err(solana_program_error::ProgramError::InvalidAccountData);
        }
        
        // Validate account is owned by this program
        check_program_account(source_account_info)?;
        
        // Unpack source account
        let source_account = Account::unpack(&source_account_info.data.borrow())?;
        
        // Validate account is initialized
        if !source_account.is_initialized {
            return Err(TokenError::NotInitialized.into());
        }
        
        // For non-native accounts, balance must be zero
        // For native accounts, we can close even with balance (the SOL will be transferred)
        let is_native = source_account.is_native.is_some();
        if !is_native && source_account.amount != 0 {
            return Err(TokenError::InvalidState.into());
        }
        
        // Validate authority is the account owner
        // Note: In full token program, there's a close_authority field,
        // but in our MVP we only support owner closing
        if source_account.owner != *authority_info.key {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Validate authority is a signer
        if !authority_info.is_signer {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Transfer all lamports from source to destination
        let source_lamports = source_account_info.lamports();
        let destination_starting_lamports = destination_account_info.lamports();
        
        // Add source lamports to destination
        **destination_account_info.lamports.borrow_mut() = destination_starting_lamports
            .checked_add(source_lamports)
            .ok_or(TokenError::Overflow)?;
        
        // Set source lamports to zero
        **source_account_info.lamports.borrow_mut() = 0;
        
        // Delete the account by assigning it to system program and clearing data
        // System program ID: 11111111111111111111111111111111
        let system_program_id = Pubkey::from_str("11111111111111111111111111111111")
            .map_err(|_| ProgramError::InvalidAccountData)?;
        source_account_info.assign(&system_program_id);
        
        // Clear account data
        let mut account_data = source_account_info.data.borrow_mut();
        account_data.fill(0);
        drop(account_data);
        
        // Log debug information
        msg!("=== CloseAccount Debug Info ===");
        msg!("[source_account_info] Account to Close:");
        msg!("  - key: {}", source_account_info.key);
        msg!("  - owner: {}", source_account.owner);
        msg!("  - mint: {}", source_account.mint);
        msg!("  - amount: {} (token balance)", source_account.amount);
        msg!("  - lamports: {} (will be transferred)", source_lamports);
        msg!("  - is_native: {}", is_native);
        
        msg!("[destination_account_info] Destination:");
        msg!("  - key: {} (will receive lamports)", destination_account_info.key);
        msg!("  - lamports before: {}", destination_starting_lamports);
        msg!("  - lamports after: {} (received {})", 
             destination_starting_lamports + source_lamports, source_lamports);
        
        msg!("[authority_info] Account Owner:");
        msg!("  - key: {} (must be account owner)", authority_info.key);
        msg!("  - is_signer: {}", authority_info.is_signer);
        
        msg!("✅ CloseAccount completed successfully");
        msg!("========================================");
        
        Ok(())
    }

    /// Processes a FreezeAccount instruction
    /// 
    /// FreezeAccount freezes an account, preventing transfers, approvals, and burns.
    /// Only the mint's freeze authority can freeze accounts.
    /// 
    /// Accounts:
    /// - [writable] The account to freeze
    /// - [] The token mint
    /// - [signer] The mint's freeze authority
    pub fn process_freeze_account(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        
        // Extract accounts in order:
        // 0. [writable] The account to freeze
        // 1. [] The token mint
        // 2. [signer] The mint's freeze authority
        let account_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        
        // Validate account is owned by this program
        check_program_account(account_info)?;
        
        // Validate account is writable
        if !account_info.is_writable {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Unpack account
        let mut account = Account::unpack(&account_info.data.borrow())?;
        
        // Validate account is initialized
        if !account.is_initialized {
            return Err(TokenError::NotInitialized.into());
        }
        
        // Check if account is already frozen
        if account.is_frozen {
            return Err(TokenError::InvalidState.into());
        }
        
        // Cannot freeze native accounts
        if account.is_native.is_some() {
            return Err(TokenError::NonNativeNotSupported.into());
        }
        
        // Validate mint matches
        if account.mint != *mint_info.key {
            return Err(TokenError::MintMismatch.into());
        }
        
        // Unpack mint to check freeze authority
        let mint = Mint::unpack(&mint_info.data.borrow())?;
        
        // Validate mint has freeze authority
        match mint.freeze_authority {
            COption::Some(freeze_authority) => {
                // Validate authority is the freeze authority
                if freeze_authority != *authority_info.key {
                    return Err(TokenError::InvalidOwner.into());
                }
                
                // Validate authority is a signer
                if !authority_info.is_signer {
                    return Err(TokenError::InvalidOwner.into());
                }
            }
            COption::None => {
                // Mint doesn't have freeze authority, cannot freeze accounts
                return Err(TokenError::MintCannotFreeze.into());
            }
        }
        
        // Freeze the account
        account.is_frozen = true;
        
        // Log debug information
        msg!("=== FreezeAccount Debug Info ===");
        msg!("[account_info] Account to Freeze:");
        msg!("  - key: {}", account_info.key);
        msg!("  - owner: {}", account.owner);
        msg!("  - mint: {}", account.mint);
        msg!("  - amount: {}", account.amount);
        msg!("  - is_frozen before: false");
        msg!("  - is_frozen after: true");
        
        msg!("[mint_info] Token Mint:");
        msg!("  - key: {}", mint_info.key);
        msg!("  - freeze_authority: {:?}", mint.freeze_authority);
        
        msg!("[authority_info] Freeze Authority:");
        msg!("  - key: {} (must be freeze authority)", authority_info.key);
        msg!("  - is_signer: {}", authority_info.is_signer);
        
        // Pack and save account data
        Account::pack(account, &mut account_info.data.borrow_mut())?;
        
        msg!("✅ FreezeAccount completed successfully");
        msg!("========================================");
        
        Ok(())
    }

    /// Processes a ThawAccount instruction
    /// 
    /// ThawAccount thaws a frozen account, restoring the ability to transfer, approve, and burn.
    /// Only the mint's freeze authority can thaw accounts.
    /// 
    /// Accounts:
    /// - [writable] The account to thaw
    /// - [] The token mint
    /// - [signer] The mint's freeze authority
    pub fn process_thaw_account(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        
        // Extract accounts in order:
        // 0. [writable] The account to thaw
        // 1. [] The token mint
        // 2. [signer] The mint's freeze authority
        let account_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        
        // Validate account is owned by this program
        check_program_account(account_info)?;
        
        // Validate account is writable
        if !account_info.is_writable {
            return Err(TokenError::InvalidOwner.into());
        }
        
        // Unpack account
        let mut account = Account::unpack(&account_info.data.borrow())?;
        
        // Validate account is initialized
        if !account.is_initialized {
            return Err(TokenError::NotInitialized.into());
        }
        
        // Check if account is already thawed (not frozen)
        if !account.is_frozen {
            return Err(TokenError::InvalidState.into());
        }
        
        // Cannot thaw native accounts (they can't be frozen in the first place)
        if account.is_native.is_some() {
            return Err(TokenError::NonNativeNotSupported.into());
        }
        
        // Validate mint matches
        if account.mint != *mint_info.key {
            return Err(TokenError::MintMismatch.into());
        }
        
        // Unpack mint to check freeze authority
        let mint = Mint::unpack(&mint_info.data.borrow())?;
        
        // Validate mint has freeze authority
        match mint.freeze_authority {
            COption::Some(freeze_authority) => {
                // Validate authority is the freeze authority
                if freeze_authority != *authority_info.key {
                    return Err(TokenError::InvalidOwner.into());
                }
                
                // Validate authority is a signer
                if !authority_info.is_signer {
                    return Err(TokenError::InvalidOwner.into());
                }
            }
            COption::None => {
                // Mint doesn't have freeze authority, cannot thaw accounts
                return Err(TokenError::MintCannotFreeze.into());
            }
        }
        
        // Thaw the account
        account.is_frozen = false;
        
        // Log debug information
        msg!("=== ThawAccount Debug Info ===");
        msg!("[account_info] Account to Thaw:");
        msg!("  - key: {}", account_info.key);
        msg!("  - owner: {}", account.owner);
        msg!("  - mint: {}", account.mint);
        msg!("  - amount: {}", account.amount);
        msg!("  - is_frozen before: true");
        msg!("  - is_frozen after: false");
        
        msg!("[mint_info] Token Mint:");
        msg!("  - key: {}", mint_info.key);
        msg!("  - freeze_authority: {:?}", mint.freeze_authority);
        
        msg!("[authority_info] Freeze Authority:");
        msg!("  - key: {} (must be freeze authority)", authority_info.key);
        msg!("  - is_signer: {}", authority_info.is_signer);
        
        // Pack and save account data
        Account::pack(account, &mut account_info.data.borrow_mut())?;
        
        msg!("✅ ThawAccount completed successfully");
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
            TokenInstruction::Burn { amount } => {
                Self::process_burn(program_id, accounts, amount)
            }
            TokenInstruction::SyncNative => {
                Self::process_sync_native(program_id, accounts)
            }
            TokenInstruction::Approve { amount } => {
                Self::process_approve(program_id, accounts, amount)
            }
            TokenInstruction::CloseAccount => {
                Self::process_close_account(program_id, accounts)
            }
            TokenInstruction::FreezeAccount => {
                Self::process_freeze_account(program_id, accounts)
            }
            TokenInstruction::ThawAccount => {
                Self::process_thaw_account(program_id, accounts)
            }
        }
    }
}
