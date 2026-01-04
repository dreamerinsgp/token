# Video Script: Building a Solana Token Program Structure

## Video Overview
**Duration**: ~15-20 minutes  
**Format**: Screen recording with voiceover  
**Audience**: Developers new to Solana program development

---

## Opening (0:00 - 1:00)

**[Screen: Show empty terminal/IDE]**

Hey! So you want to build a Solana token program, but you're staring at an empty directory wondering where to start. I've been there. Today, I'm going to walk you through setting up the project structure—the foundation that everything else builds on.

Think of it like this: you wouldn't start building a house without a foundation, right? Same thing here. Get the structure wrong, and you'll spend hours debugging dependency issues and hunting for files. Get it right, and everything else becomes easier.

Let's build this together, step by step.

---

## Part 1: Understanding What We're Building (1:00 - 2:30)

**[Screen: Show diagram of project structure]**

Before we write any code, let me show you what we're creating:

```
token/
├── Cargo.toml          # Workspace configuration
├── rust-toolchain.toml # Rust version pinning
├── rustfmt.toml        # Code formatting rules
├── README.md
└── program/
    ├── Cargo.toml      # Program package config
    ├── src/
    │   ├── lib.rs      # Main library file
    │   ├── entrypoint.rs  # Program entry point
    │   ├── error.rs    # Error definitions
    │   ├── instruction.rs # Instruction types
    │   ├── state.rs    # Data structures
    │   └── processor.rs # Business logic
    └── tests/          # Test files
```

Each file has a specific job. We'll create them one by one, and I'll explain why each one matters.

---

## Part 2: Setting Up the Workspace (2:30 - 6:00)

**[Screen: Terminal, create directory]**

Let's start. First, create your project directory:

```bash
mkdir -p token
cd token
```

**[Screen: Create Cargo.toml, show content]**

Now, create the workspace `Cargo.toml`. This file tells Rust that this directory contains multiple related packages.

```bash
touch Cargo.toml
```

Open it and add this:

```toml
[workspace]
resolver = "2"
members = ["program"]

[workspace.package]
authors = ["Your Name <your.email@example.com>"]
repository = "https://github.com/yourusername/token"
license = "Apache-2.0"
edition = "2021"

[workspace.dependencies]
solana-program-error = "3.0.0"
solana-program-pack = "3.0.0"
solana-pubkey = "3.0.0"
solana-program-entrypoint = "3.0.0"
solana-instruction = "3.0.0"
thiserror = "2.0"
arrayref = "0.3.9"
```

**[Pause, explain]**

Here's why this matters: The `[workspace.dependencies]` section lets us define shared dependencies once. Instead of repeating version numbers in every package, we define them here and reference them. This prevents version conflicts and makes updates easier.

**[Screen: Create rust-toolchain.toml]**

Next, let's pin the Rust version. This ensures everyone uses the same compiler:

```bash
touch rust-toolchain.toml
```

```toml
[toolchain]
channel = "stable"
```

**[Screen: Create rustfmt.toml]**

And finally, let's set up code formatting rules:

```bash
touch rustfmt.toml
```

```toml
edition = "2021"
max_width = 100
```

**[Screen: Run cargo check]**

Let's verify this works:

```bash
cargo check
```

You'll see an error about missing members—that's expected. We'll create the program package next.

---

## Part 3: Creating the Program Package (6:00 - 9:00)

**[Screen: Create program directory]**

Now let's create the actual Solana program:

```bash
mkdir -p program/src
mkdir -p program/tests
```

**[Screen: Create program/Cargo.toml, show content]**

Create the program's `Cargo.toml`:

```bash
touch program/Cargo.toml
```

Add this configuration:

```toml
[package]
name = "spl-token"
version = "0.1.0"
description = "Solana Program Library Token"
authors = { workspace = true }
repository = { workspace = true }
license = { workspace = true }
edition = { workspace = true }

[features]
no-entrypoint = []

[dependencies]
arrayref = "0.3.9"
solana-program-error = { workspace = true }
solana-program-pack = { workspace = true }
solana-pubkey = { workspace = true }
solana-program-entrypoint = { workspace = true }
solana-instruction = { workspace = true }
solana-account-info = "3.0.0"
solana-program-memory = "3.0.0"
solana-rent = "3.0.0"
solana-sysvar = { version = "3.0.0", features = ["bincode"] }
solana-msg = "3.0.0"
solana-cpi = "3.0.0"
thiserror = { workspace = true }

[dev-dependencies]
mollusk-svm = "0.6.3"
solana-account = "3.0.0"

[lib]
crate-type = ["cdylib", "lib"]

[package.metadata.solana]
program-id = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
```

