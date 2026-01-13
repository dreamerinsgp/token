/// 测试 InitializeMint 指令
/// 
/// 测试场景：
/// 1. 成功初始化 mint
/// 2. 验证 mint 字段设置正确
/// 3. 测试重复初始化错误
/// 4. 测试没有 freeze_authority 的情况

use {
    spl_token::{
        error::TokenError,
        instruction::TokenInstruction,
        processor::Processor,
        state::{Account, Mint},
        id,
    },
    solana_account_info::AccountInfo,
    solana_program_option::COption,
    solana_program_pack::Pack,
    solana_pubkey::Pubkey,
    solana_rent::Rent,
    solana_sysvar::{rent, Sysvar},
    std::str::FromStr,
};

    #[test]
fn test_initialize_mint() {
    // ========== 第一部分：设置测试环境 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();
    let decimals = 9u8;
    
    // 创建默认的租金系统变量（用于测试）
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let rent_exempt_lamports = rent.minimum_balance(mint_data_len);
    
    // ========== 第二部分：创建 mint 账户 ==========
    
    // 创建未初始化的 mint 账户数据
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = rent_exempt_lamports;
    
    // 创建 mint 账户信息
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true, // 可写
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    // ========== 第三部分：创建租金系统变量账户 ==========
    
    let rent_sysvar = rent::id();
    // 使用 bincode 序列化 Rent（与 SysvarSerialize 兼容）
    let mut rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    
    // sysvar 程序 ID（Rent sysvar 的程序 ID）
    let sysvar_program_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    
    let rent_account_info = AccountInfo::new(
        &rent_sysvar,
        false,
        false, // 只读
        &mut rent_lamports,
        &mut rent_data,
        &sysvar_program_id,
        false,
    );
    
    // ========== 第四部分：测试成功初始化 ==========
    
    let accounts = vec![mint_account_info.clone(), rent_account_info.clone()];
    
    let result = Processor::process_initialize_mint(
        &program_id,
        &accounts,
        decimals,
        mint_authority,
        COption::Some(freeze_authority),
    );
    
    // 验证初始化成功
    assert!(result.is_ok(), "InitializeMint 应该成功");
    
    // ========== 第五部分：验证 mint 数据 ==========
    
    // 从账户数据中解包 mint
    let mint = Mint::unpack_from_slice(&mint_account_info.data.borrow()).unwrap();
    
    // 验证字段
    assert_eq!(mint.decimals, decimals, "decimals 应该匹配");
    assert_eq!(mint.mint_authority, COption::Some(mint_authority), "mint_authority 应该匹配");
    assert_eq!(mint.freeze_authority, COption::Some(freeze_authority), "freeze_authority 应该匹配");
    assert_eq!(mint.supply, 0, "supply 应该初始化为 0");
    assert!(mint.is_initialized, "is_initialized 应该为 true");
    
    // ========== 第六部分：测试重复初始化错误 ==========
    
    // 尝试再次初始化同一个 mint
    let result = Processor::process_initialize_mint(
        &program_id,
        &accounts,
        decimals,
        mint_authority,
        COption::Some(freeze_authority),
    );
    
    // 应该返回 AlreadyInitialized 错误
    assert!(result.is_err(), "重复初始化应该失败");
    assert_eq!(
        result.unwrap_err(),
        TokenError::AlreadyInitialized.into(),
        "应该返回 AlreadyInitialized 错误"
    );
    
    println!("✅ 测试通过：InitializeMint 指令测试成功");
}

#[test]
fn test_initialize_mint_without_freeze_authority() {
    // 测试没有 freeze_authority 的情况
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let rent_exempt_lamports = rent.minimum_balance(mint_data_len);
    
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = rent_exempt_lamports;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar = rent::id();
    let mut rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    
    let sysvar_program_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    
    let rent_account_info = AccountInfo::new(
        &rent_sysvar,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data,
        &sysvar_program_id,
        false,
    );
    
    let accounts = vec![mint_account_info.clone(), rent_account_info.clone()];
    
    // 使用 COption::None 作为 freeze_authority
    let result = Processor::process_initialize_mint(
        &program_id,
        &accounts,
        decimals,
        mint_authority,
        COption::None, // 没有 freeze authority
    );
    
    assert!(result.is_ok(), "没有 freeze_authority 的初始化应该成功");
    
    // 验证 freeze_authority 为 None
    let mint = Mint::unpack_from_slice(&mint_account_info.data.borrow()).unwrap();
    assert_eq!(mint.freeze_authority, COption::None, "freeze_authority 应该为 None");
    
    println!("✅ 测试通过：没有 freeze_authority 的 InitializeMint 测试成功");
}

