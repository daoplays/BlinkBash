use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};
use solana_program::{declare_id, pubkey::Pubkey};

use spl_associated_token_account::get_associated_token_address_with_program_id;

pub const PDA_SEED: u32 = 6968193;
pub const DATA_SEED: u32 = 10399637;

pub mod daoplays_account {
    use super::*;
    declare_id!("FxVpjJ5AGY6cfCwZQP5v8QBfS4J2NPa62HbGh1Fu2LpD");
}

pub mod bash_mint {
    use super::*;
    declare_id!("BASH6YCvhMeKGzTTmHquBCHeoyPJRDMYE7yQvYXerbcg");
}

pub mod whitelist_mint {
    use super::*;
    declare_id!("BASHr9FsPoGq1LVWxSZLKHM6KMd7cjycjYH1eW25oC2K");
}

pub mod prod_fees_account {
    use super::*;
    declare_id!("HtszJ5ntXnwUFc2anMzp5RgaPxtvTFojL2qb5kcFEytA");
}

pub mod wrapped_sol_mint_account {
    use super::*;
    declare_id!("So11111111111111111111111111111111111111112");
}

pub mod core_account {
    use super::*;
    declare_id!("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");
}

//////////// helper functions for checking accounts ////////////////

pub fn check_core_key<'a>(account_info: &'a AccountInfo<'a>) -> ProgramResult {
    if account_info.key != &core_account::ID {
        msg!("expected core {} {}", core_account::ID, account_info.key);
        return Err(ProgramError::InvalidAccountData);
    }

    return Ok(());
}

pub fn check_wrapped_sol_key<'a>(account_info: &'a AccountInfo<'a>) -> ProgramResult {
    if account_info.key != &wrapped_sol_mint_account::ID {
        msg!(
            "expected wrapped sol mint {} {}",
            wrapped_sol_mint_account::ID,
            account_info.key
        );
        return Err(ProgramError::InvalidAccountData);
    }

    return Ok(());
}

pub fn check_system_program_key<'a>(account_info: &'a AccountInfo<'a>) -> ProgramResult {
    if account_info.key != &solana_program::system_program::ID {
        msg!(
            "expected system program {}",
            solana_program::system_program::ID
        );
        return Err(ProgramError::InvalidAccountData);
    }

    return Ok(());
}

pub fn check_token_program_key<'a>(
    account_info: &'a AccountInfo<'a>,
) -> Result<bool, ProgramError> {
    if account_info.key == &spl_token::id() {
        return Ok(false);
    }

    if account_info.key == &spl_token_2022::id() {
        return Ok(true);
    }

    return Err(ProgramError::InvalidAccountData);
}

pub fn check_associated_token_program_key<'a>(account_info: &'a AccountInfo<'a>) -> ProgramResult {
    if account_info.key != &spl_associated_token_account::ID {
        msg!(
            "expected associated token program {} {}",
            spl_associated_token_account::ID,
            account_info.key
        );
        return Err(ProgramError::InvalidAccountData);
    }

    return Ok(());
}

pub fn check_program_data_account<'a>(
    account_info: &'a AccountInfo<'a>,
    program_id: &Pubkey,
    seed: Vec<&[u8]>,
) -> Result<u8, ProgramError> {
    if seed.len() == 1 {
        let (expected_data_account, bump_seed) =
            Pubkey::find_program_address(&[seed[0]], &program_id);

        // the third account is the user's token account
        if account_info.key != &expected_data_account {
            msg!("expected program data account {}", expected_data_account);
            return Err(ProgramError::InvalidAccountData);
        }

        return Ok(bump_seed);
    }

    if seed.len() == 2 {
        let (expected_data_account, bump_seed) =
            Pubkey::find_program_address(&[seed[0], seed[1]], &program_id);

        // the third account is the user's token account
        if account_info.key != &expected_data_account {
            msg!("expected program data account {}", expected_data_account);
            return Err(ProgramError::InvalidAccountData);
        }

        return Ok(bump_seed);
    }

    let (expected_data_account, bump_seed) =
        Pubkey::find_program_address(&[seed[0], seed[1], seed[2]], &program_id);

    // the third account is the user's token account
    if account_info.key != &expected_data_account {
        msg!("expected program data account {}", expected_data_account);
        return Err(ProgramError::InvalidAccountData);
    }

    return Ok(bump_seed);
}

pub fn check_token_account<'a>(
    account_info: &'a AccountInfo<'a>,
    mint_account_info: &'a AccountInfo<'a>,
    token_account_info: &'a AccountInfo<'a>,
    token_program: &'a AccountInfo<'a>,
) -> ProgramResult {
    let expected_token_account = get_associated_token_address_with_program_id(
        &account_info.key,
        &mint_account_info.key,
        &token_program.key,
    );
    // the third account is the user's token account
    if token_account_info.key != &expected_token_account {
        msg!(
            "expected token account {} for mint {} and account {}, recieved {}",
            expected_token_account,
            mint_account_info.key,
            account_info.key,
            token_account_info.key
        );
        return Err(ProgramError::InvalidAccountData);
    }

    return Ok(());
}
