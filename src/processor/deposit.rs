use borsh::BorshDeserialize;
use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};
use spl_token::{instruction, state::Account};

use crate::errors::XBoothError;
use crate::processor::utils;
use crate::state::ExchangeBoothAccount;

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], amount: f64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let exchange_booth_account = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let vault = next_account_info(accounts_iter)?;
    let mint_a = next_account_info(accounts_iter)?;
    let mint_b = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // check corret permissions
    if !authority.is_signer {
        msg!("owner must be signer");
        return Err(XBoothError::AccountIsNotSigner.into());
    }

    if !exchange_booth_account.is_writable {
        msg!("exchange booth account must be writable");
        return Err(XBoothError::AccountIsNotWritable.into());
    }

    if !vault.is_writable {
        msg!("vault must be writable");
        return Err(XBoothError::AccountIsNotWritable.into());
    }

    if !token_account.is_writable {
        msg!("the token account must be writable");
        return Err(XBoothError::AccountIsNotWritable.into());
    }

    let token_account_data =
        spl_token::state::Account::unpack_from_slice(&token_account.try_borrow_data()?)?;

    let is_transfer_a_token = token_account_data.mint == *mint_a.key;

    let vault_account = Account::unpack(&vault.data.borrow())
        .map_err(|err| {
            msg!("invalid vault account");
            return err;
        })
        .unwrap();

    if !vault_account.is_initialized() {
        msg!("vault is not initialized");
        return Err(XBoothError::AccountNotInitialized.into());
    }

    if vault_account.mint != token_account_data.mint {
        msg!("vault account and token account is of different mints");
        return Err(XBoothError::InvalidMint.into());
    }

    // Decide if token A or B
    // check the owner of the exchange booth account
    let exchange_booth_data = &mut (*exchange_booth_account.data).borrow_mut();
    let xbooth_data = ExchangeBoothAccount::try_from_slice(&exchange_booth_data).unwrap();
    if xbooth_data.admin != *authority.key {
        msg!("owner is not admin for the exchange booth");
        return Err(XBoothError::InvalidOwner.into());
    }

    // Check the vault
    let (_vault_pda, _vault_bump_seed) = utils::get_vault_pda(
        program_id,
        exchange_booth_account,
        authority,
        if is_transfer_a_token { mint_a } else { mint_b },
        vault,
    )
    .unwrap();

    let (_xbooth_pda, _xbooth_bump) = utils::get_exchange_booth_pda(
        program_id,
        exchange_booth_account,
        authority,
        mint_a,
        mint_b,
    )
    .unwrap();

    // check if enough funds in owner account
    let amount_lamports =
        utils::amount_to_lamports(if is_transfer_a_token { mint_a } else { mint_b }, amount)
            .unwrap();

    if token_account_data.amount < amount_lamports {
        msg!("not enough funds in account to transfer");
        return Err(XBoothError::InsufficientFunds.into());
    }
    msg!("lamports in token_account: {}", token_account_data.amount);

    // Transfer amount from owner to the vault
    let transfer_ix = instruction::transfer(
        &token_program.key,
        &token_account.key,
        &vault.key,
        &authority.key,
        &[&authority.key],
        amount_lamports,
    )
    .unwrap();

    invoke(
        &transfer_ix,
        &[
            token_program.clone(),
            token_account.clone(),
            vault.clone(),
            authority.clone(),
        ],
    )
    .unwrap();

    Ok(())
}