#[test]
fn test_transfer() {
    // ========== 第一部分：设置测试环境 ==========
    
    let program_id = id();
    
    // 创建 mint
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    // 创建源账户和目标账户
    let source_account_keypair = Pubkey::new_unique();
    let destination_account_keypair = Pubkey::new_unique();
    let source_owner = Pubkey::new_unique();
    let destination_owner = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // ========== 第二部分：创建并初始化 mint ==========
    
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::None,
    ).unwrap();
    
    // ========== 第三部分：初始化源账户和目标账户 ==========
    
    let mut source_account_data = vec![0u8; account_data_len];
    let mut source_account_lamports = account_rent_exempt;
    
    let source_account_info = AccountInfo::new(
        &source_account_keypair,
        false,
        true,
        &mut source_account_lamports,
        &mut source_account_data,
        &program_id,
        false,
    );
    
    let mut destination_account_data = vec![0u8; account_data_len];
    let mut destination_account_lamports = account_rent_exempt;
    
    let destination_account_info = AccountInfo::new(
        &destination_account_keypair,
        false,
        true,
        &mut destination_account_lamports,
        &mut destination_account_data,
        &program_id,
        false,
    );
    
    // 初始化源账户
    let mut source_owner_lamports = 0u64;
    let mut source_owner_data = vec![];
    let source_owner_account_id = Pubkey::default();
    let source_owner_account_info = AccountInfo::new(
        &source_owner,
        false,
        false,
        &mut source_owner_lamports,
        &mut source_owner_data,
        &source_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            source_account_info.clone(),
            mint_account_info.clone(),
            source_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // 初始化目标账户
    let mut destination_owner_lamports = 0u64;
    let mut destination_owner_data = vec![];
    let destination_owner_account_id = Pubkey::default();
    let destination_owner_account_info = AccountInfo::new(
        &destination_owner,
        false,
        false,
        &mut destination_owner_lamports,
        &mut destination_owner_data,
        &destination_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            destination_account_info.clone(),
            mint_account_info.clone(),
            destination_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // ========== 第四部分：给源账户充值（通过直接修改账户数据） ==========
    
    // 为了测试 Transfer，我们需要先给源账户一些代币
    // 在实际场景中，这通常通过 MintTo 完成，但为了简化测试，我们直接设置余额
    let mut source_account = Account::unpack(&source_account_info.data.borrow()).unwrap();
    source_account.amount = 1000; // 设置初始余额为 1000
    Account::pack(source_account, &mut source_account_info.data.borrow_mut()).unwrap();
    
    // ========== 第五部分：执行 Transfer ==========
    
    let transfer_amount = 500u64;
    
    // 创建 authority 账户（源账户的所有者，必须是 signer）
    let mut authority_lamports = 0u64;
    let mut authority_data = vec![];
    let authority_account_id = Pubkey::default();
    let authority_account_info = AccountInfo::new(
        &source_owner,
        true, // is_signer = true
        false,
        &mut authority_lamports,
        &mut authority_data,
        &authority_account_id,
        false,
    );
    
    let accounts = vec![
        source_account_info.clone(),
        destination_account_info.clone(),
        authority_account_info,
    ];
    
    // 执行 Transfer
    Processor::process_transfer(&program_id, &accounts, transfer_amount).unwrap();
    
    // ========== 第六部分：验证转账结果 ==========
    
    let source_account_after = Account::unpack(&source_account_info.data.borrow()).unwrap();
    let destination_account_after = Account::unpack(&destination_account_info.data.borrow()).unwrap();
    
    assert_eq!(source_account_after.amount, 500, "源账户余额应该减少 500");
    assert_eq!(destination_account_after.amount, 500, "目标账户余额应该增加 500");
    assert_eq!(source_account_after.mint, mint_keypair, "源账户 mint 应该匹配");
    assert_eq!(destination_account_after.mint, mint_keypair, "目标账户 mint 应该匹配");
    
    println!("✅ 测试通过：Transfer 指令测试成功");
}

#[test]
fn test_transfer_insufficient_funds() {
    // ========== 测试余额不足的情况 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    let source_account_keypair = Pubkey::new_unique();
    let destination_account_keypair = Pubkey::new_unique();
    let source_owner = Pubkey::new_unique();
    let destination_owner = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // 创建并初始化 mint
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::None,
    ).unwrap();
    
    // 初始化账户
    let mut source_account_data = vec![0u8; account_data_len];
    let mut source_account_lamports = account_rent_exempt;
    
    let source_account_info = AccountInfo::new(
        &source_account_keypair,
        false,
        true,
        &mut source_account_lamports,
        &mut source_account_data,
        &program_id,
        false,
    );
    
    let mut destination_account_data = vec![0u8; account_data_len];
    let mut destination_account_lamports = account_rent_exempt;
    
    let destination_account_info = AccountInfo::new(
        &destination_account_keypair,
        false,
        true,
        &mut destination_account_lamports,
        &mut destination_account_data,
        &program_id,
        false,
    );
    
    let mut source_owner_lamports = 0u64;
    let mut source_owner_data = vec![];
    let source_owner_account_id = Pubkey::default();
    let source_owner_account_info = AccountInfo::new(
        &source_owner,
        false,
        false,
        &mut source_owner_lamports,
        &mut source_owner_data,
        &source_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            source_account_info.clone(),
            mint_account_info.clone(),
            source_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    let mut destination_owner_lamports = 0u64;
    let mut destination_owner_data = vec![];
    let destination_owner_account_id = Pubkey::default();
    let destination_owner_account_info = AccountInfo::new(
        &destination_owner,
        false,
        false,
        &mut destination_owner_lamports,
        &mut destination_owner_data,
        &destination_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            destination_account_info.clone(),
            mint_account_info.clone(),
            destination_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // 设置源账户余额为 100，但尝试转账 500
    let mut source_account = Account::unpack(&source_account_info.data.borrow()).unwrap();
    source_account.amount = 100;
    Account::pack(source_account, &mut source_account_info.data.borrow_mut()).unwrap();
    
    let transfer_amount = 500u64;
    
    let mut authority_lamports = 0u64;
    let mut authority_data = vec![];
    let authority_account_id = Pubkey::default();
    let authority_account_info = AccountInfo::new(
        &source_owner,
        true,
        false,
        &mut authority_lamports,
        &mut authority_data,
        &authority_account_id,
        false,
    );
    
    let accounts = vec![
        source_account_info.clone(),
        destination_account_info.clone(),
        authority_account_info,
    ];
    
    // 尝试转账应该失败
    let result = Processor::process_transfer(&program_id, &accounts, transfer_amount);
    assert!(result.is_err(), "余额不足的转账应该失败");
    
    if let Err(err) = result {
        assert_eq!(
            err,
            TokenError::InsufficientFunds.into(),
            "应该返回 InsufficientFunds 错误"
        );
    }
    
    println!("✅ 测试通过：余额不足的转账应该失败");
}

#[test]
fn test_mint_to() {
    // ========== 第一部分：设置测试环境 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    // 创建目标账户
    let destination_account_keypair = Pubkey::new_unique();
    let destination_owner = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // ========== 第二部分：创建并初始化 mint ==========
    
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::None,
    ).unwrap();
    
    // ========== 第三部分：初始化目标账户 ==========
    
    let mut destination_account_data = vec![0u8; account_data_len];
    let mut destination_account_lamports = account_rent_exempt;
    
    let destination_account_info = AccountInfo::new(
        &destination_account_keypair,
        false,
        true,
        &mut destination_account_lamports,
        &mut destination_account_data,
        &program_id,
        false,
    );
    
    let mut destination_owner_lamports = 0u64;
    let mut destination_owner_data = vec![];
    let destination_owner_account_id = Pubkey::default();
    let destination_owner_account_info = AccountInfo::new(
        &destination_owner,
        false,
        false,
        &mut destination_owner_lamports,
        &mut destination_owner_data,
        &destination_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            destination_account_info.clone(),
            mint_account_info.clone(),
            destination_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // ========== 第四部分：执行 MintTo ==========
    
    let mint_amount = 1000u64;
    
    // 创建 mint authority 账户（必须是 signer）
    let mut authority_lamports = 0u64;
    let mut authority_data = vec![];
    let authority_account_id = Pubkey::default();
    let authority_account_info = AccountInfo::new(
        &mint_authority,
        true, // is_signer = true
        false,
        &mut authority_lamports,
        &mut authority_data,
        &authority_account_id,
        false,
    );
    
    let accounts = vec![
        mint_account_info.clone(),
        destination_account_info.clone(),
        authority_account_info,
    ];
    
    // 执行 MintTo
    Processor::process_mint_to(&program_id, &accounts, mint_amount).unwrap();
    
    // ========== 第五部分：验证铸币结果 ==========
    
    let mint_after = Mint::unpack(&mint_account_info.data.borrow()).unwrap();
    let destination_account_after = Account::unpack(&destination_account_info.data.borrow()).unwrap();
    
    assert_eq!(mint_after.supply, mint_amount, "Mint supply 应该增加 1000");
    assert_eq!(destination_account_after.amount, mint_amount, "目标账户余额应该增加 1000");
    assert_eq!(destination_account_after.mint, mint_keypair, "目标账户 mint 应该匹配");
    
    println!("✅ 测试通过：MintTo 指令测试成功");
}

#[test]
fn test_mint_to_unauthorized() {
    // ========== 测试未授权铸币的情况 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let unauthorized_authority = Pubkey::new_unique(); // 未授权的账户
    let decimals = 6u8;
    
    let destination_account_keypair = Pubkey::new_unique();
    let destination_owner = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // 创建并初始化 mint
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::None,
    ).unwrap();
    
    // 初始化目标账户
    let mut destination_account_data = vec![0u8; account_data_len];
    let mut destination_account_lamports = account_rent_exempt;
    
    let destination_account_info = AccountInfo::new(
        &destination_account_keypair,
        false,
        true,
        &mut destination_account_lamports,
        &mut destination_account_data,
        &program_id,
        false,
    );
    
    let mut destination_owner_lamports = 0u64;
    let mut destination_owner_data = vec![];
    let destination_owner_account_id = Pubkey::default();
    let destination_owner_account_info = AccountInfo::new(
        &destination_owner,
        false,
        false,
        &mut destination_owner_lamports,
        &mut destination_owner_data,
        &destination_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            destination_account_info.clone(),
            mint_account_info.clone(),
            destination_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // 尝试使用未授权的账户铸币
    let mint_amount = 1000u64;
    
    let mut unauthorized_authority_lamports = 0u64;
    let mut unauthorized_authority_data = vec![];
    let unauthorized_authority_account_id = Pubkey::default();
    let unauthorized_authority_account_info = AccountInfo::new(
        &unauthorized_authority,
        true, // is_signer = true, but not the mint authority
        false,
        &mut unauthorized_authority_lamports,
        &mut unauthorized_authority_data,
        &unauthorized_authority_account_id,
        false,
    );
    
    let accounts = vec![
        mint_account_info.clone(),
        destination_account_info.clone(),
        unauthorized_authority_account_info,
    ];
    
    // 尝试铸币应该失败
    let result = Processor::process_mint_to(&program_id, &accounts, mint_amount);
    assert!(result.is_err(), "未授权的铸币应该失败");
    
    if let Err(err) = result {
        assert_eq!(
            err,
            TokenError::InvalidOwner.into(),
            "应该返回 InvalidOwner 错误"
        );
    }
    
    println!("✅ 测试通过：未授权的铸币应该失败");
}

#[test]
fn test_burn() {
    // ========== 第一部分：设置测试环境 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    // 创建账户
    let account_keypair = Pubkey::new_unique();
    let account_owner = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // ========== 第二部分：创建并初始化 mint ==========
    
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::None,
    ).unwrap();
    
    // ========== 第三部分：初始化账户 ==========
    
    let mut account_data = vec![0u8; account_data_len];
    let mut account_lamports = account_rent_exempt;
    
    let account_info = AccountInfo::new(
        &account_keypair,
        false,
        true,
        &mut account_lamports,
        &mut account_data,
        &program_id,
        false,
    );
    
    let mut account_owner_lamports = 0u64;
    let mut account_owner_data = vec![];
    let account_owner_account_id = Pubkey::default();
    let account_owner_account_info = AccountInfo::new(
        &account_owner,
        false,
        false,
        &mut account_owner_lamports,
        &mut account_owner_data,
        &account_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            account_info.clone(),
            mint_account_info.clone(),
            account_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // ========== 第四部分：先给账户充值（通过 MintTo） ==========
    
    // 先铸币 1000 个代币到账户
    let mint_amount = 1000u64;
    
    let mut mint_authority_lamports = 0u64;
    let mut mint_authority_data = vec![];
    let mint_authority_account_id = Pubkey::default();
    let mint_authority_account_info = AccountInfo::new(
        &mint_authority,
        true, // is_signer = true
        false,
        &mut mint_authority_lamports,
        &mut mint_authority_data,
        &mint_authority_account_id,
        false,
    );
    
    let mint_to_accounts = vec![
        mint_account_info.clone(),
        account_info.clone(),
        mint_authority_account_info,
    ];
    
    Processor::process_mint_to(&program_id, &mint_to_accounts, mint_amount).unwrap();
    
    // ========== 第五部分：执行 Burn ==========
    
    let burn_amount = 300u64;
    
    // 创建账户所有者账户（必须是 signer）
    let mut owner_lamports = 0u64;
    let mut owner_data = vec![];
    let owner_account_id = Pubkey::default();
    let owner_account_info = AccountInfo::new(
        &account_owner,
        true, // is_signer = true
        false,
        &mut owner_lamports,
        &mut owner_data,
        &owner_account_id,
        false,
    );
    
    let accounts = vec![
        account_info.clone(),
        mint_account_info.clone(),
        owner_account_info,
    ];
    
    // 执行 Burn
    Processor::process_burn(&program_id, &accounts, burn_amount).unwrap();
    
    // ========== 第六部分：验证销毁结果 ==========
    
    let account_after = Account::unpack(&account_info.data.borrow()).unwrap();
    let mint_after = Mint::unpack(&mint_account_info.data.borrow()).unwrap();
    
    assert_eq!(account_after.amount, 700, "账户余额应该减少 300，剩余 700");
    assert_eq!(mint_after.supply, 700, "Mint supply 应该减少 300，剩余 700");
    assert_eq!(account_after.mint, mint_keypair, "账户 mint 应该匹配");
    
    println!("✅ 测试通过：Burn 指令测试成功");
}

    #[test]
fn test_burn_insufficient_funds() {
    // ========== 测试余额不足的情况 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    let account_keypair = Pubkey::new_unique();
    let account_owner = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // 创建并初始化 mint
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::None,
    ).unwrap();
    
    // 初始化账户
    let mut account_data = vec![0u8; account_data_len];
    let mut account_lamports = account_rent_exempt;
    
    let account_info = AccountInfo::new(
        &account_keypair,
        false,
        true,
        &mut account_lamports,
        &mut account_data,
        &program_id,
        false,
    );
    
    let mut account_owner_lamports = 0u64;
    let mut account_owner_data = vec![];
    let account_owner_account_id = Pubkey::default();
    let account_owner_account_info = AccountInfo::new(
        &account_owner,
        false,
        false,
        &mut account_owner_lamports,
        &mut account_owner_data,
        &account_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            account_info.clone(),
            mint_account_info.clone(),
            account_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // 先铸币 100 个代币
    let mint_amount = 100u64;
    
    let mut mint_authority_lamports = 0u64;
    let mut mint_authority_data = vec![];
    let mint_authority_account_id = Pubkey::default();
    let mint_authority_account_info = AccountInfo::new(
        &mint_authority,
        true,
        false,
        &mut mint_authority_lamports,
        &mut mint_authority_data,
        &mint_authority_account_id,
        false,
    );
    
    let mint_to_accounts = vec![
        mint_account_info.clone(),
        account_info.clone(),
        mint_authority_account_info,
    ];
    
    Processor::process_mint_to(&program_id, &mint_to_accounts, mint_amount).unwrap();
    
    // 尝试销毁 500 个代币（但只有 100 个）
    let burn_amount = 500u64;
    
    let mut owner_lamports = 0u64;
    let mut owner_data = vec![];
    let owner_account_id = Pubkey::default();
    let owner_account_info = AccountInfo::new(
        &account_owner,
        true,
        false,
        &mut owner_lamports,
        &mut owner_data,
        &owner_account_id,
        false,
    );
    
    let accounts = vec![
        account_info.clone(),
        mint_account_info.clone(),
        owner_account_info,
    ];
    
    // 尝试销毁应该失败
    let result = Processor::process_burn(&program_id, &accounts, burn_amount);
    assert!(result.is_err(), "余额不足的销毁应该失败");
    
    if let Err(err) = result {
        assert_eq!(
            err,
            TokenError::InsufficientFunds.into(),
            "应该返回 InsufficientFunds 错误"
        );
    }
    
    println!("✅ 测试通过：余额不足的销毁应该失败");
}

#[test]
fn test_approve() {
    // ========== 第一部分：设置测试环境 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    // 创建账户和委托账户
    let account_keypair = Pubkey::new_unique();
    let account_owner = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // ========== 第二部分：创建并初始化 mint ==========
    
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::None,
    ).unwrap();
    
    // ========== 第三部分：初始化账户 ==========
    
    let mut account_data = vec![0u8; account_data_len];
    let mut account_lamports = account_rent_exempt;
    
    let account_info = AccountInfo::new(
        &account_keypair,
        false,
        true,
        &mut account_lamports,
        &mut account_data,
        &program_id,
        false,
    );
    
    let mut account_owner_lamports = 0u64;
    let mut account_owner_data = vec![];
    let account_owner_account_id = Pubkey::default();
    let account_owner_account_info = AccountInfo::new(
        &account_owner,
        false,
        false,
        &mut account_owner_lamports,
        &mut account_owner_data,
        &account_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            account_info.clone(),
            mint_account_info.clone(),
            account_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // ========== 第四部分：执行 Approve ==========
    
    let approve_amount = 500u64;
    
    // 创建账户所有者账户（必须是 signer）
    let mut owner_lamports = 0u64;
    let mut owner_data = vec![];
    let owner_account_id = Pubkey::default();
    let owner_account_info = AccountInfo::new(
        &account_owner,
        true, // is_signer = true
        false,
        &mut owner_lamports,
        &mut owner_data,
        &owner_account_id,
        false,
    );
    
    // 创建委托账户（不需要是 signer）
    let mut delegate_lamports = 0u64;
    let mut delegate_data = vec![];
    let delegate_account_id = Pubkey::default();
    let delegate_account_info = AccountInfo::new(
        &delegate,
        false,
        false,
        &mut delegate_lamports,
        &mut delegate_data,
        &delegate_account_id,
        false,
    );
    
    let accounts = vec![
        account_info.clone(),
        delegate_account_info,
        owner_account_info,
    ];
    
    // 执行 Approve
    Processor::process_approve(&program_id, &accounts, approve_amount).unwrap();
    
    // ========== 第五部分：验证批准结果 ==========
    
    let account_after = Account::unpack(&account_info.data.borrow()).unwrap();
    
    assert_eq!(account_after.delegate, COption::Some(delegate), "delegate 应该被设置");
    assert_eq!(account_after.delegated_amount, approve_amount, "delegated_amount 应该等于批准金额");
    
    println!("✅ 测试通过：Approve 指令测试成功");
}

#[test]
fn test_transfer_with_delegate() {
    // ========== 测试委托转账 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    let source_account_keypair = Pubkey::new_unique();
    let destination_account_keypair = Pubkey::new_unique();
    let source_owner = Pubkey::new_unique();
    let destination_owner = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // 创建并初始化 mint
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::None,
    ).unwrap();
    
    // 初始化源账户和目标账户
    let mut source_account_data = vec![0u8; account_data_len];
    let mut source_account_lamports = account_rent_exempt;
    
    let source_account_info = AccountInfo::new(
        &source_account_keypair,
        false,
        true,
        &mut source_account_lamports,
        &mut source_account_data,
        &program_id,
        false,
    );
    
    let mut destination_account_data = vec![0u8; account_data_len];
    let mut destination_account_lamports = account_rent_exempt;
    
    let destination_account_info = AccountInfo::new(
        &destination_account_keypair,
        false,
        true,
        &mut destination_account_lamports,
        &mut destination_account_data,
        &program_id,
        false,
    );
    
    let mut source_owner_lamports = 0u64;
    let mut source_owner_data = vec![];
    let source_owner_account_id = Pubkey::default();
    let source_owner_account_info = AccountInfo::new(
        &source_owner,
        false,
        false,
        &mut source_owner_lamports,
        &mut source_owner_data,
        &source_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            source_account_info.clone(),
            mint_account_info.clone(),
            source_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    let mut destination_owner_lamports = 0u64;
    let mut destination_owner_data = vec![];
    let destination_owner_account_id = Pubkey::default();
    let destination_owner_account_info = AccountInfo::new(
        &destination_owner,
        false,
        false,
        &mut destination_owner_lamports,
        &mut destination_owner_data,
        &destination_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            destination_account_info.clone(),
            mint_account_info.clone(),
            destination_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // 先给源账户充值
    let mint_amount = 1000u64;
    
    let mut mint_authority_lamports = 0u64;
    let mut mint_authority_data = vec![];
    let mint_authority_account_id = Pubkey::default();
    let mint_authority_account_info = AccountInfo::new(
        &mint_authority,
        true,
        false,
        &mut mint_authority_lamports,
        &mut mint_authority_data,
        &mint_authority_account_id,
        false,
    );
    
    let mint_to_accounts = vec![
        mint_account_info.clone(),
        source_account_info.clone(),
        mint_authority_account_info,
    ];
    
    Processor::process_mint_to(&program_id, &mint_to_accounts, mint_amount).unwrap();
    
    // 批准委托
    let approve_amount = 300u64;
    
    let mut owner_lamports = 0u64;
    let mut owner_data = vec![];
    let owner_account_id = Pubkey::default();
    let owner_account_info = AccountInfo::new(
        &source_owner,
        true,
        false,
        &mut owner_lamports,
        &mut owner_data,
        &owner_account_id,
        false,
    );
    
    let mut delegate_lamports = 0u64;
    let mut delegate_data = vec![];
    let delegate_account_id = Pubkey::default();
    let delegate_account_info = AccountInfo::new(
        &delegate,
        false,
        false,
        &mut delegate_lamports,
        &mut delegate_data,
        &delegate_account_id,
        false,
    );
    
    let approve_accounts = vec![
        source_account_info.clone(),
        delegate_account_info,
        owner_account_info,
    ];
    
    Processor::process_approve(&program_id, &approve_accounts, approve_amount).unwrap();
    
    // 使用委托进行转账
    let transfer_amount = 200u64;
    
    let mut delegate_lamports2 = 0u64;
    let mut delegate_data2 = vec![];
    let delegate_account_id2 = Pubkey::default();
    let delegate_account_info2 = AccountInfo::new(
        &delegate,
        true, // delegate must be signer
        false,
        &mut delegate_lamports2,
        &mut delegate_data2,
        &delegate_account_id2,
        false,
    );
    
    let transfer_accounts = vec![
        source_account_info.clone(),
        destination_account_info.clone(),
        delegate_account_info2,
    ];
    
    Processor::process_transfer(&program_id, &transfer_accounts, transfer_amount).unwrap();
    
    // 验证转账结果
    let source_account_after = Account::unpack(&source_account_info.data.borrow()).unwrap();
    let destination_account_after = Account::unpack(&destination_account_info.data.borrow()).unwrap();
    
    assert_eq!(source_account_after.amount, 800, "源账户余额应该减少 200，剩余 800");
    assert_eq!(destination_account_after.amount, 200, "目标账户余额应该增加 200");
    assert_eq!(source_account_after.delegated_amount, 100, "委托金额应该减少 200，剩余 100");
    assert_eq!(source_account_after.delegate, COption::Some(delegate), "delegate 应该仍然存在");
    
    println!("✅ 测试通过：委托转账测试成功");
}

#[test]
fn test_close_account() {
    // ========== 测试关闭账户 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    let account_keypair = Pubkey::new_unique();
    let account_owner = Pubkey::new_unique();
    let destination = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // 创建并初始化 mint
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::None,
    ).unwrap();
    
    // 初始化账户
    let mut account_data = vec![0u8; account_data_len];
    let mut account_lamports = account_rent_exempt;
    
    let account_info = AccountInfo::new(
        &account_keypair,
        false,
        true,
        &mut account_lamports,
        &mut account_data,
        &program_id,
        false,
    );
    
    let mut account_owner_lamports = 0u64;
    let mut account_owner_data = vec![];
    let account_owner_account_id = Pubkey::default();
    let account_owner_account_info = AccountInfo::new(
        &account_owner,
        false,
        false,
        &mut account_owner_lamports,
        &mut account_owner_data,
        &account_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            account_info.clone(),
            mint_account_info.clone(),
            account_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // 记录关闭前的 lamports
    let account_lamports_before = account_info.lamports();
    
    // 创建目标账户
    let mut destination_lamports = 0u64;
    let mut destination_data = vec![];
    let destination_account_id = Pubkey::default();
    let destination_account_info = AccountInfo::new(
        &destination,
        false,
        true,
        &mut destination_lamports,
        &mut destination_data,
        &destination_account_id,
        false,
    );
    
    // 创建账户所有者账户（必须是 signer）
    let mut owner_lamports = 0u64;
    let mut owner_data = vec![];
    let owner_account_id = Pubkey::default();
    let owner_account_info = AccountInfo::new(
        &account_owner,
        true, // is_signer = true
        false,
        &mut owner_lamports,
        &mut owner_data,
        &owner_account_id,
        false,
    );
    
    let close_accounts = vec![
        account_info.clone(),
        destination_account_info.clone(),
        owner_account_info,
    ];
    
    // 执行 CloseAccount
    Processor::process_close_account(&program_id, &close_accounts).unwrap();
    
    // 验证结果
    assert_eq!(account_info.lamports(), 0, "源账户的 lamports 应该为 0");
    assert_eq!(
        destination_account_info.lamports(),
        account_lamports_before,
        "目标账户应该收到所有 lamports"
    );
    
    println!("✅ 测试通过：CloseAccount 指令测试成功");
}

#[test]
fn test_close_account_with_balance_should_fail() {
    // ========== 测试关闭有余额的非原生账户应该失败 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    let account_keypair = Pubkey::new_unique();
    let account_owner = Pubkey::new_unique();
    let destination = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // 创建并初始化 mint
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::None,
    ).unwrap();
    
    // 初始化账户
    let mut account_data = vec![0u8; account_data_len];
    let mut account_lamports = account_rent_exempt;
    
    let account_info = AccountInfo::new(
        &account_keypair,
        false,
        true,
        &mut account_lamports,
        &mut account_data,
        &program_id,
        false,
    );
    
    let mut account_owner_lamports = 0u64;
    let mut account_owner_data = vec![];
    let account_owner_account_id = Pubkey::default();
    let account_owner_account_info = AccountInfo::new(
        &account_owner,
        false,
        false,
        &mut account_owner_lamports,
        &mut account_owner_data,
        &account_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            account_info.clone(),
            mint_account_info.clone(),
            account_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // 给账户充值
    let mint_amount = 1000u64;
    
    let mut mint_authority_lamports = 0u64;
    let mut mint_authority_data = vec![];
    let mint_authority_account_id = Pubkey::default();
    let mint_authority_account_info = AccountInfo::new(
        &mint_authority,
        true,
        false,
        &mut mint_authority_lamports,
        &mut mint_authority_data,
        &mint_authority_account_id,
        false,
    );
    
    let mint_to_accounts = vec![
        mint_account_info.clone(),
        account_info.clone(),
        mint_authority_account_info,
    ];
    
    Processor::process_mint_to(&program_id, &mint_to_accounts, mint_amount).unwrap();
    
    // 尝试关闭有余额的账户（应该失败）
    let mut destination_lamports = 0u64;
    let mut destination_data = vec![];
    let destination_account_id = Pubkey::default();
    let destination_account_info = AccountInfo::new(
        &destination,
        false,
        true,
        &mut destination_lamports,
        &mut destination_data,
        &destination_account_id,
        false,
    );
    
    let mut owner_lamports = 0u64;
    let mut owner_data = vec![];
    let owner_account_id = Pubkey::default();
    let owner_account_info = AccountInfo::new(
        &account_owner,
        true,
        false,
        &mut owner_lamports,
        &mut owner_data,
        &owner_account_id,
        false,
    );
    
    let close_accounts = vec![
        account_info.clone(),
        destination_account_info,
        owner_account_info,
    ];
    
    let result = Processor::process_close_account(&program_id, &close_accounts);
    assert!(
        result.is_err(),
        "关闭有余额的非原生账户应该失败"
    );
    
    println!("✅ 测试通过：关闭有余额的账户应该失败");
}

#[test]
fn test_freeze_account() {
    // ========== 测试冻结账户 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    let account_keypair = Pubkey::new_unique();
    let account_owner = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // 创建并初始化 mint（带 freeze_authority）
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::Some(freeze_authority),
    ).unwrap();
    
    // 初始化账户
    let mut account_data = vec![0u8; account_data_len];
    let mut account_lamports = account_rent_exempt;
    
    let account_info = AccountInfo::new(
        &account_keypair,
        false,
        true,
        &mut account_lamports,
        &mut account_data,
        &program_id,
        false,
    );
    
    let mut account_owner_lamports = 0u64;
    let mut account_owner_data = vec![];
    let account_owner_account_id = Pubkey::default();
    let account_owner_account_info = AccountInfo::new(
        &account_owner,
        false,
        false,
        &mut account_owner_lamports,
        &mut account_owner_data,
        &account_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            account_info.clone(),
            mint_account_info.clone(),
            account_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // 验证账户初始状态
    let account_before = Account::unpack(&account_info.data.borrow()).unwrap();
    assert_eq!(account_before.is_frozen, false, "账户初始状态应该不是冻结的");
    
    // 创建 freeze authority 账户（必须是 signer）
    let mut freeze_authority_lamports = 0u64;
    let mut freeze_authority_data = vec![];
    let freeze_authority_account_id = Pubkey::default();
    let freeze_authority_account_info = AccountInfo::new(
        &freeze_authority,
        true, // is_signer = true
        false,
        &mut freeze_authority_lamports,
        &mut freeze_authority_data,
        &freeze_authority_account_id,
        false,
    );
    
    let freeze_accounts = vec![
        account_info.clone(),
        mint_account_info.clone(),
        freeze_authority_account_info,
    ];
    
    // 执行 FreezeAccount
    Processor::process_freeze_account(&program_id, &freeze_accounts).unwrap();
    
    // 验证账户已被冻结
    let account_after = Account::unpack(&account_info.data.borrow()).unwrap();
    assert_eq!(account_after.is_frozen, true, "账户应该已被冻结");
    
    println!("✅ 测试通过：FreezeAccount 指令测试成功");
}

#[test]
fn test_freeze_account_without_freeze_authority_should_fail() {
    // ========== 测试没有 freeze_authority 的 mint 不能冻结账户 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    let account_keypair = Pubkey::new_unique();
    let account_owner = Pubkey::new_unique();
    let fake_freeze_authority = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // 创建并初始化 mint（不带 freeze_authority）
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::None, // 没有 freeze_authority
    ).unwrap();
    
    // 初始化账户
    let mut account_data = vec![0u8; account_data_len];
    let mut account_lamports = account_rent_exempt;
    
    let account_info = AccountInfo::new(
        &account_keypair,
        false,
        true,
        &mut account_lamports,
        &mut account_data,
        &program_id,
        false,
    );
    
    let mut account_owner_lamports = 0u64;
    let mut account_owner_data = vec![];
    let account_owner_account_id = Pubkey::default();
    let account_owner_account_info = AccountInfo::new(
        &account_owner,
        false,
        false,
        &mut account_owner_lamports,
        &mut account_owner_data,
        &account_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            account_info.clone(),
            mint_account_info.clone(),
            account_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // 尝试冻结账户（应该失败，因为 mint 没有 freeze_authority）
    let mut fake_freeze_authority_lamports = 0u64;
    let mut fake_freeze_authority_data = vec![];
    let fake_freeze_authority_account_id = Pubkey::default();
    let fake_freeze_authority_account_info = AccountInfo::new(
        &fake_freeze_authority,
        true,
        false,
        &mut fake_freeze_authority_lamports,
        &mut fake_freeze_authority_data,
        &fake_freeze_authority_account_id,
        false,
    );
    
    let freeze_accounts = vec![
        account_info.clone(),
        mint_account_info.clone(),
        fake_freeze_authority_account_info,
    ];
    
    let result = Processor::process_freeze_account(&program_id, &freeze_accounts);
    assert!(
        result.is_err(),
        "没有 freeze_authority 的 mint 不能冻结账户"
    );
    
    println!("✅ 测试通过：没有 freeze_authority 的 mint 不能冻结账户");
}

#[test]
fn test_transfer_frozen_account_should_fail() {
    // ========== 测试冻结账户不能转账 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    let source_account_keypair = Pubkey::new_unique();
    let destination_account_keypair = Pubkey::new_unique();
    let source_owner = Pubkey::new_unique();
    let destination_owner = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // 创建并初始化 mint
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::Some(freeze_authority),
    ).unwrap();
    
    // 初始化源账户和目标账户
    let mut source_account_data = vec![0u8; account_data_len];
    let mut source_account_lamports = account_rent_exempt;
    
    let source_account_info = AccountInfo::new(
        &source_account_keypair,
        false,
        true,
        &mut source_account_lamports,
        &mut source_account_data,
        &program_id,
        false,
    );
    
    let mut destination_account_data = vec![0u8; account_data_len];
    let mut destination_account_lamports = account_rent_exempt;
    
    let destination_account_info = AccountInfo::new(
        &destination_account_keypair,
        false,
        true,
        &mut destination_account_lamports,
        &mut destination_account_data,
        &program_id,
        false,
    );
    
    let mut source_owner_lamports = 0u64;
    let mut source_owner_data = vec![];
    let source_owner_account_id = Pubkey::default();
    let source_owner_account_info = AccountInfo::new(
        &source_owner,
        false,
        false,
        &mut source_owner_lamports,
        &mut source_owner_data,
        &source_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            source_account_info.clone(),
            mint_account_info.clone(),
            source_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    let mut destination_owner_lamports = 0u64;
    let mut destination_owner_data = vec![];
    let destination_owner_account_id = Pubkey::default();
    let destination_owner_account_info = AccountInfo::new(
        &destination_owner,
        false,
        false,
        &mut destination_owner_lamports,
        &mut destination_owner_data,
        &destination_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            destination_account_info.clone(),
            mint_account_info.clone(),
            destination_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // 给源账户充值
    let mint_amount = 1000u64;
    
    let mut mint_authority_lamports = 0u64;
    let mut mint_authority_data = vec![];
    let mint_authority_account_id = Pubkey::default();
    let mint_authority_account_info = AccountInfo::new(
        &mint_authority,
        true,
        false,
        &mut mint_authority_lamports,
        &mut mint_authority_data,
        &mint_authority_account_id,
        false,
    );
    
    let mint_to_accounts = vec![
        mint_account_info.clone(),
        source_account_info.clone(),
        mint_authority_account_info,
    ];
    
    Processor::process_mint_to(&program_id, &mint_to_accounts, mint_amount).unwrap();
    
    // 冻结源账户
    let mut freeze_authority_lamports = 0u64;
    let mut freeze_authority_data = vec![];
    let freeze_authority_account_id = Pubkey::default();
    let freeze_authority_account_info = AccountInfo::new(
        &freeze_authority,
        true,
        false,
        &mut freeze_authority_lamports,
        &mut freeze_authority_data,
        &freeze_authority_account_id,
        false,
    );
    
    let freeze_accounts = vec![
        source_account_info.clone(),
        mint_account_info.clone(),
        freeze_authority_account_info,
    ];
    
    Processor::process_freeze_account(&program_id, &freeze_accounts).unwrap();
    
    // 尝试从冻结账户转账（应该失败）
    let transfer_amount = 100u64;
    
    let mut source_owner_lamports2 = 0u64;
    let mut source_owner_data2 = vec![];
    let source_owner_account_id2 = Pubkey::default();
    let source_owner_account_info2 = AccountInfo::new(
        &source_owner,
        true,
        false,
        &mut source_owner_lamports2,
        &mut source_owner_data2,
        &source_owner_account_id2,
        false,
    );
    
    let transfer_accounts = vec![
        source_account_info.clone(),
        destination_account_info.clone(),
        source_owner_account_info2,
    ];
    
    let result = Processor::process_transfer(&program_id, &transfer_accounts, transfer_amount);
    assert!(
        result.is_err(),
        "冻结账户不能转账"
    );
    
    println!("✅ 测试通过：冻结账户不能转账");
}

#[test]
fn test_thaw_account() {
    // ========== 测试解冻账户 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    let account_keypair = Pubkey::new_unique();
    let account_owner = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // 创建并初始化 mint（带 freeze_authority）
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::Some(freeze_authority),
    ).unwrap();
    
    // 初始化账户
    let mut account_data = vec![0u8; account_data_len];
    let mut account_lamports = account_rent_exempt;
    
    let account_info = AccountInfo::new(
        &account_keypair,
        false,
        true,
        &mut account_lamports,
        &mut account_data,
        &program_id,
        false,
    );
    
    let mut account_owner_lamports = 0u64;
    let mut account_owner_data = vec![];
    let account_owner_account_id = Pubkey::default();
    let account_owner_account_info = AccountInfo::new(
        &account_owner,
        false,
        false,
        &mut account_owner_lamports,
        &mut account_owner_data,
        &account_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            account_info.clone(),
            mint_account_info.clone(),
            account_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // 先冻结账户
    let mut freeze_authority_lamports = 0u64;
    let mut freeze_authority_data = vec![];
    let freeze_authority_account_id = Pubkey::default();
    let freeze_authority_account_info = AccountInfo::new(
        &freeze_authority,
        true,
        false,
        &mut freeze_authority_lamports,
        &mut freeze_authority_data,
        &freeze_authority_account_id,
        false,
    );
    
    let freeze_accounts = vec![
        account_info.clone(),
        mint_account_info.clone(),
        freeze_authority_account_info,
    ];
    
    Processor::process_freeze_account(&program_id, &freeze_accounts).unwrap();
    
    // 验证账户已被冻结
    let account_frozen = Account::unpack(&account_info.data.borrow()).unwrap();
    assert_eq!(account_frozen.is_frozen, true, "账户应该已被冻结");
    
    // 解冻账户
    let mut thaw_authority_lamports = 0u64;
    let mut thaw_authority_data = vec![];
    let thaw_authority_account_id = Pubkey::default();
    let thaw_authority_account_info = AccountInfo::new(
        &freeze_authority,
        true,
        false,
        &mut thaw_authority_lamports,
        &mut thaw_authority_data,
        &thaw_authority_account_id,
        false,
    );
    
    let thaw_accounts = vec![
        account_info.clone(),
        mint_account_info.clone(),
        thaw_authority_account_info,
    ];
    
    Processor::process_thaw_account(&program_id, &thaw_accounts).unwrap();
    
    // 验证账户已被解冻
    let account_thawed = Account::unpack(&account_info.data.borrow()).unwrap();
    assert_eq!(account_thawed.is_frozen, false, "账户应该已被解冻");
    
    println!("✅ 测试通过：ThawAccount 指令测试成功");
}

#[test]
fn test_thaw_account_not_frozen_should_fail() {
    // ========== 测试解冻未冻结的账户应该失败 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    let account_keypair = Pubkey::new_unique();
    let account_owner = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // 创建并初始化 mint
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::Some(freeze_authority),
    ).unwrap();
    
    // 初始化账户（未冻结）
    let mut account_data = vec![0u8; account_data_len];
    let mut account_lamports = account_rent_exempt;
    
    let account_info = AccountInfo::new(
        &account_keypair,
        false,
        true,
        &mut account_lamports,
        &mut account_data,
        &program_id,
        false,
    );
    
    let mut account_owner_lamports = 0u64;
    let mut account_owner_data = vec![];
    let account_owner_account_id = Pubkey::default();
    let account_owner_account_info = AccountInfo::new(
        &account_owner,
        false,
        false,
        &mut account_owner_lamports,
        &mut account_owner_data,
        &account_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            account_info.clone(),
            mint_account_info.clone(),
            account_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // 尝试解冻未冻结的账户（应该失败）
    let mut thaw_authority_lamports = 0u64;
    let mut thaw_authority_data = vec![];
    let thaw_authority_account_id = Pubkey::default();
    let thaw_authority_account_info = AccountInfo::new(
        &freeze_authority,
        true,
        false,
        &mut thaw_authority_lamports,
        &mut thaw_authority_data,
        &thaw_authority_account_id,
        false,
    );
    
    let thaw_accounts = vec![
        account_info.clone(),
        mint_account_info.clone(),
        thaw_authority_account_info,
    ];
    
    let result = Processor::process_thaw_account(&program_id, &thaw_accounts);
    assert!(
        result.is_err(),
        "解冻未冻结的账户应该失败"
    );
    
    println!("✅ 测试通过：解冻未冻结的账户应该失败");
}

#[test]
fn test_freeze_thaw_cycle() {
    // ========== 测试冻结-解冻循环 ==========
    
    let program_id = id();
    let mint_keypair = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let freeze_authority = Pubkey::new_unique();
    let decimals = 6u8;
    
    let account_keypair = Pubkey::new_unique();
    let account_owner = Pubkey::new_unique();
    
    let rent = Rent::default();
    let mint_data_len = Mint::LEN;
    let account_data_len = Account::LEN;
    let mint_rent_exempt = rent.minimum_balance(mint_data_len);
    let account_rent_exempt = rent.minimum_balance(account_data_len);
    
    // 创建并初始化 mint
    let mut mint_data = vec![0u8; mint_data_len];
    let mut mint_lamports = mint_rent_exempt;
    
    let mint_account_info = AccountInfo::new(
        &mint_keypair,
        false,
        true,
        &mut mint_lamports,
        &mut mint_data,
        &program_id,
        false,
    );
    
    let rent_sysvar_id = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
    let rent_data = bincode::serialize(&rent).unwrap();
    let mut rent_lamports = 0u64;
    let mut rent_data_mut = rent_data.clone();
    
    let rent_sysvar_info = AccountInfo::new(
        &rent_sysvar_id,
        false,
        false,
        &mut rent_lamports,
        &mut rent_data_mut,
        &rent_sysvar_id,
        false,
    );
    
    Processor::process_initialize_mint(
        &program_id,
        &[mint_account_info.clone(), rent_sysvar_info.clone()],
        decimals,
        mint_authority,
        COption::Some(freeze_authority),
    ).unwrap();
    
    // 初始化账户
    let mut account_data = vec![0u8; account_data_len];
    let mut account_lamports = account_rent_exempt;
    
    let account_info = AccountInfo::new(
        &account_keypair,
        false,
        true,
        &mut account_lamports,
        &mut account_data,
        &program_id,
        false,
    );
    
    let mut account_owner_lamports = 0u64;
    let mut account_owner_data = vec![];
    let account_owner_account_id = Pubkey::default();
    let account_owner_account_info = AccountInfo::new(
        &account_owner,
        false,
        false,
        &mut account_owner_lamports,
        &mut account_owner_data,
        &account_owner_account_id,
        false,
    );
    
    Processor::process_initialize_account(
        &program_id,
        &[
            account_info.clone(),
            mint_account_info.clone(),
            account_owner_account_info,
            rent_sysvar_info.clone(),
        ],
    ).unwrap();
    
    // 创建 freeze authority 账户
    let mut freeze_authority_lamports = 0u64;
    let mut freeze_authority_data = vec![];
    let freeze_authority_account_id = Pubkey::default();
    
    // 冻结
    let freeze_authority_account_info = AccountInfo::new(
        &freeze_authority,
        true,
        false,
        &mut freeze_authority_lamports,
        &mut freeze_authority_data,
        &freeze_authority_account_id,
        false,
    );
    
    let freeze_accounts = vec![
        account_info.clone(),
        mint_account_info.clone(),
        freeze_authority_account_info,
    ];
    
    Processor::process_freeze_account(&program_id, &freeze_accounts).unwrap();
    let account1 = Account::unpack(&account_info.data.borrow()).unwrap();
    assert_eq!(account1.is_frozen, true);
    
    // 解冻
    let mut thaw_authority_lamports = 0u64;
    let mut thaw_authority_data = vec![];
    let thaw_authority_account_info = AccountInfo::new(
        &freeze_authority,
        true,
        false,
        &mut thaw_authority_lamports,
        &mut thaw_authority_data,
        &freeze_authority_account_id,
        false,
    );
    
    let thaw_accounts = vec![
        account_info.clone(),
        mint_account_info.clone(),
        thaw_authority_account_info,
    ];
    
    Processor::process_thaw_account(&program_id, &thaw_accounts).unwrap();
    let account2 = Account::unpack(&account_info.data.borrow()).unwrap();
    assert_eq!(account2.is_frozen, false);
    
    // 再次冻结
    let mut freeze_authority_lamports2 = 0u64;
    let mut freeze_authority_data2 = vec![];
    let freeze_authority_account_info2 = AccountInfo::new(
        &freeze_authority,
        true,
        false,
        &mut freeze_authority_lamports2,
        &mut freeze_authority_data2,
        &freeze_authority_account_id,
        false,
    );
    
    let freeze_accounts2 = vec![
        account_info.clone(),
        mint_account_info.clone(),
        freeze_authority_account_info2,
    ];
    
    Processor::process_freeze_account(&program_id, &freeze_accounts2).unwrap();
    let account3 = Account::unpack(&account_info.data.borrow()).unwrap();
    assert_eq!(account3.is_frozen, true);
    
    println!("✅ 测试通过：冻结-解冻循环测试成功");
}
