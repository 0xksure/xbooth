use borsh::BorshDeserialize;
use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};
use spl_token::{instruction, state::Account};

use crate::errors::XBoothError;
use crate::processor;
use crate::state::ExchangeBoothAccount;

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
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

    // Check the vault
    let (_vault_pda, _vault_bump_seed) = processor::utils::get_vault_pda(
        program_id,
        exchange_booth_account,
        authority,
        mint_a,
        vault,
    )
    .unwrap();

    // check the owner of the exchange booth account
    let exchange_booth_data = &mut (*exchange_booth_account.data).borrow_mut();

    let xbooth_data = ExchangeBoothAccount::try_from_slice(&exchange_booth_data).unwrap();
    if xbooth_data.admin != *authority.key {
        msg!("owner is not admin for the exchange booth");
        return Err(XBoothError::InvalidOwner.into());
    }

    let (_xbooth_pda, _xbooth_bump) = processor::utils::get_exchange_booth_pda(
        program_id,
        exchange_booth_account,
        authority,
        mint_a,
        mint_b,
    )
    .unwrap();

    // check if enough funds in owner account
    let spl_account = Account::unpack(&token_account.data.borrow())
        .map_err(|err| {
            msg!("invalid spl token account. Maybe account is not setup to be an spl account");
            return XBoothError::InvalidSPLTokenAccount;
        })
        .unwrap();

    if spl_account.amount < amount {
        msg!("not enough funds in account to transfer");
        return Err(XBoothError::InsufficientFunds.into());
    }
    msg!("amount in token_account: {}", spl_account.amount);

    // Transfer amount from owner to the vault
    let transfer_ix = instruction::transfer(
        &token_program.key,
        &token_account.key,
        &vault.key,
        &authority.key,
        &[&authority.key],
        amount,
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
