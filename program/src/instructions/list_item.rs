use crate::instruction::accounts::ListItemAccounts;
use crate::instruction::ListMeta;
use crate::{accounts, state, utils};
use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use mpl_core::instructions::TransferV1CpiBuilder;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token_2022::extension::StateWithExtensions;

pub fn list_item<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: ListMeta,
) -> ProgramResult {
    let ctx: crate::instruction::accounts::Context<ListItemAccounts> =
        ListItemAccounts::context(accounts)?;

    if !ctx.accounts.user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let pda_bump_seed = accounts::check_program_data_account(
        ctx.accounts.pda,
        program_id,
        vec![&accounts::PDA_SEED.to_le_bytes()],
    )
    .unwrap();

    let listing_bump_seed = accounts::check_program_data_account(
        ctx.accounts.listing,
        program_id,
        vec![&ctx.accounts.item.key.to_bytes(), b"Listing"],
    )
    .unwrap();

    accounts::check_token_account(
        ctx.accounts.user,
        ctx.accounts.whitelist_mint,
        ctx.accounts.whitelist_account,
        ctx.accounts.token_2022,
    )?;

    let listing_2022 = accounts::check_token_program_key(ctx.accounts.listing_tp)?;

    accounts::check_system_program_key(ctx.accounts.system_program)?;

    // we only need to burn if we are actually listing something new rather than updating
    if **ctx.accounts.listing.try_borrow_lamports()? == 0 {
        utils::burn(
            1,
            ctx.accounts.token_2022,
            ctx.accounts.whitelist_mint,
            ctx.accounts.whitelist_account,
            ctx.accounts.user,
        )?;

        let listing: state::Listing = state::Listing {
            account_type: state::AccountType::Listing,
            item_type: args.item_type,
            item_address: *ctx.accounts.item.key,
            price: args.price,
            quantity: 0,
            bundle_size: 1,
        };

        utils::create_program_account(
            ctx.accounts.user,
            ctx.accounts.listing,
            program_id,
            listing_bump_seed,
            to_vec(&listing).unwrap().len(),
            vec![&ctx.accounts.item.key.to_bytes(), b"Listing"],
        )?;

        listing.serialize(&mut &mut ctx.accounts.listing.data.borrow_mut()[..])?;
    }

    let mut listing = state::Listing::try_from_slice(&ctx.accounts.listing.data.borrow()[..])?;

    //token
    if args.item_type == 1 {
        listing.quantity += args.quantity;
        listing.price = args.price;

        let mint_data = ctx.accounts.item.data.borrow();
        let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;

        accounts::check_token_account(
            ctx.accounts.pda,
            ctx.accounts.item,
            ctx.accounts.pda_item.unwrap(),
            ctx.accounts.listing_tp,
        )?;

        utils::create_ata(
            ctx.accounts.user,
            ctx.accounts.pda,
            ctx.accounts.item,
            ctx.accounts.pda_item.unwrap(),
            ctx.accounts.listing_tp,
        )?;

        utils::transfer_tokens(
            listing_2022,
            args.quantity,
            ctx.accounts.user_item.unwrap(),
            ctx.accounts.item,
            ctx.accounts.pda_item.unwrap(),
            ctx.accounts.user,
            ctx.accounts.listing_tp,
            pda_bump_seed,
            &vec![&accounts::PDA_SEED.to_le_bytes()],
            mint.base.decimals,
            &Vec::new(),
        )?;
    }

    //core asset
    if args.item_type == 2 {
        listing.quantity += 1;
        listing.price = args.price;
        let _transfer = TransferV1CpiBuilder::new(ctx.accounts.core)
            .asset(ctx.accounts.item)
            .authority(Some(ctx.accounts.user))
            .payer(ctx.accounts.user)
            .new_owner(ctx.accounts.pda)
            .collection(Some(ctx.accounts.collection.unwrap()))
            .invoke_signed(&[&[&accounts::PDA_SEED.to_le_bytes(), &[pda_bump_seed]]])?;
    }

    listing.serialize(&mut &mut ctx.accounts.listing.data.borrow_mut()[..])?;

    Ok(())
}
