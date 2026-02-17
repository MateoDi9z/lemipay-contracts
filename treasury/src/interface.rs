//! Public interface of the Treasury contract.
//!
//! All function documentation lives here; the implementation is in `lib.rs`.

use crate::errors::Error;
use crate::types::{FundRound, ReleaseProposal};
use soroban_sdk::{contracttrait, Address, Env, Vec};

#[contracttrait]
pub trait ITreasuryContract {
    /// Creates a treasury for a group and initializes its balance to zero.
    ///
    /// The caller must be a member of the group. Each `group_id` can have at most one treasury.
    ///
    /// # Arguments
    /// * `group_id` - Identifier of the group (must exist in GroupContract).
    /// * `user` - Address of the creator; must be authorized and a member of the group.
    ///
    /// # Errors
    /// * [`Error::NotMember`] - `user` is not a member of the group.
    /// * [`Error::GroupIdAlreadyInUse`] - A treasury for this `group_id` already exists.
    fn create_treasury(env: Env, group_id: u64, user: Address) -> Result<(), Error>;

    /// Proposes a release of USDC from the group's balance to a destination address.
    ///
    /// The proposal must later be approved by enough group members (see GroupContract approval
    /// rule) and then executed via [`execute_release`](Self::execute_release). The group's balance must be at least
    /// `amount` at proposal time to avoid creating unexecutable proposals.
    ///
    /// # Arguments
    /// * `destination` - Address that will receive the USDC; must be a member of the group.
    /// * `amount` - Amount to release (must be positive and ≤ current group balance).
    /// * `group_id` - Group whose treasury and balance are used.
    /// * `user` - Proposer; must be authorized and a member of the group.
    ///
    /// # Returns
    /// The new release proposal id (opaque u64 for use with [`approve_release`](Self::approve_release), [`execute_release`](Self::execute_release), [`cancel_release_proposal`](Self::cancel_release_proposal)).
    ///
    /// # Errors
    /// * [`Error::NotMember`] - `user` or `destination` not in group.
    /// * [`Error::TreasuryNotInitialized`] - No treasury for this group.
    /// * [`Error::InvalidAmount`] - `amount` ≤ 0.
    /// * [`Error::InsufficientGroupBalance`] - Group balance &lt; `amount`.
    fn propose_release(
        env: Env,
        destination: Address,
        amount: i128,
        group_id: u64,
        user: Address,
    ) -> Result<u64, Error>;

    /// Records an approval for a release proposal from a group member.
    ///
    /// The destination of the release cannot approve. Once the number of approvals reaches the
    /// group's approval rule (from GroupContract), the proposal can be executed.
    ///
    /// # Arguments
    /// * `release_proposal_id` - Id returned by [`propose_release`](Self::propose_release).
    /// * `user` - Approver; must be authorized and a member of the proposal's group (and not the destination).
    ///
    /// # Errors
    /// * [`Error::ProposalNotFound`] - No proposal with this id.
    /// * [`Error::AlreadyExecuted`] - Proposal was already executed.
    /// * [`Error::ProposalCanceled`] - Proposal was canceled.
    /// * [`Error::DestinationCannotApprove`] - `user` is the release destination.
    /// * [`Error::NotMember`] - `user` not in group.
    /// * [`Error::AlreadyApproved`] - This user already approved.
    fn approve_release(
        env: Env,
        release_proposal_id: u64,
        user: Address,
    ) -> Result<(), Error>;

    /// Executes a release proposal: subtracts the amount from the group balance and transfers USDC to the destination.
    ///
    /// Requires that the proposal has enough approvals (per the group's approval rule), has not been
    /// executed or canceled, and that both the group balance and the contract's token balance are sufficient.
    ///
    /// # Arguments
    /// * `release_proposal_id` - Id of the proposal to execute.
    ///
    /// # Errors
    /// * [`Error::ProposalNotFound`] - No proposal with this id.
    /// * [`Error::AlreadyExecuted`] - Proposal already executed.
    /// * [`Error::ProposalCanceled`] - Proposal was canceled.
    /// * [`Error::NotEnoughApprovals`] - Approval count below group rule.
    /// * [`Error::InsufficientGroupBalance`] - Group balance &lt; release amount.
    /// * [`Error::InsufficientTreasuryBalance`] - Contract token balance &lt; release amount.
    fn execute_release(env: Env, release_proposal_id: u64) -> Result<(), Error>;

    /// Cancels a release proposal that has not been executed.
    ///
    /// Any member of the proposal's group can cancel it. After cancellation, [`approve_release`](Self::approve_release)
    /// and [`execute_release`](Self::execute_release) will return [`Error::ProposalCanceled`] for this proposal.
    ///
    /// # Arguments
    /// * `release_proposal_id` - Id of the proposal to cancel.
    /// * `user` - Caller; must be authorized and a member of the proposal's group.
    ///
    /// # Errors
    /// * [`Error::ProposalNotFound`] - No proposal with this id.
    /// * [`Error::AlreadyExecuted`] - Proposal already executed.
    /// * [`Error::ProposalCanceled`] - Proposal was already canceled.
    /// * [`Error::NotMember`] - `user` not in group.
    fn cancel_release_proposal(
        env: Env,
        release_proposal_id: u64,
        user: Address,
    ) -> Result<(), Error>;

