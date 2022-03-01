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
async fn test_initialize_exchange_booth() {
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

    // let mint_to_ix = instruction::mint_to(
    //     &id(),
    //     &token_mint_a_account.pubkey(),
    //     &token_account.pubkey(),
    //     &owner.pubkey(),
    //     &[],
    //     10,
    // )
    // .unwrap();

    // let mint_to_tx = Transaction::new_signed_with_payer(
    //     &[mint_to_ix],
    //     Some(&payer.pubkey()),
    //     &[&payer],
    //     recent_blockhash,
    // );
    // bank_client.process_transaction(mint_to_tx).await.unwrap();

    // let token_mint_account_info = bank_client
    //     .get_account(token_mint_a_account.pubkey().clone())
    //     .await
    //     .unwrap()
    //     .expect("could not get account");
    // let token_account_data = token_mint_account_info.data;
    // println!(
    //     "account len : {:}, token account len: {:}",
    //     Mint::LEN,
    //     &token_account_data.len()
    // );
    // let mint_data = Mint::unpack(&token_account_data).unwrap();

    //panic!("token data: {:?}", mint_data);
    // let vault_a_account_ix = spl_token::instruction::initialize_account(
    //     &spl_token::id(),
    //     &token_account_a,
    //     &token_mint_a.pubkey(),
    //     &auth.pubkey(),
    // )
    // .expect("failed to create vault account instruction");

    // let vault_a_account_tx = Transaction::new_signed_with_payer(
    //     &[vault_a_account_ix],
    //     Some(&payer.pubkey()),
    //     &[&payer, &token_mint_a],
    //     recent_blockhash,
    // );

    // let vault_a_mint_ix = spl_token::instruction::initialize_mint(
    //     &spl_token::id(),
    //     &token_mint_a.pubkey(),
    //     &token_authority_a,
    //     None,
    //     9_u8,
    // )
    // .expect("failed to create vault a mint instruction");

    // let vault_a_mint_tx = Transaction::new_signed_with_payer(
    //     &[vault_a_mint_ix],
    //     Some(&payer.pubkey()),
    //     &[&payer, &token_mint_a],
    //     recent_blockhash,
    // );

    // bank_client
    //     .process_transactions(vec![vault_a_account_tx])
    //     .await
    //     .expect("failed to process vault a account creation transaction");
}
