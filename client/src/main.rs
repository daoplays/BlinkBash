pub mod state;

use crate::state::BlinkBashInstruction;
use std::env;
use std::fs::read;
use std::str::{from_utf8, FromStr};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_program::pubkey::Pubkey;
use solana_sdk::borsh1::get_instance_packed_len;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::sysvar::rent;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::Keypair,
    signer::keypair::read_keypair_file,
    signer::Signer,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_pod::optional_keys::OptionalNonZeroPubkey;

const URL: &str = "https://api.mainnet-beta.solana.com";
//const URL: &str = "https://api.devnet.solana.com";

const PROGRAM_PUBKEY: &str = "BASHv2NgqzdjKni4Rp7PxM2EzKZPSVGHCkC92ZfNZis3";

fn main() {
    let args: Vec<String> = env::args().collect();
    let key_file = &args[1];
    let function = &args[2];

    if function == "init" {
        if let Err(err) = init(key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    if function == "wlist" {
        if let Err(err) = create_whitelist(key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }
}

pub fn init(key_file: &String) -> std::result::Result<(), state::Error> {
    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();
    let token = read_keypair_file("token.json").unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_PUBKEY).unwrap();

    let pda_seed: u32 = 6968193;
    let data_seed: u32 = 10399637;

    let (expected_pda_account, _pda_bump_seed) =
        Pubkey::find_program_address(&[&pda_seed.to_le_bytes()], &program);

    let (expected_data_account, _data_bump_seed) =
        Pubkey::find_program_address(&[&data_seed.to_le_bytes()], &program);

    let instruction = Instruction::new_with_borsh(
        program,
        &BlinkBashInstruction::Init,
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(expected_pda_account, false),
            AccountMeta::new(expected_data_account, false),
            AccountMeta::new(token.pubkey(), true),
            AccountMeta::new(solana_sdk::system_program::id(), false),
            AccountMeta::new(spl_token_2022::id(), false),
        ],
    );

    let signers = [&wallet, &token];
    let instructions = vec![instruction];
    let recent_hash = connection.get_latest_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet.pubkey()),
        &signers,
        recent_hash,
    );

    let signature = connection.send_and_confirm_transaction_with_spinner_and_config(
        &txn,
        CommitmentConfig {
            commitment: CommitmentLevel::Confirmed,
        },
        RpcSendTransactionConfig {
            skip_preflight: true,
            preflight_commitment: None,
            encoding: None,
            max_retries: None,
            min_context_slot: None,
        },
    )?;
    println!("signature: {}", signature);
    let response = connection.get_transaction(&signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response);

    Ok(())
}

pub fn create_whitelist(key_file: &String) -> std::result::Result<(), state::Error> {
    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let wallet = read_keypair_file(key_file).unwrap();
    let token = read_keypair_file("whitelist.json").unwrap();

    let my_token_address = get_associated_token_address_with_program_id(
        &wallet.pubkey(),
        &token.pubkey(),
        &spl_token_2022::id(),
    );

    println!("whitelist mint {:?}", token.pubkey().to_string());
    println!("token address {:?}", my_token_address.to_string());

    // first create the mint account for the new NFT
    let mut extension_types: Vec<spl_token_2022::extension::ExtensionType> = Vec::new();

    extension_types.push(spl_token_2022::extension::ExtensionType::MetadataPointer);

    let token_metadata = spl_token_metadata_interface::state::TokenMetadata {
        name: "BlinkBash Whitelist".to_string(),
        symbol: "$BASH_W".to_string(),
        uri: "https://gateway.irys.xyz/vs4hWL4X9EXlsdyfwQNslW_1GQWc1wn1ZsVIdSECZoI".to_string(),
        update_authority: OptionalNonZeroPubkey(wallet.pubkey()),
        mint: token.pubkey(),
        ..Default::default()
    };

    let instance_size = get_instance_packed_len(&token_metadata).unwrap();

    // first create the mint account for the new NFT

    let space = spl_token_2022::extension::ExtensionType::try_calculate_account_len::<
        spl_token_2022::state::Mint,
    >(&extension_types)
    .unwrap();
    // first create the mint account for the new NFT
    let mint_rent = rent::Rent::default().minimum_balance(space + instance_size + 8);

    let create_idx = solana_program::system_instruction::create_account(
        &wallet.pubkey(),
        &token.pubkey(),
        mint_rent,
        space as u64,
        &spl_token_2022::id(),
    );

    let metadata_config_init_idx =
        spl_token_2022::extension::metadata_pointer::instruction::initialize(
            &spl_token_2022::ID,
            &token.pubkey(),
            None,
            Some(token.pubkey()),
        )
        .unwrap();

    let mint_idx = spl_token_2022::instruction::initialize_mint2(
        &spl_token_2022::id(),
        &token.pubkey(),
        &wallet.pubkey(),
        None,
        0,
    )
    .unwrap();

    let meta_idx = spl_token_metadata_interface::instruction::initialize(
        &spl_token_2022::id(),
        &token.pubkey(),
        &wallet.pubkey(),
        &token.pubkey(),
        &wallet.pubkey(),
        token_metadata.name.to_string(),
        token_metadata.symbol.to_string(),
        token_metadata.uri.to_string(),
    );

    let create_ata_idx = create_associated_token_account(
        &wallet.pubkey(),
        &wallet.pubkey(),
        &token.pubkey(),
        &spl_token_2022::id(),
    );

    let mint_to_idx = spl_token_2022::instruction::mint_to_checked(
        &spl_token_2022::id(),
        &token.pubkey(),
        &my_token_address,
        &wallet.pubkey(),
        &[&wallet.pubkey()],
        100000,
        0,
    )
    .unwrap();

    let txn = Transaction::new_signed_with_payer(
        &vec![
            create_idx,
            metadata_config_init_idx,
            mint_idx,
            meta_idx,
            create_ata_idx,
            mint_to_idx,
        ],
        Some(&wallet.pubkey()),
        &[&wallet, &token],
        connection.get_latest_blockhash()?,
    );
    let signature = connection.send_and_confirm_transaction(&txn)?;

    println!("signature: {}", signature);
    let response = connection.get_transaction(&signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response);

    //spl-token mint 4JxGUVRp6CRffKpbtnSCZ4Z5dHqUWMZSxMuvFd7fG3nC 500 --mint-authority ~/Documents/Crypto/Solana/paper_wallets/phantom_2.json
    //spl-token transfer 4JxGUVRp6CRffKpbtnSCZ4Z5dHqUWMZSxMuvFd7fG3nC 500 7oAfRLy81EwMJAXNKbZFaMTayBFoBpkua4ukWiCZBZz5
    Ok(())
}
