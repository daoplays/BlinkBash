use mpl_core::instructions::{CreateCollectionV1CpiBuilder, CreateV1CpiBuilder};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::accounts;

use crate::instruction;
use crate::state;

pub fn set_attributes<'a>(
    user_account_info: &'a AccountInfo<'a>,
    sol_account_info: &'a AccountInfo<'a>,
    pda_sol_bump_seed: u8,
    system_program_account_info: &'a AccountInfo<'a>,
    core_account_info: &'a AccountInfo<'a>,
    nft_mint_account_info: &'a AccountInfo<'a>,
    collection_mint_account_info: &'a AccountInfo<'a>,
    randoms: [f64; 25],
    nft_index: u32,
) -> ProgramResult {
    let mut attribute_list: Vec<mpl_core::types::Attribute> = Vec::new();
    attribute_list.push(mpl_core::types::Attribute {
        key: "CookWrapIndex".to_string(),
        value: nft_index.to_string(),
    });

    // see if we need to generate some attributes
    let collection =
        mpl_core::Collection::from_bytes(&collection_mint_account_info.data.borrow()[..])?;

    let attributes_plugin = collection.plugin_list.attributes;

    if attributes_plugin.is_some() {
        let collection_attributes = attributes_plugin.unwrap().attributes.attribute_list;

        let n_attributes = collection_attributes.len() / 3;
        for i in 0..n_attributes {
            let min = collection_attributes[i * 3 + 1]
                .value
                .parse::<f64>()
                .unwrap();
            let max = collection_attributes[i * 3 + 2]
                .value
                .parse::<f64>()
                .unwrap();
            let value = ((max - min) * randoms[2 + i] + min) as u32;

            attribute_list.push(mpl_core::types::Attribute {
                key: collection_attributes[i * 3].value.to_string(),
                value: value.to_string(),
            });
        }
    }

    mpl_core::instructions::UpdatePluginV1CpiBuilder::new(core_account_info)
        .collection(Some(collection_mint_account_info))
        .asset(nft_mint_account_info)
        .payer(user_account_info)
        .authority(Some(sol_account_info))
        .plugin(mpl_core::types::Plugin::Attributes(
            mpl_core::types::Attributes {
                attribute_list: attribute_list,
            },
        ))
        .system_program(system_program_account_info)
        .invoke_signed(&[&[&accounts::PDA_SEED.to_le_bytes(), &[pda_sol_bump_seed]]])?;

    return Ok(());
}

pub fn mint_collection<'a>(
    funding_account: &'a AccountInfo<'a>,
    pda_account: &'a AccountInfo<'a>,
    pda_bump: u8,
    system_program_account_info: &'a AccountInfo<'a>,
    core_program_account_info: &'a AccountInfo<'a>,

    // collection accounts
    collection_account: &'a AccountInfo<'a>,

    // collection metadata details
    collection_config: state::CollectionDetails,
    attributes: Vec<instruction::Attribute>,
) -> ProgramResult {
    let _create_cpi = CreateCollectionV1CpiBuilder::new(core_program_account_info)
        .collection(collection_account)
        .payer(funding_account)
        .update_authority(Some(pda_account))
        .name(collection_config.name.to_string())
        .uri(collection_config.uri.to_string())
        .system_program(system_program_account_info)
        .invoke_signed(&[&[&collection_config.pda.to_le_bytes(), &[pda_bump]]])?;

    if attributes.len() > 0 {
        let mut attribute_list: Vec<mpl_core::types::Attribute> = Vec::new();
        for i in 0..attributes.len() {
            let mut name: String = "Name_".to_string().to_owned();
            name.push_str(&i.to_string().to_owned());
            attribute_list.push(mpl_core::types::Attribute {
                key: name,
                value: attributes[i].name.to_string(),
            });

            name = "Min_".to_string().to_owned();
            name.push_str(&i.to_string().to_owned());
            attribute_list.push(mpl_core::types::Attribute {
                key: name,
                value: attributes[i].min.to_string(),
            });

            name = "Max_".to_string().to_owned();
            name.push_str(&i.to_string().to_owned());
            attribute_list.push(mpl_core::types::Attribute {
                key: name,
                value: attributes[i].max.to_string(),
            });
        }

        mpl_core::instructions::AddCollectionPluginV1CpiBuilder::new(core_program_account_info)
            .collection(collection_account)
            .payer(funding_account)
            .authority(Some(pda_account))
            .plugin(mpl_core::types::Plugin::Attributes(
                mpl_core::types::Attributes {
                    attribute_list: attribute_list,
                },
            ))
            .system_program(system_program_account_info)
            .invoke_signed(&[&[&collection_config.pda.to_le_bytes(), &[pda_bump]]])?;
    }
    Ok(())
}

pub fn mint_collection_nft<'a>(
    funding_account: &'a AccountInfo<'a>,
    pda_account: &'a AccountInfo<'a>,
    pda_bump: u8,
    cook_pda_bump: u8,
    system_program_account_info: &'a AccountInfo<'a>,
    core_program_account_info: &'a AccountInfo<'a>,

    nft_mint_account: &'a AccountInfo<'a>,

    // collection accounts
    collection_mint_account: &'a AccountInfo<'a>,

    // collection metadata details
    collection_config: state::CollectionDetails,
) -> ProgramResult {
    msg!("create collection asset");
    let _create_cpi = CreateV1CpiBuilder::new(core_program_account_info)
        .authority(Some(pda_account))
        .asset(nft_mint_account)
        .collection(Some(collection_mint_account))
        .payer(funding_account)
        .owner(Some(pda_account))
        .data_state(mpl_core::types::DataState::AccountState)
        .name(collection_config.name)
        .uri(collection_config.uri)
        .system_program(system_program_account_info)
        .invoke_signed(&[
            &[
                &collection_mint_account.key.to_bytes(),
                &collection_config.index.to_le_bytes(),
                b"Asset",
                &[pda_bump],
            ],
            &[&collection_config.pda.to_le_bytes(), &[cook_pda_bump]],
        ])?;

    mpl_core::instructions::AddPluginV1CpiBuilder::new(core_program_account_info)
        .asset(nft_mint_account)
        .collection(Some(collection_mint_account))
        .payer(funding_account)
        .authority(Some(pda_account))
        .plugin(mpl_core::types::Plugin::Attributes(
            mpl_core::types::Attributes {
                attribute_list: vec![mpl_core::types::Attribute {
                    key: "CookWrapIndex".to_string(),
                    value: collection_config.index.to_string(),
                }],
            },
        ))
        .system_program(system_program_account_info)
        .invoke_signed(&[&[&collection_config.pda.to_le_bytes(), &[cook_pda_bump]]])
        .unwrap();

    Ok(())
}
