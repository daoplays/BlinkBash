use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Default, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum AccountType {
    #[default]
    Program,
    User,
    Entry,
    Leaderboard,
    Listing,
}
pub struct TokenDetails {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
}

pub struct CollectionDetails {
    pub name: String,
    pub index: u32,
    pub uri: String,
    pub pda: u32,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub struct ProgramStats {
    pub account_type: AccountType,
    pub num_users: u32,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub struct User {
    pub account_type: AccountType,
    pub user_key: Pubkey,
    pub user_id: u32,
    pub twitter: String,
    pub total_wins: u32,
    pub total_positive_votes: u32,
    pub total_negative_votes: u32,
    pub total_positive_voted: u32,
    pub total_negative_voted: u32,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub struct Leaderboard {
    pub account_type: AccountType,
    pub game: u8,
    pub date: u32,
    pub entrants: Vec<u32>,
    pub scores: Vec<u32>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub struct Entry {
    pub account_type: AccountType,
    pub positive_votes: u32,
    pub negative_votes: u32,
    pub reward_claimed: u8,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub struct Listing {
    pub account_type: AccountType,
    pub item_type: u8,
    pub item_address: Pubkey,
    pub price: u64,
    pub quantity: u64,
    pub bundle_size: u64,
}
