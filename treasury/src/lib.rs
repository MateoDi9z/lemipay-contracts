//! # Treasury contract
//!
//! Group-based treasury: fund rounds, release proposals, per-group balance.
//!
//! ## ⚠️ USDC inflow — único flujo soportado
//!
//! **El único flujo de entrada de USDC al contrato es [`contribute_to_fund_round`].**
//!
//! - No se debe hacer **transfer directo** de USDC al contrato (desde billetera o otro contrato).
//! - En Soroban, una transferencia directa la ejecuta solo el contrato del token; este contrato
//!   **no se invoca** en ese caso, por lo que no es posible rechazar ni hacer panic ante un depósito
//!   directo desde el código del Treasury.
//! - Cualquier USDC enviado por transfer directo **no** se refleja en ningún `GroupBalance`, no se
//!   asigna a ningún grupo y con la lógica actual **no hay forma de asignarlo después**. Ese saldo
//!   queda en el contrato sin uso (execute_release sigue comprobando balance del token y del grupo,
//!   así que no se puede “robar”, pero el saldo directo queda inutilizable).
//!
//! Integradores y frontends deben asegurar que el único depósito sea vía `contribute_to_fund_round`.

#![no_std]

mod config;
mod clients;
mod errors;
mod events;
mod helpers;
mod interface;
mod storage;
#[cfg(test)]
mod tests;
mod types;

pub use crate::errors::Error;
pub use crate::interface::ITreasuryContractClient as TreasuryContractClient;
pub use crate::types::{FundRound, ReleaseProposal};

#[cfg(not(test))]
use crate::clients::GroupContract;
use crate::events::{Contribution, ContributionWithdrawn, FundRoundCompleted, FundRoundProposed,
    ReleaseApproved, ReleaseCanceled, ReleaseExecuted, ReleaseProposed, TreasuryCreated};
use crate::interface::ITreasuryContract;
use crate::storage::DataKey;
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};

#[cfg(not(test))]
use soroban_sdk::token::Client as TokenClient;

#[contract]
pub struct TreasuryContract;

#[contractimpl(contracttrait)]
impl ITreasuryContract for TreasuryContract {
    fn create_treasury(
        env: Env,
        group_id: u64,
        user: Address,
    ) -> Result<(), Error> {
        user.require_auth();
        helpers::check_membership(&env, group_id, user.clone())?;

        if env.storage().persistent().has(&DataKey::GroupId(group_id)) {
            return Err(Error::GroupIdAlreadyInUse);
        }

        env.storage().persistent().set(&DataKey::GroupId(group_id), &true);
        env.storage()
            .persistent()
            .set(&DataKey::GroupBalance(group_id), &0i128);

        TreasuryCreated {
            group_id,
            creator: user,
        }
        .publish(&env);

        Ok(())
    }

    fn propose_release(
        env: Env,
        destination: Address,
        amount: i128,
        group_id: u64,
        user: Address,
    ) -> Result<u64, Error> {
        user.require_auth();
        helpers::check_membership(&env, group_id, user.clone())?;
        helpers::check_membership(&env, group_id, destination.clone())?;
        helpers::assert_treasury_exists(&env, group_id)?;

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let group_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::GroupBalance(group_id))
            .unwrap_or(0);
        if group_balance < amount {
            return Err(Error::InsufficientGroupBalance);
        }