    /// Proposes a new fund round for a group: a target amount members can contribute to.
    ///
    /// The round is identified by the returned `round_id`. Members then use [`contribute_to_fund_round`](Self::contribute_to_fund_round)
    /// until the round is fully funded or they withdraw via [`withdraw_contribution`](Self::withdraw_contribution).
    ///
    /// # Arguments
    /// * `group_id` - Group that owns the treasury and the round.
    /// * `total_amount` - Target amount for the round (must be positive).
    /// * `user` - Proposer; must be authorized and a member of the group.
    ///
    /// # Returns
    /// The new fund round id (for [`contribute_to_fund_round`](Self::contribute_to_fund_round), [`withdraw_contribution`](Self::withdraw_contribution), [`get_fund_round`](Self::get_fund_round)).
    ///
    /// # Errors
    /// * [`Error::NotMember`] - `user` not in group.
    /// * [`Error::TreasuryNotInitialized`] - No treasury for this group.
    /// * [`Error::InvalidAmount`] - `total_amount` ≤ 0.
    /// * [`Error::NoMembers`] - Group has no members.
    /// * [`Error::TooManyMembers`] - Member count overflow for internal storage.
    fn propose_fund_round(
        env: Env,
        group_id: u64,
        total_amount: i128,
        user: Address,
    ) -> Result<u64, Error>;

    /// Deposits USDC into the treasury and credits the group's balance for the given fund round.
    ///
    /// This is the **only supported way** to send USDC to the contract; direct transfers are not
    /// reflected in any GroupBalance and cannot be recovered. See crate-level documentation.
    ///
    /// USDC is taken from `user` via the token's `transfer_from` (caller must have set allowance).
    /// The round's `funded_amount` and the group balance are increased; if `funded_amount` reaches
    /// `total_amount`, the round is marked completed.
    ///
    /// # Arguments
    /// * `round_id` - Id of the fund round (from [`propose_fund_round`](Self::propose_fund_round)).
    /// * `amount` - Amount to contribute (must be positive and ≤ remaining to reach round target).
    /// * `user` - Contributor; must be authorized and a member of the round's group.
    ///
    /// # Errors
    /// * [`Error::InvalidAmount`] - `amount` ≤ 0.
    /// * [`Error::RoundNotFound`] - No round with this id.
    /// * [`Error::TreasuryNotInitialized`] - No treasury for the round's group.
    /// * [`Error::NotMember`] - `user` not in group.
    /// * [`Error::RoundAlreadyCompleted`] - Round already reached its target.
    /// * [`Error::ContributionExceedsRemaining`] - `amount` would exceed remaining to target.
    /// * [`Error::BalanceOverflow`] / [`Error::ContributionOverflow`] - Arithmetic overflow.
    fn contribute_to_fund_round(
        env: Env,
        round_id: u64,
        amount: i128,
        user: Address,
    ) -> Result<(), Error>;

    /// Withdraws the caller's contribution from an **incomplete** fund round.
    ///
    /// Refunds USDC to the user and decreases the round's `funded_amount` and the group balance.
    /// Only the contributor can withdraw their own funds; the round must not be completed.
    ///
    /// # Arguments
    /// * `round_id` - Id of the fund round.
    /// * `user` - Caller; must be authorized, a member of the round's group, and have a positive contribution.
    ///
    /// # Errors
    /// * [`Error::RoundNotFound`] - No round with this id.
    /// * [`Error::RoundAlreadyCompleted`] - Round already completed (withdrawals not allowed).
    /// * [`Error::TreasuryNotInitialized`] - No treasury for the round's group.
    /// * [`Error::NotMember`] - `user` not in group.
    /// * [`Error::NoContributionToWithdraw`] - This user has no contribution in this round.
    /// * [`Error::InsufficientTreasuryBalance`] - Contract does not hold enough USDC to refund (should not happen if only contribute/withdraw/release are used).
    fn withdraw_contribution(
        env: Env,
        round_id: u64,
        user: Address,
    ) -> Result<(), Error>;

    /// Returns the list of fund round ids for a group (order of creation).
    fn get_group_rounds(env: Env, group_id: u64) -> Vec<u64>;

    /// Returns the fund round data for a given round id.
    ///
    /// # Errors
    /// * [`Error::RoundNotFound`] - No round with this id.
    fn get_fund_round(env: Env, round_id: u64) -> Result<FundRound, Error>;

    /// Returns the total amount a user has contributed to a given fund round (0 if none).
    fn get_user_contribution(env: Env, round_id: u64, user: Address) -> i128;

    /// Returns the list of release proposal ids for a group (order of creation).
    fn get_release_proposals_of_group(env: Env, group_id: u64) -> Vec<u64>;

    /// Returns the release proposal data for a given proposal id.
    ///
    /// # Errors
    /// * [`Error::ProposalNotFound`] - No proposal with this id.
    fn get_release_proposal(env: Env, proposal_id: u64) -> Result<ReleaseProposal, Error>;

    /// Returns whether a treasury exists for the given group.
    fn check_treasury_id(env: Env, group_id: u64) -> bool;

    /// Returns the current balance (in base units) allocated to the group (0 if no treasury or no contributions/releases yet).
    fn get_group_balance(env: Env, group_id: u64) -> i128;

    /// Returns true if the group's balance is greater than or equal to `amount`.
    fn has_sufficient_group_balance(env: Env, group_id: u64, amount: i128) -> bool;
}
