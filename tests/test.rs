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
    amount: u64,
) -> Result<(), ProgramError> {
    let mint_ix = spl_token::instruction::mint_to(
        token_program,
        mint,
        account,
        &mint_authority.pubkey(),
        &[],
        amount,
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
        9,
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

// #[tokio::test]
// async fn initialize_exchange_booth() {
//     let program_id = Pubkey::new_unique();

//     let auth = Keypair::new();
//     let mint_a = Keypair::new();
//     let mint_b = Keypair::new();

//     let mut program_test = ProgramTest::new("xbooth", program_id, None);
//     program_test.add_account(
//         auth.pubkey(),
//         solana_sdk::account::Account {
//             lamports: 100_000_000_000,
//             data: vec![],
//             owner: system_program::id(),
//             ..solana_sdk::account::Account::default()
//         },
//     );

//     let (mut banks_client, auth, recent_blockhash) = program_test.start().await;

//     create_and_initialize_mint(
//         &mut banks_client,
//         recent_blockhash,
//         &auth,
//         &auth,
//         &mint_a,
//         &spl_token::id(),
//     )
//     .await
//     .unwrap();

//     // create mint b
//     create_and_initialize_mint(
//         &mut banks_client,
//         recent_blockhash,
//         &auth,
//         &auth,
//         &mint_b,
//         &spl_token::id(),
//     )
//     .await
//     .unwrap();

//     let (xbooth_pda, xbooth_bump_seed) =
//         create_exchange_booth_pda(&program_id, &auth, &mint_a, &mint_b);

//     let exchange_booth_account = instruction::AccountMeta {
//         pubkey: xbooth_pda,
//         is_writable: true,
//         is_signer: false,
//     };

//     let auth_account = instruction::AccountMeta {
//         pubkey: auth.pubkey(),
//         is_writable: false,
//         is_signer: true,
//     };

//     let system_program_account = instruction::AccountMeta {
//         pubkey: system_program::id(),
//         is_signer: false,
//         is_writable: false,
//     };

//     // * find pda for vault A
//     let (vault_a, vault_a_bump_seed) = create_vault_pda(&program_id, &auth, &mint_a, &xbooth_pda);

//     let mint_a_account = instruction::AccountMeta {
//         pubkey: mint_a.pubkey(),
//         is_signer: false,
//         is_writable: false,
//     };

//     let vault_a_account = instruction::AccountMeta {
//         pubkey: vault_a,
//         is_signer: false,
//         is_writable: true,
//     };

//     // * find pda for vault B
//     let (vault_b, vault_b_bump_seed) = create_vault_pda(&program_id, &auth, &mint_b, &xbooth_pda);

//     let mint_b_account = instruction::AccountMeta {
//         pubkey: mint_b.pubkey(),
//         is_signer: false,
//         is_writable: false,
//     };

//     let vault_b_account = instruction::AccountMeta {
//         pubkey: vault_b,
//         is_signer: false,
//         is_writable: true,
//     };

//     // * token program account
//     let token_program_account = instruction::AccountMeta {
//         pubkey: spl_token::id(),
//         is_signer: false,
//         is_writable: false,
//     };

//     let rent_account = instruction::AccountMeta {
//         pubkey: sysvar::rent::id(),
//         is_signer: false,
//         is_writable: false,
//     };

//     let accounts = vec![
//         exchange_booth_account.clone(),
//         auth_account.clone(),
//         system_program_account.clone(),
//         vault_a_account.clone(),
//         vault_b_account.clone(),
//         mint_a_account.clone(),
//         mint_b_account.clone(),
//         token_program_account.clone(),
//         rent_account.clone(),
//     ];

//     let initialize_exchange_booth_data = vec![0; mem::size_of::<u8>()];
//     let initialize_exchange_booth_ix = instruction::Instruction {
//         program_id: program_id,
//         accounts: accounts,
//         data: initialize_exchange_booth_data,
//     };

//     // * create transaction
//     let tx = Transaction::new_signed_with_payer(
//         &[initialize_exchange_booth_ix],
//         Some(&auth.pubkey()),
//         &[&auth],
//         recent_blockhash,
//     );

//     banks_client.process_transaction(tx).await.unwrap();
// }

#[tokio::test]
async fn test_deposit_into_vault() {
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

    // create and initialize mints
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

    create_and_initialize_mint(
        &mut banks_client,
        recent_blockhash,
        &authority,
        &authority,
        &mint_b,
        &spl_token::id(),
    )
    .await
    .unwrap();

    // * Prepare Accounts
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

    // mint some tokens
    mint_amount(
        &mut banks_client,
        recent_blockhash,
        &spl_token::id(),
        &token_account.pubkey(),
        &mint_a.pubkey(),
        &auth,
        &authority,
        100,
    )
    .await
    .unwrap();

    let token_account_meta = instruction::AccountMeta {
        pubkey: token_account.pubkey(),
        is_signer: true,
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

    // * compile accounts

    // * initialize exchange booth instruction
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

    // * deposit amount instruction
    let deposit_accounts = vec![
        exchange_booth_account.clone(),
        authority_account.clone(),
        token_account_meta.clone(),
        vault_a_account.clone(),
        mint_a_account.clone(),
        mint_b_account.clone(),
        token_program_account.clone(),
    ];

    let amount: u64 = 10;
    let deposit_instruction: Vec<u8> = vec![1; mem::size_of::<u8>()];
    let transfer_amount = amount.to_le_bytes().to_vec();
    let deposit_input_data = [&deposit_instruction[..], &transfer_amount[..]].concat();
    let deposit_ix = instruction::Instruction {
        program_id,
        accounts: deposit_accounts,
        data: deposit_input_data,
    };

    // * process transaction
    let tx = Transaction::new_signed_with_payer(
        &[intiialize_ix, deposit_ix],
        Some(&authority.pubkey()),
        &[&authority, &token_account],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // * check that the amount is transferred
    let token_account_info = banks_client
        .get_account(token_account.pubkey().clone())
        .await
        .unwrap()
        .expect("could not fetch account information");
    let account_data = Account::unpack(&token_account_info.data).unwrap();
    println!("token account data: {:?}", account_data);
}