**[Pause, highlight key parts]**

Notice a few important things:
- `crate-type = ["cdylib", "lib"]` tells Rust to build a shared library, which Solana needs
- `{ workspace = true }` pulls values from the workspace config we created earlier
- `program-id` is your program's on-chain identifier

**[Screen: Run cargo check in program directory]**

Let's check if this works:

```bash
cd program
cargo check
```

It'll fail because we haven't created source files yet, but it should recognize the package structure.

---

## Part 4: Creating Core Modules (9:00 - 18:00)

**[Screen: Show src/ directory]**

Now for the fun part—the actual code. We'll create six files that form the backbone of your program.

### File 1: lib.rs

**[Screen: Create and edit lib.rs]**

```bash
touch program/src/lib.rs
```

This is your main library file. It declares all modules and provides utility functions:

```rust
#![allow(clippy::arithmetic_side_effects)]
#![deny(missing_docs)]

//! A minimal ERC20-like Token program for the Solana blockchain

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

use solana_program_error::ProgramError;

/// Program ID - This should be set to your actual program ID
pub fn id() -> solana_pubkey::Pubkey {
    solana_pubkey::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
}

/// Check that the account is owned by the token program
pub fn check_id(account: &solana_pubkey::Pubkey) -> bool {
    account == &id()
}

/// Check that the account is owned by the token program
pub fn check_program_account(
    account: &solana_account_info::AccountInfo
) -> Result<(), ProgramError> {
    if account.owner != &id() {
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}
```

**[Explain]**

This file declares all your modules and gives you helper functions to verify program ownership. The `#[cfg(not(feature = "no-entrypoint"))]` line conditionally includes the entrypoint only when building for Solana.

### File 2: entrypoint.rs

**[Screen: Create and edit entrypoint.rs]**

```bash
touch program/src/entrypoint.rs
```

This is where Solana calls your program:

```rust
//! Program entrypoint

use {
    crate::processor::Processor,
    solana_account_info::AccountInfo,
    solana_program_error::ProgramResult,
    solana_pubkey::Pubkey,
};

solana_program_entrypoint::entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Processor::process(program_id, accounts, instruction_data)
}
```

**[Explain]**

Every transaction hits this function first. The `entrypoint!` macro sets up the boilerplate, and we route everything to `Processor::process`.

### File 3: error.rs

**[Screen: Create and edit error.rs]**

```bash
touch program/src/error.rs
```

Define all the ways your program can fail:

```rust
//! Error types for the token program

use {
    solana_program_error::{ProgramError, ProgramResult},
    thiserror::Error,
};

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenError {
    #[error("Account already initialized")]
    AlreadyInitialized,
    
    #[error("Account not initialized")]
    NotInitialized,
    
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    #[error("Invalid mint")]
    InvalidMint,
    
    #[error("Mint mismatch")]
    MintMismatch,
    
    #[error("Invalid owner")]
    InvalidOwner,
    
    #[error("Overflow")]
    Overflow,
    
    #[error("Not rent exempt")]
    NotRentExempt,
}

impl From<TokenError> for ProgramError {
    fn from(e: TokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
```

**[Explain]**

Using `thiserror` gives us clear error messages and automatic conversion to Solana's error type. You'll add more errors as you discover edge cases.

### File 4: instruction.rs

**[Screen: Create and edit instruction.rs]**

```bash
touch program/src/instruction.rs
```

Define what actions your program supports:

```rust
//! Instruction definitions for the token program

use {
    solana_instruction::Instruction,
    solana_pubkey::Pubkey,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenInstruction {
    InitializeMint {
        decimals: u8,
        mint_authority: Pubkey,
    },
    InitializeAccount,
    Transfer { amount: u64 },
    MintTo { amount: u64 },
    Burn { amount: u64 },
}

impl TokenInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, solana_program_error::ProgramError> {
        // TODO: Implement instruction unpacking
        Err(solana_program_error::ProgramError::InvalidInstructionData)
    }
}
```

**[Explain]**

Each variant represents a different operation. Right now, `unpack` is a placeholder—we'll implement it later when we handle instruction parsing.

### File 5: state.rs

**[Screen: Create and edit state.rs]**

```bash
touch program/src/state.rs
```

Define your on-chain data structures:

