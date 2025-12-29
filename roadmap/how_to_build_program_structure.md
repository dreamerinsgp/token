# 如何构建 Solana Token 程序结构 - 详细执行指南

## 1. 任务目标

### 1.1 当前任务目标

构建一个完整的 Solana Token 程序的基础项目结构，包括：
- 工作空间（Workspace）配置
- 程序包（Program Package）结构
- 核心模块文件组织
- 依赖管理和构建配置

### 1.2 该子目标在最终目标中的作用

程序结构是整个 Solana Token 项目的基础框架，它的作用包括：

1. **代码组织基础**：为后续的功能开发（指令处理、状态管理、错误处理等）提供清晰的模块划分和文件组织
2. **构建系统配置**：通过 `Cargo.toml` 配置依赖、编译选项、程序 ID 等，确保程序能够正确编译和部署
3. **开发效率提升**：合理的项目结构让开发者能够快速定位代码、理解模块关系、进行协作开发
4. **可维护性保障**：清晰的模块边界和职责划分，使得代码易于测试、调试和迭代
5. **标准化实践**：遵循 Solana 程序开发的最佳实践，为后续扩展（如添加测试、文档、客户端等）奠定基础

**类比理解**：程序结构就像建房子时的地基和框架，只有先把框架搭建好，才能在上面添加各种功能模块（墙壁、门窗、水电等）。没有良好的结构，后续的开发会变得混乱且难以维护。

---

## 2. 执行步骤

### Step 1: 创建工作空间根目录和基础配置文件

**目标**：建立项目根目录，配置 Rust 工作空间

**具体操作**：

1. **创建项目根目录**
   ```bash
   mkdir -p /root/token
   cd /root/token
   ```

2. **创建根目录 `Cargo.toml`（工作空间配置）**
   ```bash
   touch Cargo.toml
   ```
   
   在 `Cargo.toml` 中添加以下内容：
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

3. **创建 Rust 工具链配置文件 `rust-toolchain.toml`**
   ```bash
   touch rust-toolchain.toml
   ```
   
   添加内容：
   ```toml
   [toolchain]
   channel = "stable"
   ```

4. **创建代码格式化配置 `rustfmt.toml`**
   ```bash
   touch rustfmt.toml
   ```
   
   添加内容：
   ```toml
   edition = "2021"
   max_width = 100
   ```

**验证**：运行 `cargo check` 确认工作空间配置正确（此时会提示缺少成员包，这是正常的）

---

### Step 2: 创建程序包（Program Package）目录结构

**目标**：创建 `program` 目录及其基础结构

**具体操作**：

1. **创建程序包目录**
   ```bash
   mkdir -p program/src
   mkdir -p program/tests
   ```

2. **创建程序包的 `Cargo.toml`**
   ```bash
   touch program/Cargo.toml
   ```
   
   在 `program/Cargo.toml` 中添加：
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

**验证**：运行 `cd program && cargo check` 确认包结构正确

---

### Step 3: 创建核心模块文件骨架

**目标**：创建所有必需的 Rust 源文件，建立模块结构

**具体操作**：

1. **创建主库文件 `program/src/lib.rs`**
   ```bash
   touch program/src/lib.rs
   ```
   
   添加基础内容：
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

2. **创建入口点模块 `program/src/entrypoint.rs`**
   ```bash
   touch program/src/entrypoint.rs
   ```
   
   添加基础内容：
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

3. **创建错误处理模块 `program/src/error.rs`**
   ```bash
   touch program/src/error.rs
   ```
   
   添加基础内容：
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

4. **创建指令定义模块 `program/src/instruction.rs`**
   ```bash
   touch program/src/instruction.rs
   ```
   
   添加基础内容：
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

5. **创建状态管理模块 `program/src/state.rs`**
   ```bash
   touch program/src/state.rs
   ```
   
   添加基础内容：
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

6. **创建处理器模块 `program/src/processor.rs`**
   ```bash
   touch program/src/processor.rs
   ```
   
   添加基础内容：
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

