# How to Build Solana Token Program Structure - Detailed Execution Guide

## 1. Task Goals

### 1.1 Current Task Goal

Build a complete foundational project structure for a Solana Token program, including:
- Workspace configuration
- Program Package structure
- Core module file organization
- Dependency management and build configuration

### 1.2 Role of This Sub-goal in the Overall Objective

The program structure serves as the foundational framework for the entire Solana Token project. Its roles include:

1. **Code Organization Foundation**: Provides clear module division and file organization for subsequent feature development (instruction processing, state management, error handling, etc.)
2. **Build System Configuration**: Configures dependencies, compilation options, program ID, etc., through `Cargo.toml` to ensure the program can compile and deploy correctly
3. **Development Efficiency**: A well-organized project structure enables developers to quickly locate code, understand module relationships, and collaborate effectively
4. **Maintainability**: Clear module boundaries and responsibility division make code easy to test, debug, and iterate
5. **Standardized Practices**: Follows Solana program development best practices, laying the foundation for future extensions (such as adding tests, documentation, clients, etc.)

**Analogy**: The program structure is like the foundation and framework when building a house. Only after the framework is set up can you add various functional modules (walls, doors, windows, utilities, etc.). Without a good structure, subsequent development becomes chaotic and difficult to maintain.

---

## 2. Execution Steps

### Step 1: Create Workspace Root Directory and Base Configuration Files

**Goal**: Establish project root directory and configure Rust workspace

**Specific Operations**:

1. **Create project root directory**
   ```bash
   mkdir -p /root/token
   cd /root/token
   ```

2. **Create root `Cargo.toml` (workspace configuration)**
   ```bash
   touch Cargo.toml
   ```
   
   Add the following content to `Cargo.toml`:
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

3. **Create Rust toolchain configuration file `rust-toolchain.toml`**
   ```bash
   touch rust-toolchain.toml
   ```
   
   Add content:
   ```toml
   [toolchain]
   channel = "stable"
   ```

4. **Create code formatting configuration `rustfmt.toml`**
   ```bash
   touch rustfmt.toml
   ```
   
   Add content:
   ```toml
   edition = "2021"
   max_width = 100
   ```