```rust
//! State structures for the token program

use {
    solana_program_pack::{IsInitialized, Pack},
    solana_pubkey::Pubkey,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Mint {
    pub mint_authority: Pubkey,
    pub supply: u64,
    pub decimals: u8,
    pub is_initialized: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
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

impl Pack for Mint {
    const LEN: usize = 82;
    
    fn pack_into_slice(&self, _dst: &mut [u8]) {
        // TODO: Implement packing
    }
    
    fn unpack_from_slice(_src: &[u8]) -> Result<Self, solana_program_error::ProgramError> {
        // TODO: Implement unpacking
        Err(solana_program_error::ProgramError::InvalidAccountData)
    }
}

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
```

**[Explain]**

`Mint` represents a token type (like USDC or SOL), and `Account` represents a user's balance. The `Pack` trait lets us serialize and deserialize these structures to/from bytes. We'll implement the actual packing logic later.

### File 6: processor.rs

**[Screen: Create and edit processor.rs]**

```bash
touch program/src/processor.rs
```

This routes instructions to handlers:

```rust
//! Program state processor

use {
    crate::{error::TokenError, instruction::TokenInstruction},
    solana_account_info::AccountInfo,
    solana_program_error::ProgramResult,
    solana_pubkey::Pubkey,
};

/// Program state handler.
pub struct Processor {}

impl Processor {
    /// Main instruction processing router
    pub fn process(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = TokenInstruction::unpack(instruction_data)?;
        
        match instruction {
            TokenInstruction::InitializeMint { .. } => {
                // TODO: Implement InitializeMint handler
                Err(TokenError::NotInitialized.into())
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
```

**[Explain]**

Right now, this is just a skeleton. Each match arm returns an error, but later we'll implement the actual logic for each instruction type.

**[Screen: Run cargo check]**

Let's verify everything compiles:

```bash
cd program
cargo check
```

You should see warnings but no errors. The structure is complete!

---

## Part 5: Adding Tests and Documentation (18:00 - 20:00)

**[Screen: Create test files]**

Let's set up the test structure:

```bash
touch program/tests/setup.rs
touch program/tests/processor.rs
```

Add placeholder content:

**setup.rs:**
```rust
//! Test setup utilities

// TODO: Add test setup helpers
```

**processor.rs:**
```rust
//! Processor tests

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // TODO: Add tests
        assert!(true);
    }
}
```

**[Screen: Create README files]**

Add documentation:

**program/README.md:**
```markdown
# Solana Token Program

A minimal ERC20-like Token program for the Solana blockchain.

## Building

```bash
cargo build-sbf
```

## Testing

```bash
cargo test
```
```

**README.md (root):**
```markdown
# Token Program

Solana Token Program workspace.

## Structure

- `program/` - Main Solana program crate
```

**[Screen: Run cargo test]**

Verify tests work:

```bash
cd program
cargo test
```

Should pass, even though the test does nothing.

---

## Part 6: Final Verification (20:00 - 22:00)

**[Screen: Show directory tree]**

Let's verify the complete structure:

```bash
cd ..
tree -L 3 token
```

You should see all files in place.

**[Screen: Run final checks]**

Run these commands to verify everything:

```bash
cd token
cargo check
```

```bash
cd token/program
cargo check
cargo test
```

Everything should compile and tests should pass.

---

## Closing (22:00 - 23:00)

**[Screen: Show final structure]**

And that's it! You now have a complete Solana program structure. Here's what we built:

- Workspace configuration for managing dependencies
- Program package with all core modules
- Error handling, instructions, state, and processor skeletons
- Test framework ready for your tests
- Documentation to help you remember what you built

**[Screen: Show next steps]**

Next steps:
1. Implement state serialization (the `Pack` trait)
2. Implement instruction parsing (`TokenInstruction::unpack`)
3. Fill in the processor handlers
4. Write tests for each instruction

The foundation is solid. Now you can build on it.

**[Screen: Show resources]**

If you get stuck, check out:
- The Solana Program Library docs
- The Solana development guide
- Rust workspace documentation

Thanks for watching! If this helped, let me know what you'd like to see next.

---

## Production Notes

**Visual Elements:**
- Use terminal/IDE screen recordings
- Highlight code sections as you discuss them
- Show file tree diagrams between sections
- Display compilation output to show progress

**Voiceover Tips:**
- Speak naturally, as if explaining to a friend
- Pause after showing code to let viewers read
- Emphasize "why" not just "what"
- Use examples: "This is like..." to make concepts concrete

**Editing:**
- Cut long pauses and "um"s
- Add text overlays for key concepts
- Use zoom/pan on code sections being discussed
- Include timestamps in the description

**Code Display:**
- Use syntax highlighting
- Show line numbers for reference
- Highlight the specific line being discussed
- Keep code visible long enough to read

