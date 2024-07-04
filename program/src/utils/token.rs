use solana_program::{
    account_info::AccountInfo,
    borsh1::get_instance_packed_len,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    sysvar::rent,
};
use spl_pod::optional_keys::OptionalNonZeroPubkey;

use crate::{accounts, state};

pub fn create_2022_token<'a>(
    funding_account: &'a AccountInfo<'a>,
    pda_account: &'a AccountInfo<'a>,
    token_program: &'a AccountInfo<'a>,
    pda_bump: u8,

    // nft accounts
    nft_mint_account: &'a AccountInfo<'a>,
    token_config: state::TokenDetails,
) -> ProgramResult {
    let mut extension_types: Vec<spl_token_2022::extension::ExtensionType> = Vec::new();

    extension_types.push(spl_token_2022::extension::ExtensionType::MetadataPointer);

    let token_metadata = spl_token_metadata_interface::state::TokenMetadata {
        name: token_config.name,
        symbol: token_config.symbol,
        uri: token_config.uri,
        update_authority: OptionalNonZeroPubkey(*pda_account.key),
        mint: *nft_mint_account.key,
        ..Default::default()
    };

    let instance_size = get_instance_packed_len(&token_metadata)?;

    // first create the mint account for the new NFT

    let space = spl_token_2022::extension::ExtensionType::try_calculate_account_len::<
        spl_token_2022::state::Mint,
    >(&extension_types)
    .unwrap();
    // first create the mint account for the new NFT
    let mint_rent = rent::Rent::default().minimum_balance(space + instance_size + 8);

    let ix = solana_program::system_instruction::create_account(
        funding_account.key,
        nft_mint_account.key,
        mint_rent,
        space as u64,
        token_program.key,
    );

    msg!("create account");
    // Sign and submit transaction
    invoke(&ix, &[funding_account.clone(), nft_mint_account.clone()])?;

    let metadata_config_init_idx =
        spl_token_2022::extension::metadata_pointer::instruction::initialize(
            &spl_token_2022::ID,
            &nft_mint_account.key,
            None,
            Some(*nft_mint_account.key),
        )
        .unwrap();

    invoke(
        &metadata_config_init_idx,
        &[
            token_program.clone(),
            nft_mint_account.clone(),
            funding_account.clone(),
        ],
    )?;

    // initialize the mint, mint and freeze authority will be with the pda
    let mint_idx = spl_token_2022::instruction::initialize_mint2(
        token_program.key,
        nft_mint_account.key,
        pda_account.key,
        None,
        token_config.decimals,
    )
    .unwrap();

    msg!("init mint");
    // Sign and submit transaction
    invoke(
        &mint_idx,
        &[
            token_program.clone(),
            nft_mint_account.clone(),
            funding_account.clone(),
        ],
    )?;

    // now actually set the metadata
    invoke_signed(
        &spl_token_metadata_interface::instruction::initialize(
            &spl_token_2022::id(),
            nft_mint_account.key,
            pda_account.key,
            nft_mint_account.key,
            pda_account.key,
            token_metadata.name.to_string(),
            token_metadata.symbol.to_string(),
            token_metadata.uri.to_string(),
        ),
        &[nft_mint_account.clone(), pda_account.clone()],
        &[&[&accounts::PDA_SEED.to_le_bytes(), &[pda_bump]]],
    )?;

    Ok(())
}
