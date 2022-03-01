use solana_program::{pubkey::Pubkey, system_program};
use solana_program_test::ProgramTest;
use solana_sdk::{system_instruction, signer::Signer,transaction::Transaction};
use spl_token;

#[tokio::test]
async fn test_initialize_exchange_booth() {
    let program_id = Pubkey::new_unique();
    let program_test = ProgramTest::default();
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
    let (mut bank_client,payer,recent_blockhash) = program_test.start().await();

    // Assemble accounts 
    let token_program_a = PubKey::new();
    let token_account_a = PubKey::new();
    let token_mint_a = PubKey::new();
    let token_authority_a = PubKey::new();
    let vault_a_account_ix = spl_token::instruction::initialize_account(
        &token_program_a,
         &token_account_a, 
         &token_mint_a,
         &auth.pubkey())?;

    let vault_a_account_tx = Transaction::new_signed_with_payer(
        &[
            vault_a_account_ix
        ], Some(&payer), 
        &[&payer,&mint], recent_blockhash);


    let vault_a_mint = spl_token::instruction::initialize_mint(
        &token_program_a,
        token_mint_a.as_ref(),
         token_authority_a.as_ref(), 
        None, 
        9_u8)?;

    let vault_a_mint_tx = Transaction::new_signed_with_payer(
        &[
            vault_a_mint
        ], Some(&payer), 
        &[&payer,&mint], recent_blockhash);
    
    bank_client.process_transactions(vec![vault_a_tx,vault_a_mint_tx]).await.unwrap();    

}
