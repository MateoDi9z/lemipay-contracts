//! Storage keys for the Treasury contract.

use soroban_sdk::{contracttype, Address};

#[derive(Clone)]
#[contracttype]
pub(crate) enum DataKey {
    GroupId(u64),
    GroupBalance(u64),

    // Release proposal related keys
    ReleaseProposalCount,
    ReleaseProposal(u64),
    GroupReleaseProposals(u64),
    ReleaseApproval(u64, Address),

    // Fund round related keys
    FundRoundCount,
    FundRound(u64),
    GroupFundRounds(u64),
    FundContribution(u64, Address),
}
