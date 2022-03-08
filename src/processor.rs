use crate::errors::XBoothError;
use crate::instruction::XBoothIntruction;
use borsh::BorshDeserialize;

use solana_program::{
    account_info::AccountInfo,
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    msg,
    program_error::{PrintProgramError, ProgramError},
    pubkey::Pubkey,
};

pub mod deposit;
pub mod initialize_exchange_booth;
pub mod utils;
pub mod withdraw;
pub struct Processor;

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("xbooth: process instructions");
        let instruction = XBoothIntruction::try_from_slice(instruction_data).map_err(|err| {
            msg!("invalid instruction data. cause {:}", err);
            ProgramError::InvalidInstructionData
        })?;
        msg!("instruction: {:?}", instruction);
        match instruction {
            XBoothIntruction::InitializeExhangeBooth {} => {
                msg!("Initialize Exchange booth");
                initialize_exchange_booth::process(&program_id, &accounts)?;
            }
            XBoothIntruction::Deposit { amount } => {
                msg!("xbooth deposit ");
                deposit::process(program_id, accounts, amount)?;
            }
            XBoothIntruction::Withdraw { amount } => {
                msg!("xbooth withdraw");
                withdraw::process(program_id, accounts)?;
            }
        }
        Ok(())
    }
}

impl PrintProgramError for XBoothError {
    fn print<E>(&self)
    where
        E: 'static
            + std::error::Error
            + DecodeError<E>
            + PrintProgramError
            + num_traits::FromPrimitive,
    {
        msg!(&self.to_string());
    }
}
