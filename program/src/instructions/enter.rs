use crate::instruction::accounts::EnterAccounts;
use crate::instruction::EnterMeta;
use crate::state::{self, Entry, Leaderboard};
use crate::{accounts, utils};
use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use solana_program::clock::Clock;
use solana_program::msg;
use solana_program::program::invoke_signed;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn enter<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: EnterMeta,
) -> ProgramResult {
    let ctx: crate::instruction::accounts::Context<EnterAccounts> =
        EnterAccounts::context(accounts)?;

    if !ctx.accounts.user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let pda_bump_seed = accounts::check_program_data_account(
        ctx.accounts.pda,
        program_id,
        vec![&accounts::PDA_SEED.to_le_bytes()],
    )
    .unwrap();

    let _data_bump_seed = accounts::check_program_data_account(
        ctx.accounts.data,
        program_id,
        vec![&accounts::DATA_SEED.to_le_bytes()],
    )
    .unwrap();

    let _user_data_bump = accounts::check_program_data_account(
        ctx.accounts.user_data,
        program_id,
        vec![&ctx.accounts.user.key.to_bytes(), b"User"],
    )
    .unwrap();

    accounts::check_token_account(
        ctx.accounts.user,
        ctx.accounts.bash_mint,
        ctx.accounts.user_token,
        ctx.accounts.token_2022,
    )?;

    let clock = Clock::get()?;
    let current_date = (clock.unix_timestamp / (24 * 60 * 60)) as u32;

    let entry_bump_seed = accounts::check_program_data_account(
        ctx.accounts.entry,
        program_id,
        vec![
            &ctx.accounts.user.key.to_bytes(),
            &args.game.to_le_bytes(),
            &current_date.to_le_bytes(),
        ],
    )
    .unwrap();

    let leaderboard_bump_seed = accounts::check_program_data_account(
        ctx.accounts.leaderboard,
        program_id,
        vec![
            &args.game.to_le_bytes(),
            &current_date.to_le_bytes(),
            b"Leaderboard",
        ],
    )
    .unwrap();

    accounts::check_system_program_key(ctx.accounts.system_program)?;

    utils::create_user_data(
        ctx.accounts.user,
        ctx.accounts.user_data,
        ctx.accounts.data,
        program_id,
    )?;

    // crate the entry account if we need to
    if **ctx.accounts.entry.try_borrow_lamports()? == 0 {
        let temp: Entry = Entry {
            account_type: state::AccountType::Entry,
            positive_votes: 0,
            negative_votes: 0,
            reward_claimed: 0,
        };

        utils::create_program_account(
            ctx.accounts.user,
            ctx.accounts.entry,
            program_id,
            entry_bump_seed,
            to_vec(&temp).unwrap().len(),
            vec![
                &ctx.accounts.user.key.to_bytes(),
                &args.game.to_le_bytes(),
                &current_date.to_le_bytes(),
            ],
        )?;

        msg!("init entry data");
        temp.serialize(&mut &mut ctx.accounts.entry.data.borrow_mut()[..])?;

        utils::create_ata(
            ctx.accounts.user,
            ctx.accounts.user,
            ctx.accounts.bash_mint,
            ctx.accounts.user_token,
            ctx.accounts.token_2022,
        )?;

        // mint the token to the user
        let mint_to_idx = spl_token_2022::instruction::mint_to_checked(
            ctx.accounts.token_2022.key,
            ctx.accounts.bash_mint.key,
            ctx.accounts.user_token.key,
            ctx.accounts.pda.key,
            &[ctx.accounts.pda.key],
            100,
            1,
        )
        .unwrap();

        invoke_signed(
            &mint_to_idx,
            &[
                ctx.accounts.token_2022.clone(),
                ctx.accounts.bash_mint.clone(),
                ctx.accounts.user_token.clone(),
                ctx.accounts.user.clone(),
                ctx.accounts.pda.clone(),
            ],
            &[&[&accounts::PDA_SEED.to_le_bytes(), &[pda_bump_seed]]],
        )?;
    }

    if **ctx.accounts.leaderboard.try_borrow_lamports()? == 0 {
        let temp: Leaderboard = Leaderboard {
            account_type: state::AccountType::Leaderboard,
            game: args.game,
            date: current_date,
            entrants: Vec::new(),
            scores: Vec::new(),
        };

        utils::create_program_account(
            ctx.accounts.user,
            ctx.accounts.leaderboard,
            program_id,
            leaderboard_bump_seed,
            to_vec(&temp).unwrap().len(),
            vec![
                &args.game.to_le_bytes(),
                &current_date.to_le_bytes(),
                b"Leaderboard",
            ],
        )?;

        msg!("init leaderboard data");
        temp.serialize(&mut &mut ctx.accounts.leaderboard.data.borrow_mut()[..])?;
    }

    // check if we should add this entry to the leaderboard
    let mut leaderboard = Leaderboard::try_from_slice(&ctx.accounts.leaderboard.data.borrow()[..])?;
    let user_data = state::User::try_from_slice(&ctx.accounts.user_data.data.borrow()[..])?;

    if leaderboard.scores.len() < 10 {
        let old_size = ctx.accounts.leaderboard.data_len();

        leaderboard.entrants.push(user_data.user_id);
        leaderboard.scores.push(0);

        utils::check_for_realloc(
            ctx.accounts.leaderboard,
            ctx.accounts.user,
            old_size,
            to_vec(&leaderboard).unwrap().len(),
        )?;

        leaderboard.serialize(&mut &mut ctx.accounts.leaderboard.data.borrow_mut()[..])?;
    }

    Ok(())
}
