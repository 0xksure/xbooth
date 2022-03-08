use borsh::BorshSerialize;
use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::Sysvar,
};
use spl_token::{instruction, state::Account as TokenAccount};

use crate::errors::XBoothError;
use crate::processor;
use crate::state;

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let exchange_booth_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let vault_a = next_account_info(accounts_iter)?;
    let vault_b = next_account_info(accounts_iter)?;
    let mint_a = next_account_info(accounts_iter)?;
    let mint_b = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let rent_account = next_account_info(accounts_iter)?;

    if !payer.is_signer {
        msg!("payer have to be a signer");
        return Err(XBoothError::AccountIsNotSigner.into());
    }

    if !exchange_booth_account.is_writable {
        msg!("exchange booth account needs to be writable");
        return Err(XBoothError::ExchangeBoothNotWritable.into());
    }

    if !vault_a.is_writable {
        msg!("vault a needs to be writable!");
        return Err(XBoothError::AccountIsNotWritable.into());
    }

    if !vault_b.is_writable {
        msg!("vault b needs to be writable!");
        return Err(XBoothError::AccountIsNotWritable.into());
    }

    // * --- Vault A
    // find pda
    let (vault_a_pda, vault_a_bump_seed) =
        processor::utils::get_vault_pda(program_id, exchange_booth_account, payer, mint_a, vault_a)
            .unwrap();

    // * --- Vault B
    // find pda
    let (vault_b_pda, vault_b_bump_seed) =
        processor::utils::get_vault_pda(program_id, exchange_booth_account, payer, mint_b, vault_b)
            .unwrap();

    // * -- Exchange Booth Account
    // get pda
    let (xbooth_pda, xbooth_bump_seed) = processor::utils::get_exchange_booth_pda(
        program_id,
        exchange_booth_account,
        payer,
        mint_a,
        mint_b,
    )
    .unwrap();

    // * Create exchange booth account
    msg!("create exchange booth account");
    invoke_signed(
        &create_account(
            &payer.key,
            &exchange_booth_account.key,
            Rent::get()?.minimum_balance(state::EXCHANGE_BOOTH_ACCOUNT_LEN),
            state::EXCHANGE_BOOTH_ACCOUNT_LEN as u64,
            program_id,
        ),
        &[
            payer.clone(),
            system_program.clone(),
            exchange_booth_account.clone(),
        ],
        &[&[
            b"xbooth",
            payer.key.as_ref(),
            mint_a.key.as_ref(),
            mint_b.key.as_ref(),
            &[xbooth_bump_seed],
        ]],
    )?;

    // Calculate vault rent
    let rent = Rent::get()?.minimum_balance(TokenAccount::LEN);

    // * Create and initialize vault a
    msg!("create Vault A");
    invoke_signed(
        &create_account(
            &payer.key,
            &vault_a_pda,
            rent,
            TokenAccount::LEN as u64,
            &token_program.key,
        ),
        &[payer.clone(), vault_a.clone(), token_program.clone()],
        &[&[
            b"xbooth",
            payer.key.as_ref(),
            mint_a.key.as_ref(),
            exchange_booth_account.key.as_ref(),
            &[vault_a_bump_seed],
        ]],
    )?;

    let ix = spl_token::instruction::initialize_account(
        &token_program.key,
        &vault_a.key,
        &mint_a.key,
        &exchange_booth_account.key,
    )?;
    invoke(
        &ix,
        &[
            vault_a.clone(),
            mint_a.clone(),
            exchange_booth_account.clone(),
            rent_account.clone(),
            token_program.clone(),
        ],
    )?;

    // * Create and initialize vault b
    msg!("create Vault B");
    invoke_signed(
        &create_account(
            &payer.key,
            &vault_b_pda,
            rent,
            TokenAccount::LEN as u64,
            &token_program.key,
        ),
        &[payer.clone(), vault_b.clone(), token_program.clone()],
        &[&[
            b"xbooth",
            payer.key.as_ref(),
            mint_b.key.as_ref(),
            exchange_booth_account.key.as_ref(),
            &[vault_b_bump_seed],
        ]],
    )?;

    invoke(
        &instruction::initialize_account(
            &token_program.key,
            &vault_b.key,
            &mint_b.key,
            &exchange_booth_account.key,
        )?,
        &[
            vault_b.clone(),
            mint_b.clone(),
            exchange_booth_account.clone(),
            rent_account.clone(),
            token_program.clone(),
        ],
    )?;

    // * Allocate data to exchange booth
    let xbooth_info = state::ExchangeBoothAccount {
        admin: *payer.key,
        vault_a: *vault_a.key,
        vault_b: *vault_b.key,
    };
    let exchange_booth_data = &mut *exchange_booth_account.data.borrow_mut();
    xbooth_info.serialize(exchange_booth_data)?;
    Ok(())
}
