use crate::instruction::accounts::VoteAccounts;
use crate::instruction::VoteMeta;
use crate::state::Leaderboard;
use crate::{accounts, state, utils};
use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use solana_program::clock::Clock;
use solana_program::msg;
use solana_program::program::invoke_signed;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn vote<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: VoteMeta,
) -> ProgramResult {
    let ctx: crate::instruction::accounts::Context<VoteAccounts> = VoteAccounts::context(accounts)?;

    if !ctx.accounts.user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let clock = Clock::get()?;
    let current_date = (clock.unix_timestamp / (24 * 60 * 60)) as u32;

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
            &ctx.accounts.creator.key.to_bytes(),
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

    let _user_data_bump = accounts::check_program_data_account(
        ctx.accounts.user_data,
        program_id,
        vec![&ctx.accounts.user.key.to_bytes(), b"User"],
    )
    .unwrap();

    let _creator_data_bump = accounts::check_program_data_account(
        ctx.accounts.creator_data,
        program_id,
        vec![&ctx.accounts.creator.key.to_bytes(), b"User"],
    )
    .unwrap();

    accounts::check_token_account(
        ctx.accounts.user,
        ctx.accounts.bash_mint,
        ctx.accounts.user_token,
        ctx.accounts.token_2022,
    )?;

    accounts::check_system_program_key(ctx.accounts.system_program)?;

    if ctx.accounts.creator.key == ctx.accounts.user.key {
        msg!("Creator cannot vote on their own entry");
        return Err(ProgramError::InvalidArgument);
    }

    if **ctx.accounts.entry.try_borrow_lamports()? == 0 {
        msg!("No entry for date and user");
        return Err(ProgramError::UninitializedAccount);
    }

    utils::create_user_data(
        ctx.accounts.user,
        ctx.accounts.user_data,
        ctx.accounts.data,
        program_id,
    )?;

    let mut entry = state::Entry::try_from_slice(&ctx.accounts.entry.data.borrow()[..])?;
    let mut creator_data =
        state::User::try_from_slice(&ctx.accounts.creator_data.data.borrow()[..])?;
    let mut voter_data = state::User::try_from_slice(&ctx.accounts.user_data.data.borrow()[..])?;

    msg!("have initial data");

    if args.vote == 1 {
        entry.positive_votes += 1;
        creator_data.total_positive_votes += 1;
        voter_data.total_positive_voted += 1;
    } else if args.vote == 2 {
        entry.negative_votes += 1;
        creator_data.total_negative_votes += 1;
        voter_data.total_negative_voted += 1;
    } else {
        return Err(ProgramError::InvalidArgument);
    }

    entry.serialize(&mut &mut ctx.accounts.entry.data.borrow_mut()[..])?;
    creator_data.serialize(&mut &mut ctx.accounts.creator_data.data.borrow_mut()[..])?;
    voter_data.serialize(&mut &mut ctx.accounts.user_data.data.borrow_mut()[..])?;

    msg!("mint reward");

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
        10,
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

    msg!("update leaderboard");

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

    // check if we have a reference
    if ctx.accounts.reference.is_some() {
        let reference = ctx.accounts.reference.unwrap();
        if reference.key != ctx.accounts.user.key {
            let ref_bash = ctx.accounts.ref_bash.unwrap();
            accounts::check_token_account(
                reference,
                ctx.accounts.bash_mint,
                ref_bash,
                ctx.accounts.token_2022,
            )?;

            // only transfer if the ATA already exists
            if **ref_bash.try_borrow_lamports()? > 0 {
                utils::mint(
                    10,
                    ctx.accounts.token_2022,
                    ctx.accounts.bash_mint,
                    ref_bash,
                    ctx.accounts.pda,
                    pda_bump_seed,
                )?;
            }
        }
    }

    let mut leaderboard = Leaderboard::try_from_slice(&ctx.accounts.leaderboard.data.borrow()[..])?;
    let old_size = ctx.accounts.leaderboard.data_len();

    // check if the new score is higher than the lowest of the top 10
    let mut min: i64 = i64::MAX;
    let mut min_index: usize = 0;
    let mut present: bool = false;
    for i in 0..leaderboard.scores.len() {
        if creator_data.user_id == leaderboard.entrants[i] {
            present = true;
            min_index = i;
            break;
        }
        if (leaderboard.scores[i] as i64) < min {
            min = leaderboard.scores[i] as i64;
            min_index = i;
        }
    }

    let entry_score: i64 = (entry.positive_votes as i64) - (entry.negative_votes as i64);
    msg!("Have score {}", entry_score);
    if entry_score < 0 {
        return Ok(());
    }
    if present {
        msg!("User already present in the top 10!");

        leaderboard.scores[min_index] = entry_score as u32;
        leaderboard.entrants[min_index] = creator_data.user_id;

        leaderboard.serialize(&mut &mut ctx.accounts.leaderboard.data.borrow_mut()[..])?;
    }

    if !present && (entry_score as i64 > min || leaderboard.scores.len() < 10) {
        msg!(
            "New entry in top 10! {} > {} for user {}",
            entry_score,
            min,
            creator_data.user_id
        );

        // if we have less than 10 then just add, otherwse replace
        if leaderboard.scores.len() < 10 {
            leaderboard.scores.push(entry_score as u32);
            leaderboard.entrants.push(creator_data.user_id);

            utils::check_for_realloc(
                ctx.accounts.leaderboard,
                ctx.accounts.user,
                old_size,
                to_vec(&leaderboard).unwrap().len(),
            )?;
        } else {
            leaderboard.scores[min_index] = entry_score as u32;
            leaderboard.entrants[min_index] = creator_data.user_id;
        }

        leaderboard.serialize(&mut &mut ctx.accounts.leaderboard.data.borrow_mut()[..])?;
    }

    Ok(())
}
