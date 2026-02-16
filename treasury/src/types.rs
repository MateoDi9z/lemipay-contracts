//! Contract data types.

use soroban_sdk::{contracttype, Address};

#[derive(Clone)]
#[contracttype]
pub struct ReleaseProposal {
    pub group_id: u64,
    pub destination: Address,
    pub amount: i128,
    pub approvals: u32,
    pub executed: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct FundRound {
    pub group_id: u64,
    pub total_amount: i128,
    pub amount_of_members: u32,
    pub funded_amount: i128,
    pub completed: bool,
}
