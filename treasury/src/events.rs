//! Contract events for observability, indexación y frontends.

use soroban_sdk::{contractevent, Address};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryCreated {
    #[topic]
    pub group_id: u64,
    pub creator: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseProposed {
    #[topic]
    pub proposal_id: u64,
    #[topic]
    pub group_id: u64,
    pub destination: Address,
    pub amount: i128,
    pub proposer: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseApproved {
    #[topic]
    pub proposal_id: u64,
    #[topic]
    pub group_id: u64,
    pub approver: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseExecuted {
    #[topic]
    pub proposal_id: u64,
    #[topic]
    pub group_id: u64,
    pub destination: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundRoundProposed {
    #[topic]
    pub round_id: u64,
    #[topic]
    pub group_id: u64,
    pub total_amount: i128,
    pub proposer: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundRoundCompleted {
    #[topic]
    pub round_id: u64,
    #[topic]
    pub group_id: u64,
    pub total_amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contribution {
    #[topic]
    pub round_id: u64,
    #[topic]
    pub group_id: u64,
    pub user: Address,
    pub amount: i128,
    pub new_funded_amount: i128,
}
