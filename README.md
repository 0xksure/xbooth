# Exchange Booth Program

**As part of the Solana Chicago Bootcamp**

The repo is a WIP for the Solana smart contract specified in the [Exchange_booth_Program_Spec](https://github.com/jarry-xiao/solana-bootcamp-lectures/blob/master/project_specs/Exchange_Booth_Program_Spec.pdf). The current code is inspired by the material presented during the [Solana Bootcamp series on youtube](https://www.youtube.com/watch?v=O0uhZEfVPt8&list=PLilwLeBwGuK7Z2dXft_pmLZ675fuPgkA0).

## Structure

The project follows pretty basic Solana structure with a src folder containing the smart contract and a test folder containing the solana_program_test.

There are no unit tests as the integration like tests provided by the solana_program_test crate provides more robust testing.

TODO:

- [x] Initialize Exchange Booth
- [x] Deposit into vault
- [x] Withdraw from vault
- [x] Exchange tokens using exchange booth
- [ ] Close excahnge booth

**The code is only meant as educational and is not a complete smart contract ready for anything else than you local validator network**
