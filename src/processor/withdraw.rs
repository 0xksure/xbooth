use solana_program::program_pack::Pack;
use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
};

use crate::errors::XBoothError;
use crate::processor::utils;

/// process will withdraw amount from an account
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], amount: f64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let exchange_booth_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let vault_account = next_account_info(accounts_iter)?;
    let mint_a = next_account_info(accounts_iter)?;
    let mint_b = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // * Checks checks checks
    // check permissions
    if !authority_account.is_signer {
        msg!("authority is not a signer!");
        return Err(XBoothError::AccountIsNotSigner.into());
    }

    if !vault_account.is_writable {
        msg!("vault needs to be writable");
        return Err(XBoothError::AccountIsNotWritable.into());
    }

    if !token_account.is_signer {
        msg!("token account needs to be signer");
        return Err(XBoothError::AccountIsNotSigner.into());
    }

    if !token_account.is_writable {
        msg!("token accounts must be writable");
        return Err(XBoothError::AccountIsNotWritable.into());
    }

    if !exchange_booth_account.is_writable {
        msg!("Exchange booth needs to be writable");
        return Err(XBoothError::AccountIsNotWritable.into());
    }

    // check ownership of exchange booth
    let (_xbooth_pda, xbooth_bump) = utils::get_exchange_booth_pda(
        program_id,
        exchange_booth_account,
        authority_account,
        mint_a,
        mint_b,
    )
    .unwrap();

    // check ownership of vault
    let (_vault_pda, vault_bump) = utils::get_vault_pda(
        program_id,
        exchange_booth_account,
        authority_account,
        mint_a,
        vault_account,
    )
    .unwrap();

    // check stored admin/owner of exchange booth
    utils::check_stored_owner(exchange_booth_account, authority_account).unwrap();

    // * withdraw money from vault into token_account using spl program
    // Check amount in vault
    let vault_account_data =
        spl_token::state::Account::unpack(&vault_account.data.borrow()).unwrap();

    let amount_lamports = utils::amount_to_lamports(mint_a, amount).unwrap();
    if amount_lamports > vault_account_data.amount {
        msg!("insufficient funds in vault accounts");
        return Err(XBoothError::InsufficientFunds.into());
    }

    let transfer_ix = spl_token::instruction::transfer(
        &token_program.key,
        &vault_account.key,
        &token_account.key,
        &exchange_booth_account.key,
        &[],
        amount_lamports,
    )
    .unwrap();

    invoke_signed(
        &transfer_ix,
        &[
            token_program.clone(),
            vault_account.clone(),
            token_account.clone(),
            exchange_booth_account.clone(),
        ],
        &[&[
            b"xbooth",
            authority_account.key.as_ref(),
            mint_a.key.as_ref(),
            mint_b.key.as_ref(),
            &[xbooth_bump],
        ]],
    )
    .unwrap();

    Ok(())
}
