//#![cfg(feature = "test-bpf")]
use solana_program::{
    clock::BankId,
    instruction::{self, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
    sysvar::{self, recent_blockhashes},
};
use solana_program_test::*;
use solana_sdk::{
    hash::Hash, program_pack::Pack, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction, transport::TransportError,
};
use spl_token::state::{Account, Mint};
use std::mem;

/// create_and_initialize_account sets up a new account and

async fn mint_amount(
    banks_client: &mut BanksClient,
    recent_blockhash: Hash,
    token_program: &Pubkey,
    account: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Keypair,
    payer: &Keypair,
    amount: f64,
    mint_decimals: u8,
) -> Result<(), ProgramError> {
    let mint_amount = (amount * f64::powf(10., mint_decimals.into())) as u64;
    let mint_ix = spl_token::instruction::mint_to(
        token_program,
        mint,
        account,
        &mint_authority.pubkey(),
        &[],
        mint_amount,
    )
    .unwrap();

    let mint_tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&payer.pubkey()),
        &[payer, mint_authority],
        recent_blockhash,
    );

    banks_client.process_transaction(mint_tx).await.unwrap();
    Ok(())
}

/// create_and_initialize_account_for_mint does two things
/// 1. creates an account with the payer as owner
/// 2. initializes the account for the current mint
async fn create_and_initialize_account_for_mint(
    banks_client: &mut BanksClient,
    recent_blockhash: Hash,
    token_program: &Pubkey,
    token_account: &Keypair,
    mint: &Keypair,
    payer: &Keypair,
) -> Result<(), ProgramError> {
    let rent = banks_client.get_rent().await.unwrap();
    let account_rent = rent.minimum_balance(Account::LEN);
    let create_account_ix = solana_program::system_instruction::create_account(
        &payer.pubkey(),
        &token_account.pubkey(),
        account_rent,
        Account::LEN as u64,
        token_program,
    );

    let initialize_account_ix = spl_token::instruction::initialize_account(
        token_program,
        &token_account.pubkey(),
        &mint.pubkey(),
        &payer.pubkey(),
    )
    .unwrap();

    let initialize_account_tx = Transaction::new_signed_with_payer(
        &[create_account_ix, initialize_account_ix],
        Some(&payer.pubkey()),
        &[payer, token_account],
        recent_blockhash,
    );

    banks_client
        .process_transaction(initialize_account_tx)
        .await
        .unwrap();

    Ok(())
}

async fn create_and_initialize_mint(
    banks_client: &mut BanksClient,
    recent_blockhash: Hash,
    payer: &Keypair,
    mint_authority: &Keypair,
    mint_account: &Keypair,
    token_program: &Pubkey,
    decimals: &u8,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);
    // create account to hold newly minted tokens
    let token_mint_a_account_ix = solana_program::system_instruction::create_account(
        &payer.pubkey(),
        &mint_account.pubkey(),
        mint_rent,
        Mint::LEN as u64,
        token_program,
    );

    // initialize mint
    let token_mint_a_ix = spl_token::instruction::initialize_mint(
        token_program,
        &mint_account.pubkey(),
        &mint_authority.pubkey(),
        None,
        *decimals,
    )
    .unwrap();

    // create mint transaction
    let token_mint_a_tx = Transaction::new_signed_with_payer(
        &[token_mint_a_account_ix, token_mint_a_ix],
        Some(&payer.pubkey()),
        &[payer, mint_account],
        recent_blockhash,
    );

    banks_client
        .process_transaction(token_mint_a_tx)
        .await
        .unwrap();
    Ok(())
}

fn create_exchange_booth_pda(
    program_id: &Pubkey,
    authority: &Keypair,
    mint_a: &Keypair,
    mint_b: &Keypair,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"xbooth",
            authority.pubkey().as_ref(),
            mint_a.pubkey().as_ref(),
            mint_b.pubkey().as_ref(),
        ],
        program_id,
    )
}

fn create_vault_pda(
    program_id: &Pubkey,
    authority: &Keypair,
    mint_account: &Keypair,
    owner: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"xbooth",
            authority.pubkey().as_ref(),
            mint_account.pubkey().as_ref(),
            owner.as_ref(),
        ],
        program_id,
    )
}