**验证**：运行 `cd program && cargo check` 确认所有模块文件可以编译（可能会有警告，但不应有错误）

---

### Step 4: 创建测试目录结构

**目标**：建立测试文件组织

**具体操作**：

1. **创建测试辅助文件 `program/tests/setup.rs`**
   ```bash
   touch program/tests/setup.rs
   ```
   
   添加基础内容：
   ```rust
   //! Test setup utilities
   
   // TODO: Add test setup helpers
   ```

2. **创建处理器测试文件 `program/tests/processor.rs`**
   ```bash
   touch program/tests/processor.rs
   ```
   
   添加基础内容：
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

**验证**：运行 `cd program && cargo test` 确认测试框架可以运行

---

### Step 5: 创建 README 和文档文件

**目标**：添加项目说明文档

**具体操作**：

1. **创建程序包 README `program/README.md`**
   ```bash
   touch program/README.md
   ```
   
   添加基础内容：
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

2. **创建根目录 README `README.md`**
   ```bash
   touch README.md
   ```
   
   添加基础内容：
   ```markdown
   # Token Program
   
   Solana Token Program workspace.
   
   ## Structure
   
   - `program/` - Main Solana program crate
   ```

**验证**：检查文件是否存在且内容正确

---

### Step 6: 验证项目结构完整性

**目标**：确保所有文件就位，项目可以编译

**具体操作**：

1. **检查目录结构**
   ```bash
   tree -L 3 /root/token
   ```
   
   预期结构：
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

2. **验证编译**
   ```bash
   cd /root/token
   cargo check
   ```

3. **验证程序包编译**
   ```bash
   cd /root/token/program
   cargo check
   ```

4. **验证测试框架**
   ```bash
   cd /root/token/program
   cargo test
   ```

**验证**：所有命令应成功执行（允许有警告，但不应有错误）

---

### Step 7: 配置 Git 版本控制（可选但推荐）

**目标**：初始化 Git 仓库，添加 `.gitignore`

**具体操作**：

1. **创建 `.gitignore`**
   ```bash
   touch .gitignore
   ```
   
   添加内容：
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

2. **初始化 Git 仓库**
   ```bash
   git init
   git add .
   git commit -m "Initial project structure"
   ```

**验证**：运行 `git status` 确认文件已正确跟踪

---

## 3. 任务完成标准

### 3.1 文件结构完整性标准

✅ **必须满足以下所有条件**：

1. **工作空间配置**
   - [ ] 根目录存在 `Cargo.toml`，包含 `[workspace]` 配置
   - [ ] `members` 字段包含 `"program"`
   - [ ] 定义了 `workspace.dependencies`

2. **程序包结构**
   - [ ] `program/` 目录存在
   - [ ] `program/Cargo.toml` 存在且配置正确
   - [ ] `program/Cargo.toml` 中包含 `[package.metadata.solana]` 和 `program-id`
   - [ ] `program/Cargo.toml` 中定义了所有必需的依赖

3. **源代码文件**
   - [ ] `program/src/lib.rs` 存在，包含模块声明和基础函数
   - [ ] `program/src/entrypoint.rs` 存在，包含入口点宏
   - [ ] `program/src/error.rs` 存在，定义了 `TokenError` 枚举
   - [ ] `program/src/instruction.rs` 存在，定义了 `TokenInstruction` 枚举
   - [ ] `program/src/state.rs` 存在，定义了 `Mint` 和 `Account` 结构体
   - [ ] `program/src/processor.rs` 存在，定义了 `Processor` 结构体

4. **测试文件**
   - [ ] `program/tests/` 目录存在
   - [ ] `program/tests/setup.rs` 存在
   - [ ] `program/tests/processor.rs` 存在

5. **配置文件**
   - [ ] `rust-toolchain.toml` 存在
   - [ ] `rustfmt.toml` 存在
   - [ ] `README.md` 存在（根目录和 program 目录）

### 3.2 编译验证标准

✅ **必须满足以下所有条件**：

