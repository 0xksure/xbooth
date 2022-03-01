//#![cfg(feature = "test-bpf")]
use solana_program::{pubkey::Pubkey, system_program};
use solana_program_test::{processor, tokio, ProgramTest};

use solana_sdk::{
    program_pack::Pack, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};
use spl_token::{
    id, instruction,
    state::{Account, Mint},
};

#[tokio::test]
async fn test_create_mint() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::default();
    program_test.add_program("xbooth", program_id, None);
    let auth = solana_sdk::signature::Keypair::new();
    program_test.add_account(
        auth.pubkey(),
        solana_sdk::account::Account {
            lamports: 100_000_000_000,
            data: vec![],
            owner: system_program::id(),
            ..solana_sdk::account::Account::default()
        },
    );
    let (mut bank_client, payer, recent_blockhash) = program_test.start().await;

    // Assemble accounts

    let rent = bank_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);
    let mint_a_account = solana_sdk::signature::Keypair::new();
    let owner = solana_sdk::signature::Keypair::new();

    // create account to hold newly minted tokens
    let token_mint_a_account_ix = solana_program::system_instruction::create_account(
        &payer.pubkey(),
        &mint_a_account.pubkey(),
        mint_rent,
        Mint::LEN as u64,
        &id(),
    );

    // initialize mint
    let token_mint_a_ix = spl_token::instruction::initialize_mint(
        &id(),
        &mint_a_account.pubkey(),
        &owner.pubkey(),
        None,
        9,
    )
    .unwrap();

    // create mint transaction
    let token_mint_a_tx = Transaction::new_signed_with_payer(
        &[token_mint_a_account_ix, token_mint_a_ix],
        Some(&payer.pubkey()),
        &[&payer, &mint_a_account],
        recent_blockhash,
    );

    bank_client
        .process_transaction(token_mint_a_tx)
        .await
        .unwrap();

    let token_account = Keypair::new();
    let account_rent = rent.minimum_balance(Account::LEN);
    let new_token_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &token_account.pubkey(),
        account_rent,
        Account::LEN as u64,
        &id(),
    );

    let initialize_account_ix = instruction::initialize_account(
        &id(),
        &token_account.pubkey(),
        &mint_a_account.pubkey(),
        &owner.pubkey(),
    )
    .unwrap();

    let create_new_token_account_tx = Transaction::new_signed_with_payer(
        &[new_token_account_ix, initialize_account_ix],
        Some(&payer.pubkey()),
        &[&payer, &token_account],
        recent_blockhash,
    );
    bank_client
        .process_transaction(create_new_token_account_tx)
        .await
        .unwrap();

    /// Mint tokens to token account
    let mint_amount: u64 = 10;
    let mint_to_ix = instruction::mint_to(
        &id(),
        &mint_a_account.pubkey(),
        &token_account.pubkey(),
        &owner.pubkey(),
        &[],
        mint_amount.clone(),
    )
    .unwrap();

    let mint_to_tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&payer.pubkey()),
        &[&payer, &owner],
        recent_blockhash,
    );
    bank_client.process_transaction(mint_to_tx).await.unwrap();

    /// Inspect mint state
    let token_mint_account_info = bank_client
        .get_account(mint_a_account.pubkey().clone())
        .await
        .unwrap()
        .expect("could not get account");
    let token_account_data = token_mint_account_info.data;
    println!(
        "account len : {:}, token account len: {:}",
        Mint::LEN,
        &token_account_data.len()
    );
    let mint_data = Mint::unpack(&token_account_data).unwrap();

    let token_account_info = bank_client
        .get_account(token_account.pubkey().clone())
        .await
        .unwrap()
        .expect("could not fetch account information");
    let account_data = Account::unpack(&token_account_info.data).unwrap();
    println!("account data: {:?}", account_data);
    assert_eq!(
        account_data.amount,
        mint_amount.clone(),
        "not correct amount"
    );
}
