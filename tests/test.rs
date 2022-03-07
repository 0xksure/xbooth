#![cfg(feature = "test-bpf")]
use solana_program::{instruction, pubkey::Pubkey, system_program, sysvar};
use solana_program_test::*;
use solana_sdk::{
    hash::Hash, program_pack::Pack, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction, transport::TransportError,
};
use spl_token::state::{Account, Mint};
use std::mem;

/// create_and_initialize_account sets up a new account and
/// initializes it to hold tokens from a specific mint
// async fn create_and_initialize_account(
//     bank_client: &mut BanksClient,
//     recent_blockhash: Hash,
//     auth: &Keypair,
//     owner: &Keypair,
//     token_account: &Keypair,
//     token_program: &Pubkey,
//     mint_account: &Keypair,
// ) -> Result<(), TransportError> {
//     let rent = bank_client.get_rent().await.unwrap();
//     let account_rent = rent.minimum_balance(Account::LEN);
//     let new_token_account_ix = system_instruction::create_account(
//         &auth.pubkey(),
//         &token_account.pubkey(),
//         account_rent,
//         Account::LEN as u64,
//         token_program,
//     );

//     let initialize_account_ix = instruction::initialize_account(
//         token_program,
//         &token_account.pubkey(),
//         &mint_account.pubkey(),
//         &owner.pubkey(),
//     )
//     .unwrap();

//     let create_new_token_account_tx = Transaction::new_signed_with_auth(
//         &[new_token_account_ix, initialize_account_ix],
//         Some(&auth.pubkey()),
//         &[auth, token_account],
//         recent_blockhash,
//     );
//     bank_client
//         .process_transaction(create_new_token_account_tx)
//         .await?;
//     Ok(())
// }

async fn create_and_initialize_mint(
    bank_client: &mut BanksClient,
    recent_blockhash: Hash,
    auth: &Keypair,
    owner: &Keypair,
    mint_account: &Keypair,
    token_program: &Pubkey,
) -> Result<(), TransportError> {
    let rent = bank_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);
    // create account to hold newly minted tokens
    let token_mint_a_account_ix = solana_program::system_instruction::create_account(
        &auth.pubkey(),
        &mint_account.pubkey(),
        mint_rent,
        Mint::LEN as u64,
        token_program,
    );

    // initialize mint
    let token_mint_a_ix = spl_token::instruction::initialize_mint(
        token_program,
        &mint_account.pubkey(),
        &owner.pubkey(),
        None,
        9,
    )
    .unwrap();

    // create mint transaction
    let token_mint_a_tx = Transaction::new_signed_with_payer(
        &[token_mint_a_account_ix, token_mint_a_ix],
        Some(&auth.pubkey()),
        &[auth, mint_account],
        recent_blockhash,
    );

    bank_client
        .process_transaction(token_mint_a_tx)
        .await
        .unwrap();
    Ok(())
}

#[tokio::test]
async fn initialize_exchange_booth() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new("xbooth", program_id, None);

    let auth = Keypair::new();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();

    program_test.add_account(
        auth.pubkey(),
        solana_sdk::account::Account {
            lamports: 100_000_000_000,
            data: vec![],
            owner: system_program::id(),
            ..solana_sdk::account::Account::default()
        },
    );

    let (mut banks_client, auth, recent_blockhash) = program_test.start().await;

    let (xbooth_pda, xbooth_bump_seed) = Pubkey::find_program_address(
        &[
            b"xbooth",
            auth.pubkey().as_ref(),
            mint_a.pubkey().as_ref(),
            mint_b.pubkey().as_ref(),
        ],
        &program_id,
    );

    let exchange_booth_account = instruction::AccountMeta {
        pubkey: xbooth_pda,
        is_writable: true,
        is_signer: false,
    };

    let auth_account = instruction::AccountMeta {
        pubkey: auth.pubkey(),
        is_writable: false,
        is_signer: true,
    };

    let system_program_account = instruction::AccountMeta {
        pubkey: system_program::id(),
        is_signer: false,
        is_writable: false,
    };

    // * find pda for vault A
    let (vault_a, vault_a_bump_seed) = Pubkey::find_program_address(
        &[
            b"xbooth",
            auth.pubkey().as_ref(),
            mint_a.pubkey().as_ref(),
            xbooth_pda.as_ref(),
        ],
        &program_id,
    );

    let mint_a_account = instruction::AccountMeta {
        pubkey: mint_a.pubkey(),
        is_signer: false,
        is_writable: false,
    };

    // create mint a
    create_and_initialize_mint(
        &mut banks_client,
        recent_blockhash,
        &auth,
        &auth,
        &mint_a,
        &spl_token::id(),
    )
    .await
    .unwrap();

    // create mint b
    create_and_initialize_mint(
        &mut banks_client,
        recent_blockhash,
        &auth,
        &auth,
        &mint_b,
        &spl_token::id(),
    )
    .await
    .unwrap();

    let vault_a_account = instruction::AccountMeta {
        pubkey: vault_a,
        is_signer: false,
        is_writable: true,
    };

    // * find pda for vault B
    let (vault_b, vault_b_bump_seed) = Pubkey::find_program_address(
        &[
            b"xbooth",
            auth.pubkey().as_ref(),
            mint_b.pubkey().as_ref(),
            xbooth_pda.as_ref(),
        ],
        &program_id,
    );

    let mint_b_account = instruction::AccountMeta {
        pubkey: mint_b.pubkey(),
        is_signer: false,
        is_writable: false,
    };

    let vault_b_account = instruction::AccountMeta {
        pubkey: vault_b,
        is_signer: false,
        is_writable: true,
    };

    // * token program account
    let token_program_account = instruction::AccountMeta {
        pubkey: spl_token::id(),
        is_signer: false,
        is_writable: false,
    };

    let rent_account = instruction::AccountMeta {
        pubkey: sysvar::rent::id(),
        is_signer: false,
        is_writable: false,
    };

    let accounts = vec![
        exchange_booth_account.clone(),
        auth_account.clone(),
        system_program_account.clone(),
        vault_a_account.clone(),
        vault_b_account.clone(),
        mint_a_account.clone(),
        mint_b_account.clone(),
        token_program_account.clone(),
        rent_account.clone(),
    ];

    let initialize_exchange_booth_data = vec![0; mem::size_of::<u8>()];
    let initialize_exchange_booth_ix = instruction::Instruction {
        program_id: program_id,
        accounts: accounts,
        data: initialize_exchange_booth_data,
    };

    // * create transaction
    let tx = Transaction::new_signed_with_payer(
        &[initialize_exchange_booth_ix],
        Some(&auth.pubkey()),
        &[&auth],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await.unwrap();
}
