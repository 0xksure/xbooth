use crate::errors::XBoothError;
use crate::state::ExchangeBoothAccount;
use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, msg, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
};

pub fn get_exchange_booth_pda(
    program_id: &Pubkey,
    exchange_booth_account: &AccountInfo,
    owner: &AccountInfo,
    mint_a: &AccountInfo,
    mint_b: &AccountInfo,
) -> Result<(Pubkey, u8), ProgramError> {
    let (xbooth_pda, xbooth_bump_seed) = Pubkey::find_program_address(
        &[
            b"xbooth",
            owner.key.as_ref(),
            mint_a.key.as_ref(),
            mint_b.key.as_ref(),
        ],
        program_id,
    );

    // check if correct public key
    if xbooth_pda != *exchange_booth_account.key {
        msg!("Invalid account key for exchange booth");
        return Err(XBoothError::InvalidVaultAccount.into());
    }
    Ok((xbooth_pda, xbooth_bump_seed))
}

pub fn check_stored_owner(
    exchange_booth_account: &AccountInfo,
    authority: &AccountInfo,
) -> Result<(), ProgramError> {
    let data = &mut (*exchange_booth_account.data).borrow_mut();
    let exchange_booth_account_data = ExchangeBoothAccount::try_from_slice(&data).unwrap();
    if authority.key != &exchange_booth_account_data.admin {
        msg!("authority is not stored as sole owner of the exchange_booth_account");
        return Err(XBoothError::InvalidOwner.into());
    }
    Ok(())
}

pub fn get_vault_pda(
    program_id: &Pubkey,
    exchange_booth_account: &AccountInfo,
    owner: &AccountInfo,
    mint: &AccountInfo,
    vault: &AccountInfo,
) -> Result<(Pubkey, u8), ProgramError> {
    let (vault_pda, vault_b_bump_seed) = Pubkey::find_program_address(
        &[
            b"xbooth",
            owner.key.as_ref(),
            mint.key.as_ref(),
            exchange_booth_account.key.as_ref(),
        ],
        program_id,
    );
    // check if correct public key
    if vault_pda != *vault.key {
        msg!("Invalid account key for vault b");
        return Err(XBoothError::InvalidVaultAccount.into());
    }
    Ok((vault_pda, vault_b_bump_seed))
}

pub fn amount_to_lamports(mint: &AccountInfo, amount: f64) -> Result<u64, ProgramError> {
    let mint_account_data = spl_token::state::Mint::unpack_from_slice(&mint.try_borrow_data()?)?;
    let mint_decimals = mint_account_data.decimals;

    let lamports = (amount * f64::powf(10., mint_decimals.into())) as u64;
    Ok(lamports)
}