1. **工作空间编译**
   - [ ] 运行 `cargo check` 在根目录成功执行
   - [ ] 无编译错误（允许有警告）

2. **程序包编译**
   - [ ] 运行 `cd program && cargo check` 成功执行
   - [ ] 所有模块可以正确解析和编译
   - [ ] 无编译错误（允许有警告）

3. **测试框架**
   - [ ] 运行 `cd program && cargo test` 成功执行
   - [ ] 测试框架可以正常运行（即使测试为空）

### 3.3 代码质量标准

✅ **必须满足以下所有条件**：

1. **模块组织**
   - [ ] `lib.rs` 正确导出所有公共模块
   - [ ] 模块之间的依赖关系清晰
   - [ ] 没有循环依赖

2. **代码规范**
   - [ ] 所有文件使用 `edition = "2021"`
   - [ ] 代码可以通过 `cargo fmt` 格式化
   - [ ] 代码可以通过 `cargo clippy` 检查（允许有警告）

3. **文档注释**
   - [ ] 每个模块文件有模块级文档注释（`//!`）
   - [ ] 公共函数和结构体有文档注释

### 3.4 功能完整性标准

✅ **必须满足以下所有条件**：

1. **入口点配置**
   - [ ] `entrypoint.rs` 正确使用 `solana_program_entrypoint::entrypoint!` 宏
   - [ ] 入口函数正确调用 `Processor::process`

2. **错误处理**
   - [ ] `TokenError` 枚举定义了至少 8 个错误变体
   - [ ] 实现了 `From<TokenError> for ProgramError`

3. **指令定义**
   - [ ] `TokenInstruction` 枚举定义了 5 个指令变体
   - [ ] 实现了 `unpack` 方法（可以是占位实现）

4. **状态结构**
   - [ ] `Mint` 结构体包含必需字段
   - [ ] `Account` 结构体包含必需字段
   - [ ] 实现了 `Pack` trait（可以是占位实现）
   - [ ] 实现了 `IsInitialized` trait

5. **处理器框架**
   - [ ] `Processor` 结构体存在
   - [ ] `process` 方法可以路由所有指令类型（可以是占位实现）

---

## 4. 常见问题排查

### 问题 1: `cargo check` 报错 "could not find `Cargo.toml`"

**原因**：不在正确的目录下执行命令

**解决方案**：
- 确保在项目根目录 `/root/token` 执行 `cargo check`
- 或在 `program` 目录执行 `cd program && cargo check`

### 问题 2: 模块找不到错误

**原因**：`lib.rs` 中模块声明与文件名不匹配

**解决方案**：
- 检查 `lib.rs` 中的 `pub mod` 声明是否与文件名一致
- 确保文件名使用下划线（如 `error.rs`），模块名也使用下划线

### 问题 3: 依赖版本冲突

**原因**：工作空间依赖与程序包依赖版本不一致

**解决方案**：
- 优先使用工作空间依赖：`solana-program-error = { workspace = true }`
- 确保工作空间 `Cargo.toml` 中定义了所有共享依赖

### 问题 4: 程序 ID 配置错误

**原因**：`program-id` 格式不正确

**解决方案**：
- 确保 `program-id` 是有效的 Base58 编码的公钥
- 格式：`program-id = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"`

---

## 5. 下一步行动

完成程序结构搭建后，按照以下顺序继续开发：

1. **实现状态序列化**（Milestone 3）：完善 `Mint` 和 `Account` 的 `Pack` trait 实现
2. **实现指令处理**（Milestone 4）：完善 `Processor` 中各个指令的处理逻辑
3. **实现指令解析**（Milestone 2）：完善 `TokenInstruction::unpack` 方法
4. **编写单元测试**（Milestone 6）：为各个模块添加测试用例

---

## 6. 参考资源

- [Solana Program Library Documentation](https://spl.solana.com/)
- [Solana Program Development Guide](https://docs.solana.com/developing/programming-model/overview)
- [Rust Workspace Documentation](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

---

**文档创建时间**：2025-01-27  
**最后更新**：2025-01-27

