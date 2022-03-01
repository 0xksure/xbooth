use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum XBoothIntruction {
    /// Initialize Exhcange booth
    /// initialized the exchange booth to trade x for y and
    /// the price x/y can be found using an oracle
    ///
    /// Accounts:
    /// 1. exchange_boot_account: pda
    ///     - is_signer: false,
    ///     - is_writable: true,
    /// 2. payer
    ///     - is_signer: true,
    ///     - is_writable: false
    /// 3. system_program
    ///     - is_signer: false,
    ///     - is_writable: false,
    /// 4. vault A
    ///     - is_signer: false,
    ///     - is_writable: false
    /// 5. vault B
    ///     - is_signer: false,
    ///     - is_writable: false
    /// 6. token program A
    ///     - is_signer: false,
    ///     - is_writable: false
    ///  7. token program B
    ///     - is_signer: false,
    ///     - is_writable: false
    ///
    /// instruction_data
    /// amount: amount of tokens of x that should be deposited
    InitializeExhangeBooth {},
}