        let mut count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::ReleaseProposalCount)
            .unwrap_or(0);

        count += 1;

        let proposal = ReleaseProposal {
            group_id,
            destination: destination.clone(),
            amount,
            approvals: 0,
            executed: false,
        };

        let mut group_proposals: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::GroupReleaseProposals(group_id))
            .unwrap_or(Vec::new(&env));

        group_proposals.push_back(count);

        env.storage()
            .persistent()
            .set(&DataKey::GroupReleaseProposals(group_id), &group_proposals);

        env.storage()
            .persistent()
            .set(&DataKey::ReleaseProposal(count), &proposal);
        env.storage()
            .persistent()
            .set(&DataKey::ReleaseProposalCount, &count);

        ReleaseProposed {
            proposal_id: count,
            group_id,
            destination,
            amount,
            proposer: user,
        }
        .publish(&env);

        Ok(count)
    }

    fn approve_release(
        env: Env,
        release_proposal_id: u64,
        user: Address,
    ) -> Result<(), Error> {
        user.require_auth();

        let release: ReleaseProposal = env
            .storage()
            .persistent()
            .get(&DataKey::ReleaseProposal(release_proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        if release.executed {
            return Err(Error::AlreadyExecuted);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::ReleaseProposalCanceled(release_proposal_id))
        {
            return Err(Error::ProposalCanceled);
        }

        if release.destination == user {
            return Err(Error::DestinationCannotApprove);
        }

        helpers::check_membership(&env, release.group_id, user.clone())?;

        let approval_key = DataKey::ReleaseApproval(release_proposal_id, user.clone());

        if env.storage().persistent().has(&approval_key) {
            return Err(Error::AlreadyApproved);
        }

        env.storage().persistent().set(&approval_key, &true);

        let mut updated = release.clone();
        updated.approvals += 1;

        env.storage()
            .persistent()
            .set(&DataKey::ReleaseProposal(release_proposal_id), &updated);

        ReleaseApproved {
            proposal_id: release_proposal_id,
            group_id: release.group_id,
            approver: user,
        }
        .publish(&env);

        Ok(())
    }

    fn execute_release(env: Env, release_proposal_id: u64) -> Result<(), Error> {
        let release: ReleaseProposal = env
            .storage()
            .persistent()
            .get(&DataKey::ReleaseProposal(release_proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        if release.executed {
            return Err(Error::AlreadyExecuted);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::ReleaseProposalCanceled(release_proposal_id))
        {
            return Err(Error::ProposalCanceled);
        }

        #[cfg(test)]
        let min: u32 = 1;

        #[cfg(not(test))]
        let min: u32 = {
            let group_contract = Address::from_str(&env, config::GROUP_CONTRACT);
            let client = GroupContract::new(&env, &group_contract);
            client.get_approval_rule(&release.group_id)
        };

        if release.approvals < min {
            return Err(Error::NotEnoughApprovals);
        }

        let group_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::GroupBalance(release.group_id))
            .ok_or(Error::GroupBalanceNotFound)?;

        if group_balance < release.amount {
            return Err(Error::InsufficientGroupBalance);
        }

        let new_balance = group_balance
            .checked_sub(release.amount)
            .ok_or(Error::BalanceUnderflow)?;
        env.storage()
            .persistent()
            .set(&DataKey::GroupBalance(release.group_id), &new_balance);

        #[cfg(not(test))]
        {
            let usdc_address = Address::from_str(&env, config::USDC_ADDRESS);
            let token = TokenClient::new(&env, &usdc_address);

            let treasury_address = env.current_contract_address();

            if token.balance(&treasury_address) < release.amount {
                return Err(Error::InsufficientTreasuryBalance);
            }

            token.transfer(
                &treasury_address,
                &release.destination,
                &release.amount,
            );
        }

        let mut updated = release.clone();
        updated.executed = true;
        env.storage()
            .persistent()
            .set(&DataKey::ReleaseProposal(release_proposal_id), &updated);

        ReleaseExecuted {
            proposal_id: release_proposal_id,
            group_id: release.group_id,
            destination: release.destination,
            amount: release.amount,
        }
        .publish(&env);

        Ok(())
    }

    fn cancel_release_proposal(
        env: Env,
        release_proposal_id: u64,
        user: Address,
    ) -> Result<(), Error> {
        user.require_auth();

        let release: ReleaseProposal = env
            .storage()
            .persistent()
            .get(&DataKey::ReleaseProposal(release_proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        if release.executed {
            return Err(Error::AlreadyExecuted);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::ReleaseProposalCanceled(release_proposal_id))
        {
            return Err(Error::ProposalCanceled);
        }

        helpers::check_membership(&env, release.group_id, user.clone())?;

        env.storage()
            .persistent()
            .set(
                &DataKey::ReleaseProposalCanceled(release_proposal_id),
                &true,
            );

        ReleaseCanceled {
            proposal_id: release_proposal_id,
            group_id: release.group_id,
            canceled_by: user,
        }
        .publish(&env);

        Ok(())
    }

    fn propose_fund_round(
        env: Env,
        group_id: u64,
        total_amount: i128,
        user: Address,
    ) -> Result<u64, Error> {
        user.require_auth();
        helpers::check_membership(&env, group_id, user.clone())?;
        helpers::assert_treasury_exists(&env, group_id)?;

        if total_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let member_count: u32;

        #[cfg(test)]
        {
            member_count = 4;
        }

        #[cfg(not(test))]
        {
            let group_contract = Address::from_str(&env, config::GROUP_CONTRACT);
            let client = GroupContract::new(&env, &group_contract);

            let members = client.get_members(&group_id);
            member_count = members.len().try_into().map_err(|_| Error::TooManyMembers)?;
        }

        if member_count == 0 {
            return Err(Error::NoMembers);
        }

        let mut fund_count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::FundRoundCount)
            .unwrap_or(0);

        fund_count += 1;
        let round_id = fund_count;

        let new_round = FundRound {
            group_id,
            total_amount,
            amount_of_members: member_count,
            funded_amount: 0i128,
            completed: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::FundRound(round_id), &new_round);

        let mut group_rounds: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::GroupFundRounds(group_id))
            .unwrap_or(Vec::new(&env));

        group_rounds.push_back(round_id);

        env.storage()
            .persistent()
            .set(&DataKey::GroupFundRounds(group_id), &group_rounds);

        env.storage()
            .persistent()
            .set(&DataKey::FundRoundCount, &fund_count);

        FundRoundProposed {
            round_id,
            group_id,
            total_amount,
            proposer: user,
        }
        .publish(&env);

        Ok(round_id)
    }

    fn contribute_to_fund_round(
        env: Env,
        round_id: u64,
        amount: i128,
        user: Address,
    ) -> Result<(), Error> {
        user.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut round: FundRound = env
            .storage()
            .persistent()
            .get(&DataKey::FundRound(round_id))
            .ok_or(Error::RoundNotFound)?;

        helpers::assert_treasury_exists(&env, round.group_id)?;
        helpers::check_membership(&env, round.group_id, user.clone())?;

        if round.completed {
            return Err(Error::RoundAlreadyCompleted);
        }

        let remaining = round
            .total_amount
            .checked_sub(round.funded_amount)
            .ok_or(Error::FundedExceedsTotal)?;

        if amount > remaining {
            return Err(Error::ContributionExceedsRemaining);
        }

        #[cfg(not(test))]
        {
            let usdc_address = Address::from_str(&env, config::USDC_ADDRESS);
            let token = TokenClient::new(&env, &usdc_address);

            token.transfer_from(
                &env.current_contract_address(),
                &user,
                &env.current_contract_address(),
                &amount,
            );
        }

        let group_id = round.group_id;
        let current: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::GroupBalance(group_id))
            .unwrap_or(0);
        let new_balance = current.checked_add(amount).ok_or(Error::BalanceOverflow)?;
        env.storage()
            .persistent()
            .set(&DataKey::GroupBalance(group_id), &new_balance);

        let contribution_key = DataKey::FundContribution(round_id, user.clone());

        let previous: i128 = env
            .storage()
            .persistent()
            .get(&contribution_key)
            .unwrap_or(0);

        let new_total_user = previous
            .checked_add(amount)
            .ok_or(Error::ContributionOverflow)?;

        env.storage()
            .persistent()
            .set(&contribution_key, &new_total_user);

        round.funded_amount += amount;

        if round.funded_amount >= round.total_amount {
            round.completed = true;
        }

        env.storage()
            .persistent()
            .set(&DataKey::FundRound(round_id), &round);

        Contribution {
            round_id,
            group_id,
            user: user.clone(),
            amount,
            new_funded_amount: round.funded_amount,
        }
        .publish(&env);

        if round.completed {
            FundRoundCompleted {
                round_id,
                group_id,
                total_amount: round.total_amount,
            }
            .publish(&env);
        }

        Ok(())
    }

    fn withdraw_contribution(
        env: Env,
        round_id: u64,
        user: Address,
    ) -> Result<(), Error> {
        user.require_auth();

        let mut round: FundRound = env
            .storage()
            .persistent()
            .get(&DataKey::FundRound(round_id))
            .ok_or(Error::RoundNotFound)?;

        if round.completed {
            return Err(Error::RoundAlreadyCompleted);
        }

        helpers::assert_treasury_exists(&env, round.group_id)?;
        helpers::check_membership(&env, round.group_id, user.clone())?;

        let contribution_key = DataKey::FundContribution(round_id, user.clone());
        let amount: i128 = env
            .storage()
            .persistent()
            .get(&contribution_key)
            .unwrap_or(0);

        if amount <= 0 {
            return Err(Error::NoContributionToWithdraw);
        }

        let group_id = round.group_id;

        round.funded_amount = round
            .funded_amount
            .checked_sub(amount)
            .ok_or(Error::BalanceUnderflow)?;

        let current_balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::GroupBalance(group_id))
            .ok_or(Error::GroupBalanceNotFound)?;
        let new_balance = current_balance
            .checked_sub(amount)
            .ok_or(Error::BalanceUnderflow)?;
        env.storage()
            .persistent()
            .set(&DataKey::GroupBalance(group_id), &new_balance);

        env.storage().persistent().set(&contribution_key, &0i128);
        env.storage()
            .persistent()
            .set(&DataKey::FundRound(round_id), &round);

        #[cfg(not(test))]
        {
            let usdc_address = Address::from_str(&env, config::USDC_ADDRESS);
            let token = TokenClient::new(&env, &usdc_address);
            let treasury_address = env.current_contract_address();
            if token.balance(&treasury_address) < amount {
                return Err(Error::InsufficientTreasuryBalance);
            }
            token.transfer(&treasury_address, &user, &amount);
        }

        ContributionWithdrawn {
            round_id,
            group_id,
            user,
            amount,
            new_funded_amount: round.funded_amount,
        }
        .publish(&env);

        Ok(())
    }

    fn get_group_rounds(env: Env, group_id: u64) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::GroupFundRounds(group_id))
            .unwrap_or(Vec::new(&env))
    }

    fn get_fund_round(env: Env, round_id: u64) -> Result<FundRound, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::FundRound(round_id))
            .ok_or(Error::RoundNotFound)
    }

    fn get_user_contribution(env: Env, round_id: u64, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::FundContribution(round_id, user))
            .unwrap_or(0)
    }

    fn get_release_proposals_of_group(env: Env, group_id: u64) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::GroupReleaseProposals(group_id))
            .unwrap_or(Vec::new(&env))
    }

    fn get_release_proposal(env: Env, proposal_id: u64) -> Result<ReleaseProposal, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::ReleaseProposal(proposal_id))
            .ok_or(Error::ProposalNotFound)
    }

    fn check_treasury_id(env: Env, group_id: u64) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::GroupId(group_id))
            .unwrap_or(false)
    }

    fn get_group_balance(env: Env, group_id: u64) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::GroupBalance(group_id))
            .unwrap_or(0)
    }

    fn has_sufficient_group_balance(env: Env, group_id: u64, amount: i128) -> bool {
        Self::get_group_balance(env, group_id) >= amount
    }
}
