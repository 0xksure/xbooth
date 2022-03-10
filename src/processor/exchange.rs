use solana_program::{
    account_info::next_account_info, account_info::AccountInfo, entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], amount: f64) -> ProgramResult {
    let accounts = &mut accounts.iter();
    let exchange_booth_account = next_account_info(accounts)?;
    let authority_account = next_account_info(accounts)?;
    let to_token_account = next_account_info(accounts)?;
    let from_token_account = next_account_info(accounts)?;
    let vault_a = next_account_info(accounts)?;
    let vault_b = next_account_info(accounts)?;
    let mint_a = next_account_info(accounts)?;
    let mint_b = next_account_info(accounts)?;
    let token_program = next_account_info(accounts)?;

    // * checks

    // * Exchange

    Ok(())
}
