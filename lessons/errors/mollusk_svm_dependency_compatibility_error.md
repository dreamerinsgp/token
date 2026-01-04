# Mollusk SVM 依赖兼容性错误总结

本文档总结了在使用 Mollusk SVM 测试框架时遇到的依赖版本兼容性问题及其解决方案。

## 错误1：TransactionAccount 导入错误

### 背景
在编译 Solana Token MVP 程序时，使用 `mollusk-svm` 作为开发依赖进行测试。程序使用 Solana SDK v3.0.0+ 系列的依赖包，包括 `solana-transaction-context` v3.1.5。

### 错误
编译时出现以下错误：
```
error[E0432]: unresolved import `solana_transaction_context::TransactionAccount`
 --> /root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/mollusk-svm-keys-0.6.3/src/accounts.rs:9:70
  |
9 |     solana_transaction_context::{IndexOfAccount, InstructionAccount, TransactionAccount},
  |                                                                      ^^^^^^^^^^^^^^^^^^
  |                                                                      |
  |                                                                      no `TransactionAccount` in the root
  |                                                                      help: a similar name exists in the module: `TransactionAccounts`
```

同时还有相关的警告信息：
```
warning: unexpected `cfg` condition value: `solana`
warning: unexpected `cfg` condition value: `custom-heap`
warning: unexpected `cfg` condition value: `custom-panic`
```

### 原因
1. **版本不兼容**：`mollusk-svm` v0.6.3 是为旧版本的 Solana 依赖构建的，它期望 `solana-transaction-context` 中存在 `TransactionAccount` 类型（单数形式）。
2. **API 变更**：在 `solana-transaction-context` v3.1.5 中，`TransactionAccount` 被重命名为 `TransactionAccounts`（复数形式），导致旧版本的 `mollusk-svm-keys` 无法找到该类型。
3. **依赖链问题**：`mollusk-svm-keys-0.6.3` 作为 `mollusk-svm-0.6.3` 的依赖，直接依赖了 `solana-transaction-context`，但没有指定兼容的版本范围，导致 Cargo 解析到了不兼容的新版本。

### 方案
1. **更新 mollusk-svm 版本**：
   在 `program/Cargo.toml` 中将 `mollusk-svm` 从 `0.6.3` 更新到 `0.9.0`：
   ```toml
   [dev-dependencies]
   mollusk-svm = "0.9.0"
   solana-account = "3.0.0"
   ```

2. **验证更新**：
   运行 `cargo search mollusk-svm --limit 5` 确认最新版本可用：
   ```
   mollusk-svm = "0.9.0"            # SVM program test harness.
   mollusk-svm-keys = "0.8.1"       # SVM transaction keys utils.
   ```

3. **清理未使用的导入**（额外优化）：
   - 在 `error.rs` 中移除未使用的 `ProgramResult` 导入
   - 在 `instruction.rs` 中移除未使用的 `Instruction` 导入

4. **重新编译验证**：
   运行 `cargo build-sbf` 确认编译成功，错误已解决。

### 经验总结
1. **依赖版本管理**：在使用第三方测试框架时，需要确保其版本与项目使用的核心依赖版本兼容。定期检查并更新依赖包可以避免此类问题。
2. **错误信息分析**：当遇到 "unresolved import" 错误时，注意错误信息中的提示（如 "help: a similar name exists"），这往往指向 API 变更或重命名。
3. **版本选择策略**：对于开发依赖，可以更积极地使用较新版本，因为它们通常包含对最新 API 的支持和 bug 修复。
4. **依赖树检查**：使用 `cargo tree` 命令可以帮助理解依赖关系，识别潜在的版本冲突。

### 相关命令
```bash
# 检查依赖树
cargo tree -p mollusk-svm-keys --depth 1

# 搜索可用版本
cargo search mollusk-svm --limit 5

# 更新依赖后重新编译
cargo build-sbf
```

