# Building a Solana Token Program Structure - Step-by-Step Guide

## Why This Matters

Before we dive into code, let me explain why getting the structure right matters.

Think about building a house. You wouldn't start putting up walls before you have a foundation and frame, right? Same thing here. The program structure is your foundation. Without it, you'll end up with code scattered everywhere, dependencies that don't work together, and a project that's impossible to maintain.

Here's what a good structure gives you:
- **Clear organization**: You know exactly where to find things
- **Working builds**: Dependencies are configured correctly from the start
- **Easier development**: Adding new features doesn't break existing code
- **Better collaboration**: Other developers can understand your codebase quickly

Let's build this step by step.

---

## Step 1: Set Up the Workspace

First, we need to create the project root and configure Rust to treat this as a workspace.

**What you'll do:**

1. Create the project directory and navigate into it:
   ```bash
   mkdir -p token
   cd token
   ```

2. Create the workspace `Cargo.toml`:
   ```bash
   touch Cargo.toml
   ```
   
   Add this content:
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

   **Why this matters**: The workspace lets us share dependencies across multiple packages. Instead of repeating version numbers everywhere, we define them once here.

3. Create `rust-toolchain.toml` to pin the Rust version:
   ```bash
   touch rust-toolchain.toml
   ```
   
   ```toml
   [toolchain]
   channel = "stable"
   ```

4. Create `rustfmt.toml` for consistent formatting:
   ```bash
   touch rustfmt.toml
   ```
   
   ```toml
   edition = "2021"
   max_width = 100
   ```

**Check your work**: Run `cargo check`. It might complain about missing members, but that's expected—we'll create the program package next.

---

## Step 2: Create the Program Package

Now let's create the actual Solana program package.

**What you'll do:**

1. Create the directory structure:
   ```bash
   mkdir -p program/src
   mkdir -p program/tests
   ```

2. Create `program/Cargo.toml`:
   ```bash
   touch program/Cargo.toml
   ```
   
   Add this:
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

   **Key points**:
   - `crate-type = ["cdylib", "lib"]` tells Rust to build a shared library (needed for Solana)
   - `program-id` is the on-chain program identifier
   - `{ workspace = true }` pulls values from the workspace config

**Check your work**: Run `cd program && cargo check`. It should fail because we haven't created the source files yet, but it should recognize the package structure.

---

## Step 3: Create the Core Modules

This is where the actual code lives. We'll create skeleton files for each module.

### 3.1 Main Library File (`lib.rs`)

```bash
touch program/src/lib.rs
```

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

**What this does**: This is the main entry point for your library. It declares all your modules and provides utility functions to check program ownership.

### 3.2 Entry Point (`entrypoint.rs`)

```bash
touch program/src/entrypoint.rs
```

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

**What this does**: This is where Solana calls your program. Every transaction hits this function first, then routes to the appropriate handler.

### 3.3 Error Handling (`error.rs`)

```bash
touch program/src/error.rs
```

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

**What this does**: Defines all the ways your program can fail. Using `thiserror` makes error messages clearer and conversion to Solana's error type automatic.

### 3.4 Instructions (`instruction.rs`)

```bash
touch program/src/instruction.rs
```

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

**What this does**: Defines what actions your program can perform. Each variant represents a different operation users can request.

### 3.5 State Management (`state.rs`)

```bash
touch program/src/state.rs
```

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

**What this does**: Defines the data structures your program stores on-chain. `Mint` represents a token type, `Account` represents a user's token balance.

### 3.6 Processor (`processor.rs`)

```bash
touch program/src/processor.rs
```

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

**What this does**: Routes incoming instructions to the right handler. Right now it's just a skeleton—we'll fill in the actual logic later.

**Check your work**: Run `cd program && cargo check`. You should see warnings but no errors. The code compiles, even though the implementations are placeholders.

---

## Step 4: Set Up Tests

Tests help you catch bugs before deployment. Let's set up the test structure.

1. Create test setup file:
   ```bash
   touch program/tests/setup.rs
   ```
   
   ```rust
   //! Test setup utilities
   
   // TODO: Add test setup helpers
   ```

2. Create processor tests:
   ```bash
   touch program/tests/processor.rs
   ```
   
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

**Check your work**: Run `cd program && cargo test`. It should run successfully (even though the test does nothing).

---

## Step 5: Add Documentation

Documentation helps you remember what you built and helps others understand your code.

1. Create `program/README.md`:
   ```bash
   touch program/README.md
   ```
   
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

2. Create root `README.md`:
   ```bash
   touch README.md
   ```
   
   ```markdown
   # Token Program
   
   Solana Token Program workspace.
   
   ## Structure
   
   - `program/` - Main Solana program crate
   ```

---

## Step 6: Verify Everything Works

Let's make sure everything is in place and compiles correctly.

1. Check directory structure:
   ```bash
   tree -L 3 token
   ```
   
   You should see:
   ```
   token/
   ├── Cargo.toml
   ├── rust-toolchain.toml
   ├── rustfmt.toml
   ├── README.md
   └── program/
       ├── Cargo.toml
       ├── README.md
       ├── src/
       │   ├── lib.rs
       │   ├── entrypoint.rs
       │   ├── error.rs
       │   ├── instruction.rs
       │   ├── processor.rs
       │   └── state.rs
       └── tests/
           ├── setup.rs
           └── processor.rs
   ```

2. Verify compilation:
   ```bash
   cd token
   cargo check
   ```
   
   ```bash
   cd token/program
   cargo check
   ```

3. Run tests:
   ```bash
   cd token/program
   cargo test
   ```

All commands should succeed. You might see warnings, but no errors.

---

## Step 7: Set Up Version Control (Optional but Recommended)

Version control helps you track changes and collaborate.

1. Create `.gitignore`:
   ```bash
   touch .gitignore
   ```
   
   ```
   # Rust
   /target/
   **/*.rs.bk
   Cargo.lock
   
   # IDE
   .idea/
   .vscode/
   *.swp
   *.swo
   *~
   
   # OS
   .DS_Store
   Thumbs.db
   ```

2. Initialize Git:
   ```bash
   git init
   git add .
   git commit -m "Initial project structure"
   ```

---

## Common Issues and Solutions

### Issue: `cargo check` says "could not find `Cargo.toml`"

**Problem**: You're in the wrong directory.

**Solution**: Make sure you're in the project root (`token/`) or the program directory (`token/program/`).

### Issue: Module not found errors

**Problem**: Module declarations in `lib.rs` don't match filenames.

**Solution**: Check that `pub mod error;` matches `error.rs`, `pub mod instruction;` matches `instruction.rs`, etc.

### Issue: Dependency version conflicts

**Problem**: Workspace and package dependencies don't match.

**Solution**: Use `{ workspace = true }` in `program/Cargo.toml` to pull versions from the workspace config.

### Issue: Program ID format error

**Problem**: Invalid program ID format.

**Solution**: Make sure it's a valid Base58-encoded public key, like: `"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"`

---

## What's Next?

Now that you have the structure, here's what to tackle next:

1. **Implement state serialization**: Fill in the `Pack` trait implementations for `Mint` and `Account`
2. **Implement instruction parsing**: Complete `TokenInstruction::unpack`
3. **Implement instruction handlers**: Fill in the `Processor::process` match arms
4. **Write tests**: Add test cases for each instruction type

---

## Resources

- [Solana Program Library Documentation](https://spl.solana.com/)
- [Solana Program Development Guide](https://docs.solana.com/developing/programming-model/overview)
- [Rust Workspace Documentation](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

---

**Created**: 2025-01-27  
**Last Updated**: 2025-01-27

