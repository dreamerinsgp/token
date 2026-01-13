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
        processor::Processor,
        state::Mint,
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
