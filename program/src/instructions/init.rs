use crate::instruction::accounts::InitAccounts;
use crate::state::{self, ProgramStats};
use crate::{accounts, utils};
use borsh::to_vec;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn init<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let ctx: crate::instruction::accounts::Context<InitAccounts> = InitAccounts::context(accounts)?;

    if !ctx.accounts.user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let data_bump_seed = accounts::check_program_data_account(
        ctx.accounts.data,
        program_id,
        vec![&accounts::DATA_SEED.to_le_bytes()],
    )
    .unwrap();

    let pda_bump_seed = accounts::check_program_data_account(
        ctx.accounts.pda,
        program_id,
        vec![&accounts::PDA_SEED.to_le_bytes()],
    )
    .unwrap();

    if ctx.accounts.token_mint.key != &accounts::bash_mint::ID {
        return Err(ProgramError::InvalidAccountData);
    }

    accounts::check_system_program_key(ctx.accounts.system_program)?;

    // create the account if required
    utils::create_program_account(
        ctx.accounts.user,
        ctx.accounts.pda,
        ctx.accounts.system_program.key,
        pda_bump_seed,
        0,
        vec![&accounts::PDA_SEED.to_le_bytes()],
    )?;

    let temp: ProgramStats = ProgramStats {
        account_type: state::AccountType::Program,
        num_users: 0,
    };

    utils::create_program_account(
        ctx.accounts.user,
        ctx.accounts.data,
        program_id,
        data_bump_seed,
        to_vec(&temp).unwrap().len(),
        vec![&accounts::DATA_SEED.to_le_bytes()],
    )?;

    //create $BASH mint
    utils::create_2022_token(
        ctx.accounts.user,
        ctx.accounts.pda,
        ctx.accounts.token_2022,
        pda_bump_seed,
        ctx.accounts.token_mint,
        state::TokenDetails {
            name: "$BASH".to_string(),
            symbol: "$BASH".to_string(),
            uri: "https://gateway.irys.xyz/qLQB-e_wH7Mq2PWIuREWocVxIjHrkH_KJcE1X7RSnm8".to_string(),
            decimals: 1,
        },
    )?;

    Ok(())
}
