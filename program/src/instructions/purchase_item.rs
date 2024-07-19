use crate::instruction::accounts::PurchaseItemAccounts;
use crate::instruction::PurchaseMeta;
use crate::{accounts, state, utils};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_core::instructions::TransferV1CpiBuilder;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token_2022::extension::StateWithExtensions;

pub fn purchase_item<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: PurchaseMeta,
) -> ProgramResult {
    let ctx: crate::instruction::accounts::Context<PurchaseItemAccounts> =
        PurchaseItemAccounts::context(accounts)?;

    if !ctx.accounts.user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let pda_bump_seed = accounts::check_program_data_account(
        ctx.accounts.pda,
        program_id,
        vec![&accounts::PDA_SEED.to_le_bytes()],
    )
    .unwrap();

    let _listing_bump_seed = accounts::check_program_data_account(
        ctx.accounts.listing,
        program_id,
        vec![&ctx.accounts.item.key.to_bytes(), b"Listing"],
    )
    .unwrap();

    accounts::check_token_account(
        ctx.accounts.user,
        ctx.accounts.bash_mint,
        ctx.accounts.user_bash,
        ctx.accounts.token_2022,
    )?;

    let listing_2022 = accounts::check_token_program_key(ctx.accounts.listing_tp)?;

    if ctx.accounts.bash_mint.key != &accounts::bash_mint::ID {
        return Err(ProgramError::InvalidAccountData);
    }

    let mut listing = state::Listing::try_from_slice(&ctx.accounts.listing.data.borrow()[..])?;

    if listing.item_address != *ctx.accounts.item.key {
        return Err(ProgramError::InvalidAccountData);
    }

    accounts::check_system_program_key(ctx.accounts.system_program)?;

    if listing.item_address != *ctx.accounts.item.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let quantity = args.quantity.min(listing.quantity);

    //token
    if listing.item_type == 1 {
        let mint_data = ctx.accounts.item.data.borrow();
        let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;

        let float_q = quantity as f64 / 10_f64.powi(mint.base.decimals as i32);
        let price = (float_q * listing.price as f64) as u64;

        utils::burn(
            price,
            ctx.accounts.token_2022,
            ctx.accounts.bash_mint,
            ctx.accounts.user_bash,
            ctx.accounts.user,
        )?;

        accounts::check_token_account(
            ctx.accounts.user,
            ctx.accounts.item,
            ctx.accounts.user_item,
            ctx.accounts.listing_tp,
        )?;

        accounts::check_token_account(
            ctx.accounts.pda,
            ctx.accounts.item,
            ctx.accounts.pda_item,
            ctx.accounts.listing_tp,
        )?;

        utils::create_ata(
            ctx.accounts.user,
            ctx.accounts.user,
            ctx.accounts.item,
            ctx.accounts.user_item,
            ctx.accounts.listing_tp,
        )?;

        utils::transfer_tokens(
            listing_2022,
            quantity,
            ctx.accounts.pda_item,
            ctx.accounts.item,
            ctx.accounts.user_item,
            ctx.accounts.pda,
            ctx.accounts.listing_tp,
            pda_bump_seed,
            &vec![&accounts::PDA_SEED.to_le_bytes()],
            mint.base.decimals,
            &Vec::new(),
        )?;

        listing.quantity -= args.quantity;
        listing.serialize(&mut &mut ctx.accounts.listing.data.borrow_mut()[..])?;
    }

    //core asset
    if listing.item_type == 2 {
        let _transfer = TransferV1CpiBuilder::new(ctx.accounts.core)
            .asset(ctx.accounts.item)
            .authority(Some(ctx.accounts.pda))
            .payer(ctx.accounts.user)
            .new_owner(ctx.accounts.user)
            .collection(Some(ctx.accounts.collection))
            .invoke_signed(&[&[&accounts::PDA_SEED.to_le_bytes(), &[pda_bump_seed]]])?;

        utils::burn(
            listing.price,
            ctx.accounts.token_2022,
            ctx.accounts.bash_mint,
            ctx.accounts.user_bash,
            ctx.accounts.user,
        )?;

        listing.quantity = 0;
        listing.serialize(&mut &mut ctx.accounts.listing.data.borrow_mut()[..])?;

        let account_lamports = **ctx.accounts.listing.try_borrow_lamports()?;

        **ctx.accounts.listing.try_borrow_mut_lamports()? -= account_lamports;
        **ctx.accounts.pda.try_borrow_mut_lamports()? += account_lamports;
    }

    Ok(())
}
