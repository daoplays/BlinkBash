use crate::instruction::accounts::ClaimPrizeAccounts;
use crate::instruction::ClaimPrizeMeta;
use crate::{accounts, state};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::clock::Clock;
use solana_program::msg;
use solana_program::program::invoke_signed;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

fn sort_users_by_scores(users: &Vec<u32>, scores: &Vec<u32>) -> Vec<u32> {
    let mut indexed_users: Vec<(usize, &u32)> = users.iter().enumerate().collect();

    indexed_users.sort_by(|&(i, _), &(j, _)| scores[j].cmp(&scores[i]));

    indexed_users.into_iter().map(|(_, &user)| user).collect()
}

pub fn claim_prize<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: ClaimPrizeMeta,
) -> ProgramResult {
    let ctx: crate::instruction::accounts::Context<ClaimPrizeAccounts> =
        ClaimPrizeAccounts::context(accounts)?;

    if !ctx.accounts.user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let pda_bump_seed = accounts::check_program_data_account(
        ctx.accounts.pda,
        program_id,
        vec![&accounts::PDA_SEED.to_le_bytes()],
    )
    .unwrap();

    let _entry_bump_seed = accounts::check_program_data_account(
        ctx.accounts.entry,
        program_id,
        vec![
            &ctx.accounts.user.key.to_bytes(),
            &args.game.to_le_bytes(),
            &args.date.to_le_bytes(),
        ],
    )
    .unwrap();

    let _leaderboard_bump_seed = accounts::check_program_data_account(
        ctx.accounts.leaderboard,
        program_id,
        vec![
            &args.game.to_le_bytes(),
            &args.date.to_le_bytes(),
            b"Leaderboard",
        ],
    )
    .unwrap();

    let _user_data_bump = accounts::check_program_data_account(
        ctx.accounts.user_data,
        program_id,
        vec![&ctx.accounts.user.key.to_bytes(), b"User"],
    )
    .unwrap();

    accounts::check_system_program_key(ctx.accounts.system_program)?;

    let clock = Clock::get()?;
    let current_date = (clock.unix_timestamp / (24 * 60 * 60)) as u32;

    if args.date == current_date {
        msg!("cannot claim prize on the same day as the game");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut entry = state::Entry::try_from_slice(&ctx.accounts.entry.data.borrow()[..])?;
    let mut user_data = state::User::try_from_slice(&ctx.accounts.user_data.data.borrow()[..])?;
    let leaderboard =
        state::Leaderboard::try_from_slice(&ctx.accounts.leaderboard.data.borrow()[..])?;

    if entry.reward_claimed == 1 {
        msg!("reward already claimed");
        return Err(ProgramError::InvalidAccountData);
    }

    let n_players = leaderboard.entrants.len();
    if n_players == 0 {
        msg!("no entrants in the leaderboard");
        return Err(ProgramError::InvalidAccountData);
    }

    let sorted_users = sort_users_by_scores(&leaderboard.entrants, &leaderboard.scores);
    msg!("have users sorted by scores: {:?}", sorted_users);
    let amount: u64 = if sorted_users[0] == user_data.user_id {
        5000
    } else if n_players >= 2 && sorted_users[1] == user_data.user_id {
        2500
    } else if n_players >= 3 && sorted_users[2] == user_data.user_id {
        1000
    } else {
        0
    };

    if amount == 0 {
        msg!("user did not win a prize");
        return Err(ProgramError::InvalidAccountData);
    }

    if amount == 5000 {
        user_data.total_wins += 1;
        user_data.serialize(&mut &mut ctx.accounts.user_data.data.borrow_mut()[..])?;
    }

    // mint the token to the user
    let mint_to_idx = spl_token_2022::instruction::mint_to_checked(
        ctx.accounts.token_2022.key,
        ctx.accounts.bash_mint.key,
        ctx.accounts.user_token.key,
        ctx.accounts.pda.key,
        &[ctx.accounts.pda.key],
        amount,
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

    entry.reward_claimed = 1;

    entry.serialize(&mut &mut ctx.accounts.entry.data.borrow_mut()[..])?;

    Ok(())
}
