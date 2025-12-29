//! Program entrypoint

use {
    crate::processor::Processor,
    solana_account_info::AccountInfo,
    solana_msg::msg,
    solana_program_error::ProgramResult,
    solana_pubkey::Pubkey,
    crate::error::TokenError,
};

solana_program_entrypoint::entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        if let Ok(token_error) = error.to_str::<TokenError>() {
            msg!(token_error);
        }
        return Err(error);
    }
    Ok(())
}
