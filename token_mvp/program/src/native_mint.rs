//! Native mint utilities
//! 
//! The native mint is a special mint that represents wrapped SOL.
//! Its address is: So11111111111111111111111111111111111111112

use solana_pubkey::Pubkey;

/// Native mint program ID
/// This is the address of the wrapped SOL mint: So11111111111111111111111111111111111111112
pub fn id() -> Pubkey {
    Pubkey::new_from_array([
        6, 167, 213, 23, 24, 199, 116, 201, 55, 218, 154, 117, 160, 96, 109, 189, 179, 139, 142, 80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ])
}
