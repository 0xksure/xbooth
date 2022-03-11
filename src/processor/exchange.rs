use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
};

use crate::errors::XBoothError;
use crate::processor::utils;

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], amount: f64) -> ProgramResult {
    let accounts = &mut accounts.iter();
    let exchange_booth_account = next_account_info(accounts)?;
    let authority_account = next_account_info(accounts)?;
    let from_token_account = next_account_info(accounts)?;
    let receiving_token_account = next_account_info(accounts)?;
    let vault_a = next_account_info(accounts)?;
    let vault_b = next_account_info(accounts)?;
    let mint_a = next_account_info(accounts)?;
    let mint_b = next_account_info(accounts)?;
    let token_program = next_account_info(accounts)?;

    // * checks
    if !authority_account.is_signer {
        msg!("authority needs to have signer privilege");
        return Err(XBoothError::AccountIsNotSigner.into());
    }

    if !receiving_token_account.is_writable {
        msg!("receiving token account needs to be writable");
        return Err(XBoothError::AccountIsNotWritable.into());
    }

    if !from_token_account.is_writable {
        msg!("from token account needs to be writable");
        return Err(XBoothError::AccountIsNotWritable.into());
    }

    if !vault_a.is_writable {
        msg!("vault A is not writable");
        return Err(XBoothError::AccountIsNotWritable.into());
    }

    if !vault_b.is_writable {
        msg!("vailt B is not writable");
        return Err(XBoothError::AccountIsNotWritable.into());
    }

    let receiving_token_account_data =
        spl_token::state::Account::unpack(&receiving_token_account.data.borrow())?;
    let from_token_account_data =
        spl_token::state::Account::unpack(&from_token_account.data.borrow())?;

    if &from_token_account_data.mint != mint_a.key {
        msg!("sending token account is not of the same mint as token A");
        return Err(XBoothError::InvalidMint.into());
    }

    if &receiving_token_account_data.mint != mint_b.key {
        msg!("receving token account is not of the same mint as token B");
        return Err(XBoothError::InvalidMint.into());
    }

    if receiving_token_account_data.mint == from_token_account_data.mint {
        msg!("receiving token account cannot be of the same mint as the sending token account");
        return Err(XBoothError::UniqueMintAccounts.into());
    }

    // get exchange_booth_account pda and bump
    let (_exchange_booth_pda, exchange_booth_bump) = utils::get_exchange_booth_pda(
        program_id,
        exchange_booth_account,
        authority_account,
        mint_a,
        mint_b,
    )
    .unwrap();

    // * Exchange
    // send
    let token_a_b_xr = 0.5;
    let amount_a: u64 = utils::amount_to_lamports(mint_a, amount).unwrap();
    let amount_b: u64 = utils::amount_to_lamports(mint_b, amount * token_a_b_xr).unwrap();

    msg!(
        "transfer amount: {} from token account to vault A",
        amount_a
    );
    let deposit_into_a_ix = spl_token::instruction::transfer(
        &token_program.key,
        &from_token_account.key,
        &vault_a.key,
        &authority_account.key,
        &[&authority_account.key],
        amount_a,
    )
    .unwrap();

    invoke(
        &deposit_into_a_ix,
        &[
            token_program.clone(),
            from_token_account.clone(),
            vault_a.clone(),
            authority_account.clone(),
        ],
    )
    .unwrap();

    // return
    let vault_b_account =
        spl_token::state::Account::unpack_from_slice(&vault_b.try_borrow_mut_data()?)?;
    msg!(
        "transfer amount: {} from vault B with balance {} to receiving token account",
        amount_b,
        vault_b_account.amount
    );
    let withdraw_from_b_ix = spl_token::instruction::transfer(
        &token_program.key,
        &vault_b.key,
        &receiving_token_account.key,
        &exchange_booth_account.key,
        &[],
        amount_b,
    )
    .unwrap();

    invoke_signed(
        &withdraw_from_b_ix,
        &[
            token_program.clone(),
            vault_b.clone(),
            receiving_token_account.clone(),
            exchange_booth_account.clone(),
        ],
        &[&[
            b"xbooth",
            authority_account.key.as_ref(),
            mint_a.key.as_ref(),
            mint_b.key.as_ref(),
            &[exchange_booth_bump],
        ]],
    )
    .unwrap();

    Ok(())
}