#[tokio::test]
async fn test_deposit_into_vault() {
    // Initialize program test
    let program_id = Pubkey::new_unique();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();
    let auth = Keypair::new();

    let mut program_test = ProgramTest::new("xbooth", program_id, None);
    program_test.add_account(
        auth.pubkey(),
        solana_sdk::account::Account {
            lamports: 100_000_000_000,
            data: vec![],
            owner: system_program::id(),
            ..solana_sdk::account::Account::default()
        },
    );
    let (mut banks_client, authority, recent_blockhash) = program_test.start().await;

    // Create and initialize mints
    let mint_a_decimals = 9;
    let mint_b_decimals = mint_a_decimals.clone();
    create_and_initialize_mint(
        &mut banks_client,
        recent_blockhash,
        &auth,
        &auth,
        &mint_a,
        &spl_token::id(),
        &mint_a_decimals,
    )
    .await
    .unwrap();

    create_and_initialize_mint(
        &mut banks_client,
        recent_blockhash,
        &auth,
        &auth,
        &mint_b,
        &spl_token::id(),
        &mint_b_decimals,
    )
    .await
    .unwrap();
    // ------------------------------

    // * Create Accounts to hold A token and B token
    let system_program_account = instruction::AccountMeta {
        pubkey: system_program::id(),
        is_signer: false,
        is_writable: false,
    };

    let (xbooth_pda, _xbooth_bump_seed) =
        create_exchange_booth_pda(&program_id, &authority, &mint_a, &mint_b);

    let exchange_booth_account = instruction::AccountMeta {
        pubkey: xbooth_pda,
        is_signer: false,
        is_writable: true,
    };

    let authority_account = instruction::AccountMeta {
        pubkey: authority.pubkey(),
        is_signer: true,
        is_writable: false,
    };

    // * Create token account for A tokens
    let token_account = Keypair::new();
    create_and_initialize_account_for_mint(
        &mut banks_client,
        recent_blockhash,
        &spl_token::id(),
        &token_account,
        &mint_a,
        &authority,
    )
    .await
    .unwrap();

    // Mint A tokens into token A account
    let initial_token_a_amount = 100.0;
    mint_amount(
        &mut banks_client,
        recent_blockhash,
        &spl_token::id(),
        &token_account.pubkey(),
        &mint_a.pubkey(),
        &auth,
        &authority,
        initial_token_a_amount.clone(),
        mint_a_decimals,
    )
    .await
    .unwrap();

    // * Create token Account for B tokens
    let token_account_b = Keypair::new();
    create_and_initialize_account_for_mint(
        &mut banks_client,
        recent_blockhash,
        &spl_token::id(),
        &token_account_b,
        &mint_b,
        &authority,
    )
    .await
    .unwrap();

    let initial_token_b_amount = 100.0;
    mint_amount(
        &mut banks_client,
        recent_blockhash,
        &spl_token::id(),
        &token_account_b.pubkey(),
        &mint_b.pubkey(),
        &auth,
        &authority,
        initial_token_b_amount.clone(),
        mint_b_decimals,
    )
    .await
    .unwrap();

    // * Create accounts needed for the instructions
    let token_account_a_meta = instruction::AccountMeta {
        pubkey: token_account.pubkey(),
        is_signer: false,
        is_writable: true,
    };

    let token_account_b_meta = instruction::AccountMeta {
        pubkey: token_account_b.pubkey(),
        is_signer: false,
        is_writable: true,
    };
    // initialize authority account to hold same token as in vault

    let (vault_a_pda, _vault_a_bump) =
        create_vault_pda(&program_id, &authority, &mint_a, &xbooth_pda);

    let vault_a_account = instruction::AccountMeta {
        pubkey: vault_a_pda,
        is_signer: false,
        is_writable: true,
    };

    let (vault_b_pda, _vault_b_bump) =
        create_vault_pda(&program_id, &authority, &mint_b, &xbooth_pda);

    let vault_b_account = instruction::AccountMeta {
        pubkey: vault_b_pda,
        is_signer: false,
        is_writable: true,
    };

    let mint_a_account = instruction::AccountMeta {
        pubkey: mint_a.pubkey(),
        is_signer: false,
        is_writable: false,
    };

    let mint_b_account = instruction::AccountMeta {
        pubkey: mint_b.pubkey(),
        is_signer: false,
        is_writable: false,
    };

    let token_program_account = instruction::AccountMeta {
        pubkey: spl_token::id(),
        is_signer: false,
        is_writable: false,
    };

    let rent_account = instruction::AccountMeta {
        pubkey: solana_program::sysvar::rent::id(),
        is_signer: false,
        is_writable: false,
    };

    // * ---------- Instructions and Transactions ---------------

    // * 1. Initialize exchange booth, instruction
    let initialize_accounts = vec![
        exchange_booth_account.clone(),
        authority_account.clone(),
        system_program_account.clone(),
        vault_a_account.clone(),
        vault_b_account.clone(),
        mint_a_account.clone(),
        mint_b_account.clone(),
        token_program_account.clone(),
        rent_account.clone(),
    ];

    let initialize_exchange_booth_data = vec![0; mem::size_of::<u8>()];
    let intiialize_ix = instruction::Instruction {
        program_id: program_id,
        accounts: initialize_accounts,
        data: initialize_exchange_booth_data,
    };

    // * Deposit tokens into vault A, instruction
    let deposit_accounts = vec![
        exchange_booth_account.clone(),
        authority_account.clone(),
        token_account_a_meta.clone(),
        vault_a_account.clone(),
        mint_a_account.clone(),
        mint_b_account.clone(),
        token_program_account.clone(),
    ];

    let deposit_amount: f64 = 50.0;
    let deposit_instruction: Vec<u8> = vec![1; mem::size_of::<u8>()];
    let transfer_amount = deposit_amount.to_le_bytes().to_vec();
    let deposit_input_data = [&deposit_instruction[..], &transfer_amount[..]].concat();
    let deposit_ix = instruction::Instruction {
        program_id,
        accounts: deposit_accounts.clone(),
        data: deposit_input_data.clone(),
    };

    // * Process transaction
    let tx = Transaction::new_signed_with_payer(
        &[intiialize_ix, deposit_ix],
        Some(&authority.pubkey()),
        &[&authority],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // * TEST: the amount is transferred
    let token_account_info = banks_client
        .get_account(token_account.pubkey().clone())
        .await
        .unwrap()
        .expect("could not fetch account information");
    let account_data = Account::unpack(&token_account_info.data).unwrap();
    println!("token account data: {:?}", account_data);

    // * Deposit tokens into vault B, instruction
    let deposit_b_accounts = vec![
        exchange_booth_account.clone(),
        authority_account.clone(),
        token_account_b_meta.clone(),
        vault_b_account.clone(),
        mint_a_account.clone(),
        mint_b_account.clone(),
        token_program_account.clone(),
    ];

    let deposit_b_account_ix = instruction::Instruction {
        program_id,
        accounts: deposit_b_accounts.clone(),
        data: deposit_input_data.clone(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[deposit_b_account_ix],
        Some(&authority.pubkey()),
        &[&authority],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // // * 2. Withdraw funds from vault A
    // let withdraw_ix_accounts = deposit_accounts.clone();
    // let withdraw_amount: f64 = 3.0;
    // let withdraw_instruction: Vec<u8> = vec![2; mem::size_of::<u8>()];
    // let withdraw_amount_ba = withdraw_amount.to_le_bytes().to_vec();
    // let withdraw_instruction_data = [&withdraw_instruction[..], &withdraw_amount_ba[..]].concat();

    // let withdraw_ix = instruction::Instruction {
    //     program_id,
    //     accounts: withdraw_ix_accounts,
    //     data: withdraw_instruction_data,
    // };

    // let withdraw_tx = Transaction::new_signed_with_payer(
    //     &[withdraw_ix],
    //     Some(&authority.pubkey()),
    //     &[&authority, &token_account],
    //     recent_blockhash,
    // );
    // banks_client.process_transaction(withdraw_tx).await.unwrap();

    // // sanity check withdraw
    // let token_account_info = banks_client
    //     .get_account(token_account.pubkey().clone())
    //     .await
    //     .unwrap()
    //     .expect("could not fetch account information");
    // let account_data = Account::unpack(&token_account_info.data).unwrap();
    // let expected_account_amount = ((initial_token_a_amount - deposit_amount + withdraw_amount)
    //     * f64::powf(10., mint_a_decimals.into())) as u64;
    // assert_eq!(
    //     account_data.amount, expected_account_amount,
    //     "token amount {} ",
    //     account_data.amount,
    // );

    // // * 3. Exchange A tokens with B tokens
    // let exchange_tokens_ix_accounts = vec![
    //     exchange_booth_account.clone(),
    //     authority_account.clone(),
    //     // receiving account
    //     token_account_a_meta.clone(),
    //     // from account
    //     token_account_b_meta.clone(),
    //     vault_a_account.clone(),
    //     vault_b_account.clone(),
    //     mint_a_account.clone(),
    //     mint_b_account.clone(),
    //     token_program_account.clone(),
    // ];

    // let exchange_token_instruction: Vec<u8> = vec![3; mem::size_of::<u8>()];
    // let token_a_amount: f64 = 10.0;
    // let exhange_token_amount_instruction: Vec<u8> = token_a_amount.to_le_bytes().to_vec();
    // let exhange_token_instruction_data = [
    //     &exchange_token_instruction[..],
    //     &exhange_token_amount_instruction[..],
    // ]
    // .concat();

    // let exhange_token_ix = instruction::Instruction {
    //     program_id,
    //     accounts: exchange_tokens_ix_accounts,
    //     data: exhange_token_instruction_data,
    // };

    // let exhange_token_tx = Transaction::new_signed_with_payer(
    //     &[exhange_token_ix],
    //     Some(&authority.pubkey()),
    //     &[&authority, &token_account, &token_account_b],
    //     recent_blockhash,
    // );

    // banks_client
    //     .process_transaction(exhange_token_tx)
    //     .await
    //     .unwrap();
}
