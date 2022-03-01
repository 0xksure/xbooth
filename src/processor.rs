use crate::errors::XBoothError;
use crate::instruction::XBoothIntruction;
use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::next_account_info, account_info::AccountInfo, entrypoint::ProgramResult, msg,
    program::invoke, program::invoke_signed, program_error::ProgramError, program_pack::Pack,
    pubkey::Pubkey, system_instruction::create_account,
};

use spl_token::state::Account as TokenAccount;

pub struct Processor;

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = XBoothIntruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match instruction {
            XBoothIntruction::InitializeExhangeBooth {} => {
                msg!("Initialize Exchange booth");
                let accounts_iter = &mut accounts.iter();
                let exchange_booth_account = next_account_info(accounts_iter)?;
                let payer = next_account_info(accounts_iter)?;
                let system_program = next_account_info(accounts_iter)?;
                let vault_a = next_account_info(accounts_iter)?;
                let vault_b = next_account_info(accounts_iter)?;
                let mint_a = next_account_info(accounts_iter)?;
                let mint_b = next_account_info(accounts_iter)?;
                let token_program = next_account_info(accounts_iter)?;
                let (pda, bump) = Pubkey::find_program_address(
                    &[
                        b"initialize_exchange_booth",
                        payer.key.as_ref(),
                        vault_a.key.as_ref(),
                        vault_b.key.as_ref(),
                    ],
                    program_id,
                );

                if pda != *exchange_booth_account.key {
                    msg!("exchange booth account ");
                    return Err(XBoothError::InvalidAccountAddress.into());
                }

                // check if vaults exists
                // create vault if not existant?
                let _vault_a = TokenAccount::unpack_unchecked(vault_a.data.borrow().as_ref())
                    .map_err(|e| {
                        msg!("invalid vault A");
                        return e;
                    })?;

                let _vault_b = TokenAccount::unpack_unchecked(vault_b.data.borrow().as_ref())
                    .map_err(|e| {
                        msg!("invalid vault B");
                        return e;
                    })?;

                // create exchange_boot_program
                let create_xbooth_program_ix =
                    create_account(&payer.key, &pda, 0_64, 0_u64, &payer.key);

                invoke_signed(
                    &create_xbooth_program_ix,
                    &[
                        payer.clone(),
                        exchange_booth_account.clone(),
                        system_program.clone(),
                    ],
                    &[&[
                        b"initialize_exchange_booth",
                        exchange_booth_account.key.as_ref(),
                        &[bump],
                    ]],
                )
                .unwrap();

                // deposit amount into exchange booth program_id
                let deposit_mint_a_token_ix = spl_token::instruction::transfer(
                    token_program.key,
                    payer.key,
                    &pda,
                    &payer.key,
                    &[&payer.key],
                    10,
                )?;

                let deposit_mint_b_token_ix = spl_token::instruction::transfer(
                    token_program.key,
                    payer.key,
                    &pda,
                    &payer.key,
                    &[&payer.key],
                    10,
                )?;

                invoke(
                    &deposit_mint_a_token_ix,
                    &[
                        payer.clone(),
                        exchange_booth_account.clone(),
                        system_program.clone(),
                    ],
                )?;

                invoke(
                    &deposit_mint_b_token_ix,
                    &[
                        payer.clone(),
                        exchange_booth_account.clone(),
                        system_program.clone(),
                    ],
                )?;
            }
        }
        Ok(())
    }
}
