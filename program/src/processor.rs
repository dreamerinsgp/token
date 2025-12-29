//! Program state processor

use {
    crate::{
        error::TokenError,
        instruction::TokenInstruction,
        state::{Account, Mint},
    },
    solana_account_info::{next_account_info, AccountInfo},
    solana_msg::msg,
    solana_program_error::{ProgramError, ProgramResult},
    solana_program_memory::sol_memcmp,
    solana_program_pack::{IsInitialized, Pack},
    solana_pubkey::{Pubkey, PUBKEY_BYTES},
    solana_rent::Rent,
    solana_sysvar::Sysvar,
};

/// Program state handler.
pub struct Processor {}

impl Processor {
    /// Processes an [`InitializeMint`](enum.TokenInstruction.html) instruction.
    pub fn process_initialize_mint(
        accounts: &[AccountInfo],
        decimals: u8,
        mint_authority: Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_info = next_account_info(account_info_iter)?;
        let mint_data_len = mint_info.data_len();
        let rent = Rent::from_account_info(next_account_info(account_info_iter)?)?;

        let mut mint = Mint::unpack_unchecked(&mint_info.data.borrow())?;
        if mint.is_initialized {
            return Err(TokenError::AlreadyInitialized.into());
        }

        if !rent.is_exempt(mint_info.lamports(), mint_data_len) {
            return Err(TokenError::NotRentExempt.into());
        }

        mint.mint_authority = mint_authority;
        mint.decimals = decimals;
        mint.supply = 0;
        mint.is_initialized = true;

        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes an [`InitializeAccount`](enum.TokenInstruction.html) instruction.
    pub fn process_initialize_account(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let new_account_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;
        let rent = Rent::from_account_info(next_account_info(account_info_iter)?)?;

        let new_account_info_data_len = new_account_info.data_len();

        let mut account = Account::unpack_unchecked(&new_account_info.data.borrow())?;
        if account.is_initialized() {
            return Err(TokenError::AlreadyInitialized.into());
        }

        if !rent.is_exempt(new_account_info.lamports(), new_account_info_data_len) {
            return Err(TokenError::NotRentExempt.into());
        }

        // Verify mint account is owned by this program and is initialized
        Self::check_account_owner(program_id, mint_info)?;
        let _ = Mint::unpack(&mint_info.data.borrow())
            .map_err(|_| Into::<ProgramError>::into(TokenError::InvalidMint))?;

        account.mint = *mint_info.key;
        account.owner = *owner_info.key;
        account.amount = 0;
        account.is_initialized = true;

        Account::pack(account, &mut new_account_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes a [`Transfer`](enum.TokenInstruction.html) instruction.
    pub fn process_transfer(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let source_account_info = next_account_info(account_info_iter)?;
        let destination_account_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;

        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        let mut destination_account = Account::unpack(&destination_account_info.data.borrow())?;

        if !source_account.is_initialized() || !destination_account.is_initialized() {
            return Err(TokenError::NotInitialized.into());
        }

        if source_account.amount < amount {
            return Err(TokenError::InsufficientFunds.into());
        }

        if !Self::cmp_pubkeys(&source_account.mint, &destination_account.mint) {
            return Err(TokenError::MintMismatch.into());
        }

        // Validate owner signature
        if !Self::cmp_pubkeys(&source_account.owner, authority_info.key) {
            return Err(TokenError::InvalidOwner.into());
        }
        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Handle self-transfer (no-op)
        if Self::cmp_pubkeys(source_account_info.key, destination_account_info.key) {
            return Ok(());
        }

        // Update balances with checked arithmetic
        source_account.amount = source_account
            .amount
            .checked_sub(amount)
            .ok_or(TokenError::Overflow)?;
        destination_account.amount = destination_account
            .amount
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;

        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;
        Account::pack(
            destination_account,
            &mut destination_account_info.data.borrow_mut(),
        )?;

        Ok(())
    }

    /// Processes a [`MintTo`](enum.TokenInstruction.html) instruction.
    pub fn process_mint_to(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_info = next_account_info(account_info_iter)?;
        let destination_account_info = next_account_info(account_info_iter)?;
        let mint_authority_info = next_account_info(account_info_iter)?;

        let mut destination_account = Account::unpack(&destination_account_info.data.borrow())?;
        if !destination_account.is_initialized() {
            return Err(TokenError::NotInitialized.into());
        }

        if !Self::cmp_pubkeys(mint_info.key, &destination_account.mint) {
            return Err(TokenError::MintMismatch.into());
        }

        let mut mint = Mint::unpack(&mint_info.data.borrow())?;
        if !mint.is_initialized() {
            return Err(TokenError::NotInitialized.into());
        }

        // Validate mint authority signature
        if !Self::cmp_pubkeys(&mint.mint_authority, mint_authority_info.key) {
            return Err(TokenError::InvalidOwner.into());
        }
        if !mint_authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Update balances with checked arithmetic
        destination_account.amount = destination_account
            .amount
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;

        mint.supply = mint
            .supply
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;

        Account::pack(
            destination_account,
            &mut destination_account_info.data.borrow_mut(),
        )?;
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes a [`Burn`](enum.TokenInstruction.html) instruction.
    pub fn process_burn(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let source_account_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;

        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        let mut mint = Mint::unpack(&mint_info.data.borrow())?;

        if !source_account.is_initialized() {
            return Err(TokenError::NotInitialized.into());
        }

        if source_account.amount < amount {
            return Err(TokenError::InsufficientFunds.into());
        }

        if !Self::cmp_pubkeys(mint_info.key, &source_account.mint) {
            return Err(TokenError::MintMismatch.into());
        }

        // Validate owner signature
        if !Self::cmp_pubkeys(&source_account.owner, authority_info.key) {
            return Err(TokenError::InvalidOwner.into());
        }
        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Update balances with checked arithmetic
        source_account.amount = source_account
            .amount
            .checked_sub(amount)
            .ok_or(TokenError::Overflow)?;
        mint.supply = mint
            .supply
            .checked_sub(amount)
            .ok_or(TokenError::Overflow)?;

        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes an [`Instruction`](enum.Instruction.html).
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = TokenInstruction::unpack(input)?;

        match instruction {
            TokenInstruction::InitializeMint {
                decimals,
                mint_authority,
            } => {
                msg!("Instruction: InitializeMint");
                Self::process_initialize_mint(accounts, decimals, mint_authority)
            }
            TokenInstruction::InitializeAccount => {
                msg!("Instruction: InitializeAccount");
                Self::process_initialize_account(program_id, accounts)
            }
            TokenInstruction::Transfer { amount } => {
                msg!("Instruction: Transfer");
                Self::process_transfer(program_id, accounts, amount)
            }
            TokenInstruction::MintTo { amount } => {
                msg!("Instruction: MintTo");
                Self::process_mint_to(program_id, accounts, amount)
            }
            TokenInstruction::Burn { amount } => {
                msg!("Instruction: Burn");
                Self::process_burn(program_id, accounts, amount)
            }
        }
    }

    /// Checks that the account is owned by the expected program
    pub fn check_account_owner(program_id: &Pubkey, account_info: &AccountInfo) -> ProgramResult {
        if !Self::cmp_pubkeys(program_id, account_info.owner) {
            Err(ProgramError::IncorrectProgramId)
        } else {
            Ok(())
        }
    }

    /// Checks two pubkeys for equality in a computationally cheap way using
    /// `sol_memcmp`
    pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
        unsafe { sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0 }
    }
}
