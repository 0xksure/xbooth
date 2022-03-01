use num_derive::FromPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, FromPrimitive, PartialEq)]
pub enum XBoothError {
    #[error("Invalid Account address.")]
    InvalidAccountAddress,
    #[error("Invalid Vault Account")]
    InvalidVaultAccount,
}

impl From<XBoothError> for ProgramError {
    fn from(e: XBoothError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
