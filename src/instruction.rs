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
    ///
    /// instruction_data
    /// amount: amount of tokens of x that should be deposited
    InitializeExhangeBooth {},
    /// Deposit
    /// allows the booth admin to deposit tokens into one of the vaults
    /// from the booth
    ///
    /// Accounts:
    /// 1. exchange_booth_account: pda
    ///     - is_signer:false,
    ///     - is_writable: true
    /// 2. authority: the signer of the transaction, owner of token_account
    ///     - is_signer: true,
    ///     - is_writable: false
    /// 3. token_account: account holding tokens from mint A
    ///     - is_signer: true,
    ///     - is_writable: true,
    /// 4. vault A: pda, vault that can hold mint A
    ///     - is_signer:false,
    ///     - is_writable: false
    /// 5. mint A: the mint account of Token A
    ///     - is_signer: false,
    ///     - is_writable: true
    /// 6. mint B: the mint account of Token B
    ///     - is_signer: false,
    ///     - is_writable: true
    /// 7. token program: The spl_token program
    ///     - is_signer: false,
    ///     - is_writable: false
    Deposit { amount: f64 },
    /// Withdraw
    /// should allow the owner of the exchange booth to
    /// withdraw from any of the vaults and transfer it to a token account
    ///
    /// Accounts:
    /// 1. exchange_booth_account: pda
    ///     - is_signer:false,
    ///     - is_writable: true
    /// 2. authority: the signer of the transaction, owner of token_account
    ///     - is_signer: true,
    ///     - is_writable: false
    /// 3. token_account: account holding tokens from mint A
    ///     - is_signer: true,
    ///     - is_writable: true,
    /// 4. vault A: pda, vault that can hold mint A
    ///     - is_signer:false,
    ///     - is_writable: true
    /// 5. mint A: the mint account of Token A
    ///     - is_signer: false,
    ///     - is_writable: false
    /// 6. mint B: the mint account of Token B
    ///     - is_signer: false,
    ///     - is_writable: false
    /// 7. token program: The spl_token program
    ///     - is_signer: false,
    ///     - is_writable: false
    Withdraw { amount: f64 },
    /// Exchange tokens
    /// should allow anybody to exchange token A for token B at an exchange rate A/B
    ///
    /// Accounts:
    /// 1. Exchange_booth_account: pda
    ///     - is_signer:false,
    ///     - is_writable: false
    /// 2. authority: signer of the transaction, owner of the token_account
    ///     - is_signer:true,
    ///     - is_writable: false,
    /// 3. to_token_account: token to withdraw and deposit into
    ///     - is_signer: false,
    ///     - is_writable: true,
    /// 4. from_token_account: token to withdraw and deposit into
    ///     - is_signer: false,
    ///     - is_writable: true,
    /// 4. vault A: pda
    ///     - is_signer: false,
    ///     - is_writable:true
    /// 5. vault B: pda
    ///     - is_signer: false,
    ///     - is_writable: true,
    /// 6. mint_a: mint account for token A
    ///     - is_signer: false,
    ///     - is_writable: false
    /// 7. mint_b: mint account for token B
    ///     - is_signer: false,
    ///     - is_writable: false
    /// 8. token_program: the spl_token program
    ///     - is_signer: false,
    ///     - is_writable: false
    /// (9. oracle account)
    Exchange { amount: f64 },
}