**Verification**: Run `cargo check` to confirm workspace configuration is correct (it's normal to see warnings about missing member packages at this stage)

---

### Step 2: Create Program Package Directory Structure

**Goal**: Create `program` directory and its base structure

**Specific Operations**:

1. **Create program package directory**
   ```bash
   mkdir -p program/src
   mkdir -p program/tests
   ```

2. **Create program package `Cargo.toml`**
   ```bash
   touch program/Cargo.toml
   ```
   
   Add to `program/Cargo.toml`:
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

**Verification**: Run `cd program && cargo check` to confirm package structure is correct

---

### Step 3: Create Core Module File Skeletons

**Goal**: Create all required Rust source files and establish module structure

**Specific Operations**:

1. **Create main library file `program/src/lib.rs`**
   ```bash
   touch program/src/lib.rs
   ```
   
   Add base content:
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

2. **Create entrypoint module `program/src/entrypoint.rs`**
   ```bash
   touch program/src/entrypoint.rs
   ```
   
   Add base content:
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

3. **Create error handling module `program/src/error.rs`**
   ```bash
   touch program/src/error.rs
   ```
   
   Add base content:
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

4. **Create instruction definition module `program/src/instruction.rs`**
   ```bash
   touch program/src/instruction.rs
   ```
   
   Add base content:
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

5. **Create state management module `program/src/state.rs`**
   ```bash
   touch program/src/state.rs
   ```
   
   Add base content:
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

6. **Create processor module `program/src/processor.rs`**
   ```bash
   touch program/src/processor.rs
   ```
   
   Add base content:
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

**Verification**: Run `cd program && cargo check` to confirm all module files can compile (warnings are acceptable, but there should be no errors)

---

### Step 4: Create Test Directory Structure

**Goal**: Establish test file organization

**Specific Operations**:

1. **Create test helper file `program/tests/setup.rs`**
   ```bash
   touch program/tests/setup.rs
   ```
   
   Add base content:
   ```rust
   //! Test setup utilities
   
   // TODO: Add test setup helpers
   ```

2. **Create processor test file `program/tests/processor.rs`**
   ```bash
   touch program/tests/processor.rs
   ```
   
   Add base content:
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

**Verification**: Run `cd program && cargo test` to confirm test framework can run

---

### Step 5: Create README and Documentation Files

**Goal**: Add project documentation

**Specific Operations**:

1. **Create program package README `program/README.md`**
   ```bash
   touch program/README.md
   ```
   
   Add base content:
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

2. **Create root directory README `README.md`**
   ```bash
   touch README.md
   ```
   
   Add base content:
   ```markdown
   # Token Program
   
   Solana Token Program workspace.
   
   ## Structure
   
   - `program/` - Main Solana program crate
   ```

**Verification**: Check that files exist and content is correct

---

### Step 6: Verify Project Structure Completeness

**Goal**: Ensure all files are in place and the project can compile

**Specific Operations**:

1. **Check directory structure**
   ```bash
   tree -L 3 /root/token
   ```
   
   Expected structure:
   ```
   /root/token/
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

2. **Verify compilation**
   ```bash
   cd /root/token
   cargo check
   ```

3. **Verify program package compilation**
   ```bash
   cd /root/token/program
   cargo check
   ```

4. **Verify test framework**
   ```bash
   cd /root/token/program
   cargo test
   ```

**Verification**: All commands should execute successfully (warnings are acceptable, but there should be no errors)

---

### Step 7: Configure Git Version Control (Optional but Recommended)

**Goal**: Initialize Git repository and add `.gitignore`

**Specific Operations**:

1. **Create `.gitignore`**
   ```bash
   touch .gitignore
   ```
   
   Add content:
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

2. **Initialize Git repository**
   ```bash
   git init
   git add .
   git commit -m "Initial project structure"
   ```

**Verification**: Run `git status` to confirm files are properly tracked

---

## 3. Task Completion Criteria

### 3.1 File Structure Completeness Criteria

✅ **All of the following conditions must be met**:

1. **Workspace Configuration**
   - [ ] Root directory has `Cargo.toml` with `[workspace]` configuration
   - [ ] `members` field includes `"program"`
   - [ ] `workspace.dependencies` is defined

2. **Program Package Structure**
   - [ ] `program/` directory exists
   - [ ] `program/Cargo.toml` exists and is correctly configured
   - [ ] `program/Cargo.toml` contains `[package.metadata.solana]` and `program-id`
   - [ ] `program/Cargo.toml` defines all required dependencies

3. **Source Code Files**
   - [ ] `program/src/lib.rs` exists with module declarations and base functions
   - [ ] `program/src/entrypoint.rs` exists with entrypoint macro
   - [ ] `program/src/error.rs` exists with `TokenError` enum defined
   - [ ] `program/src/instruction.rs` exists with `TokenInstruction` enum defined
   - [ ] `program/src/state.rs` exists with `Mint` and `Account` structs defined
   - [ ] `program/src/processor.rs` exists with `Processor` struct defined

4. **Test Files**
   - [ ] `program/tests/` directory exists
   - [ ] `program/tests/setup.rs` exists
   - [ ] `program/tests/processor.rs` exists

5. **Configuration Files**
   - [ ] `rust-toolchain.toml` exists
   - [ ] `rustfmt.toml` exists
   - [ ] `README.md` exists (both root and program directories)

### 3.2 Compilation Verification Criteria

✅ **All of the following conditions must be met**:

1. **Workspace Compilation**
   - [ ] Running `cargo check` in root directory executes successfully
   - [ ] No compilation errors (warnings are acceptable)

2. **Program Package Compilation**
   - [ ] Running `cd program && cargo check` executes successfully
   - [ ] All modules can be correctly parsed and compiled
   - [ ] No compilation errors (warnings are acceptable)

3. **Test Framework**
   - [ ] Running `cd program && cargo test` executes successfully
   - [ ] Test framework can run normally (even if tests are empty)

### 3.3 Code Quality Criteria

✅ **All of the following conditions must be met**:

1. **Module Organization**
   - [ ] `lib.rs` correctly exports all public modules
   - [ ] Module dependencies are clear
   - [ ] No circular dependencies

2. **Code Standards**
   - [ ] All files use `edition = "2021"`
   - [ ] Code can be formatted with `cargo fmt`
   - [ ] Code can be checked with `cargo clippy` (warnings are acceptable)

3. **Documentation Comments**
   - [ ] Each module file has module-level documentation comments (`//!`)
   - [ ] Public functions and structs have documentation comments

### 3.4 Functional Completeness Criteria

✅ **All of the following conditions must be met**:

1. **Entrypoint Configuration**
   - [ ] `entrypoint.rs` correctly uses `solana_program_entrypoint::entrypoint!` macro
   - [ ] Entry function correctly calls `Processor::process`

2. **Error Handling**
   - [ ] `TokenError` enum defines at least 8 error variants
   - [ ] Implements `From<TokenError> for ProgramError`

3. **Instruction Definition**
   - [ ] `TokenInstruction` enum defines 5 instruction variants
   - [ ] Implements `unpack` method (can be placeholder implementation)

4. **State Structures**
   - [ ] `Mint` struct contains required fields
   - [ ] `Account` struct contains required fields
   - [ ] Implements `Pack` trait (can be placeholder implementation)
   - [ ] Implements `IsInitialized` trait

5. **Processor Framework**
   - [ ] `Processor` struct exists
   - [ ] `process` method can route all instruction types (can be placeholder implementation)

---

## 4. Common Issues Troubleshooting

### Issue 1: `cargo check` reports error "could not find `Cargo.toml`"

**Cause**: Not executing command in the correct directory

**Solution**:
- Ensure you're in project root directory `/root/token` when running `cargo check`
- Or execute `cd program && cargo check` in the `program` directory

### Issue 2: Module not found errors

**Cause**: Module declarations in `lib.rs` don't match file names

**Solution**:
- Check that `pub mod` declarations in `lib.rs` match file names
- Ensure file names use underscores (e.g., `error.rs`) and module names also use underscores

### Issue 3: Dependency version conflicts

**Cause**: Workspace dependencies don't match program package dependency versions

**Solution**:
- Prefer using workspace dependencies: `solana-program-error = { workspace = true }`
- Ensure all shared dependencies are defined in workspace `Cargo.toml`

### Issue 4: Program ID configuration error

**Cause**: `program-id` format is incorrect

**Solution**:
- Ensure `program-id` is a valid Base58-encoded public key
- Format: `program-id = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"`

---

## 5. Next Steps

After completing the program structure setup, continue development in the following order:

1. **Implement State Serialization** (Milestone 3): Complete `Pack` trait implementation for `Mint` and `Account`
2. **Implement Instruction Processing** (Milestone 4): Complete instruction processing logic in `Processor`
3. **Implement Instruction Parsing** (Milestone 2): Complete `TokenInstruction::unpack` method
4. **Write Unit Tests** (Milestone 6): Add test cases for each module

---

## 6. Reference Resources

- [Solana Program Library Documentation](https://spl.solana.com/)
- [Solana Program Development Guide](https://docs.solana.com/developing/programming-model/overview)
- [Rust Workspace Documentation](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

---

**Document Created**: 2025-01-27  
**Last Updated**: 2025-01-27

