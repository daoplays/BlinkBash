use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct CreateMeta {
    pub name: String,
    pub uri: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct EnterMeta {
    pub game: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct VoteMeta {
    pub game: u8,
    pub vote: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct ListMeta {
    pub item_type: u8,
    pub quantity: u64,
    pub price: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct PurchaseMeta {
    pub quantity: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct ClaimPrizeMeta {
    pub game: u8,
    pub date: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub min: String,
    pub max: String,
}

#[derive(
    ShankInstruction, ShankContext, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq,
)]
pub enum BlinkInstruction {
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "pda", desc = "pda account")]
    #[account(2, writable, name = "data", desc = "data account")]
    #[account(3, writable, name = "token_mint", desc = "token mint account")]
    #[account(4, name = "system_program", desc = "System program")]
    #[account(5, name = "token_2022", desc = "token program")]
    Init(),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "pda", desc = "pda account")]
    #[account(2, writable, name = "data", desc = "data account")]
    #[account(3, writable, name = "entry", desc = "entry account")]
    #[account(4, writable, name = "user_data", desc = "user data account")]
    #[account(5, writable, name = "bash_mint", desc = "user data account")]
    #[account(6, writable, name = "user_token", desc = "user data account")]
    #[account(7, writable, name = "leaderboard", desc = "leaderboard account")]
    #[account(8, name = "system_program", desc = "System program")]
    #[account(9, name = "token_2022", desc = "System program")]
    #[account(10, name = "associated", desc = "System program")]
    #[account(11, optional, name = "reference", desc = "ref user")]
    #[account(12, optional, writable, name = "ref_bash", desc = "ref bash")]
    Enter(EnterMeta),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "pda", desc = "pda account")]
    #[account(2, writable, name = "data", desc = "data account")]
    #[account(3, writable, name = "entry", desc = "entry account")]
    #[account(4, writable, name = "user_data", desc = "user data account")]
    #[account(5, writable, name = "creator", desc = "creator account")]
    #[account(6, writable, name = "creator_data", desc = "creator data account")]
    #[account(7, writable, name = "leaderboard", desc = "leaderboard account")]
    #[account(8, writable, name = "bash_mint", desc = "user data account")]
    #[account(9, writable, name = "user_token", desc = "user data account")]
    #[account(10, name = "system_program", desc = "System program")]
    #[account(11, name = "token_2022", desc = "System program")]
    #[account(12, name = "associated", desc = "System program")]
    #[account(13, optional, name = "reference", desc = "ref user")]
    #[account(14, optional, writable, name = "ref_bash", desc = "ref bash")]
    Vote(VoteMeta),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "pda", desc = "pda account")]
    #[account(2, writable, name = "entry", desc = "entry account")]
    #[account(3, writable, name = "user_data", desc = "user data account")]
    #[account(4, writable, name = "leaderboard", desc = "leaderboard account")]
    #[account(5, writable, name = "bash_mint", desc = "user data account")]
    #[account(6, writable, name = "user_token", desc = "user data account")]
    #[account(7, name = "system_program", desc = "System program")]
    #[account(8, name = "token_2022", desc = "System program")]
    #[account(9, name = "associated", desc = "System program")]
    ClaimPrize(ClaimPrizeMeta),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "pda", desc = "pda account")]
    #[account(2, writable, name = "whitelist_mint", desc = "whitelist token")]
    #[account(3, writable, name = "whitelist_account", desc = "whitelist token")]
    #[account(4, writable, name = "item", desc = "item account")]
    #[account(5, writable, name = "listing", desc = "item account")]
    #[account(6, optional, writable, name = "pda_item", desc = "item account")]
    #[account(7, optional, writable, name = "user_item", desc = "item account")]
    #[account(8, optional, writable, name = "collection", desc = "item account")]
    #[account(9, name = "system_program", desc = "System program")]
    #[account(10, name = "core", desc = "Core program")]
    #[account(11, name = "token_2022", desc = "Token 2022 program")]
    #[account(12, name = "associated", desc = "Token 2022 program")]
    ListItem(ListMeta),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "pda", desc = "pda account")]
    #[account(2, writable, name = "item", desc = "item account")]
    #[account(3, writable, name = "listing", desc = "item account")]
    #[account(4, writable, name = "pda_item", desc = "item account")]
    #[account(5, writable, name = "user_item", desc = "item account")]
    #[account(6, writable, name = "collection", desc = "item account")]
    #[account(7, writable, name = "bash_mint", desc = "item account")]
    #[account(8, writable, name = "user_bash", desc = "item account")]
    #[account(9, name = "system_program", desc = "System program")]
    #[account(10, name = "core", desc = "Core program")]
    #[account(11, name = "token_2022", desc = "Token 2022 program")]
    #[account(12, name = "associated", desc = "Token 2022 program")]
    PurchaseItem(PurchaseMeta),
}
