//! Custom errors for the Treasury contract.
//!
//! Used with `Result<T, Error>` in public functions; contract clients get `try_*` variants.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotMember = 1,
    TreasuryNotInitialized = 2,
    GroupIdAlreadyInUse = 3,
    InvalidAmount = 4,
    ProposalNotFound = 5,
    AlreadyExecuted = 6,
    DestinationCannotApprove = 7,
    AlreadyApproved = 8,
    NotEnoughApprovals = 9,
    InsufficientTreasuryBalance = 10,
    GroupBalanceNotFound = 11,
    InsufficientGroupBalance = 12,
    BalanceUnderflow = 13,
    NoMembers = 14,
    RoundNotFound = 15,
    RoundAlreadyCompleted = 16,
    FundedExceedsTotal = 17,
    ContributionExceedsRemaining = 18,
    BalanceOverflow = 19,
    ContributionOverflow = 20,
    TooManyMembers = 21,
    NoContributionToWithdraw = 22,
    ProposalCanceled = 23,
}
