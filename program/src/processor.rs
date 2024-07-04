use crate::instructions;
use borsh::BorshDeserialize;

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::instruction::BlinkInstruction;

pub struct Processor;
impl Processor {
    pub fn process<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = BlinkInstruction::try_from_slice(instruction_data)?;

        match instruction {
            BlinkInstruction::Init() => {
                msg!("Init");
                instructions::init(program_id, accounts)
            }
            BlinkInstruction::Enter(args) => {
                msg!("Enter");
                instructions::enter(program_id, accounts, args)
            }
            BlinkInstruction::Vote(args) => {
                msg!("Vote");
                instructions::vote(program_id, accounts, args)
            }
            BlinkInstruction::ClaimPrize(args) => {
                msg!("ClaimPrize");
                instructions::claim_prize(program_id, accounts, args)
            }
            BlinkInstruction::ListItem(args) => {
                msg!("ListItem");
                instructions::list_item(program_id, accounts, args)
            }
            BlinkInstruction::PurchaseItem(args) => {
                msg!("PurchaseItem");
                instructions::purchase_item(program_id, accounts, args)
            }
        }
    }
}
