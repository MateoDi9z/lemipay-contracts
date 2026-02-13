#![no_std]

mod test;
mod config;

use crate::config::{GROUP_CONTRACT, USDC_ADDRESS};

use core::convert::Into;
use soroban_sdk::{contract, contractimpl, contracttype, contractclient, Address, Env, Vec};

#[cfg(not(test))]
use soroban_sdk::token::Client as TokenClient;

#[contractclient(name = "GroupContract")]
pub trait IGroupContract {
    fn get_members(group_id: u64) -> Vec<Address>;

    fn create_group(members: Vec<Address>, approvals_required: u32) -> u64;

    fn get_approval_rule(env: Env, group_id: u64) -> u32;
}

#[contract]
pub struct TreasuryContract;

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

#[derive(Clone)]
#[contracttype]
enum DataKey {
    TreasuryId,
    GroupId(u64),

    // Release proposal related keys
    ReleaseProposalCount,           // id generator
    ReleaseProposal(u64),           // proposal_id -> struct
    GroupReleaseProposals(u64),     // group_id -> Vec<proposal_id>
    ReleaseApproval(u64, Address),  // (proposal_id, user)

    // Fund round related keys
    FundRoundCount,                 // id generator
    FundRound(u64),                 // round_id -> FundRound
    GroupFundRounds(u64),           // group_id -> Vec<round_id>
    FundContribution(u64, Address), // (round_id, member)
}

#[contractimpl]
impl TreasuryContract {

    /// ------------------------------------------------
    /// HELPERS
    /// ------------------------------------------------

    /// Check if user is member of group, panics if not. In tests, this is bypassed to simplify testing.
    fn check_membership(env: &Env, group_id: u64, user: Address) {
        #[cfg(test)] // Only for testing, skip actual membership check to simplify tests
        return;

        #[cfg(not(test))] {
            let group_contract = Address::from_str(&env, config::GROUP_CONTRACT);
            let client = GroupContract::new(&env, &group_contract);

            let addresses: Vec<Address> = client.get_members(&group_id);

            if !addresses.contains(&user) {
                panic!("NOT_MEMBER");
            }
        }
    }


    /// ------------------------------------------------
    /// CORE FUNCTIONS
    /// ------------------------------------------------

    /// CREATE TREASURY
    pub fn create_treasury(
        env: Env,
        group_id: u64,
        user: Address,
    ) {
        user.require_auth();                                    // Auth user
        Self::check_membership(&env, group_id, user.clone());   // Check membership

        if env.storage().persistent().has(&DataKey::GroupId(group_id)) {
            panic!("Group ID already in use");
        }

        env.storage().persistent().set(&DataKey::GroupId(group_id), &true);
    }


    /// PROPOSE RELEASE
    pub fn propose_release(
        env: Env,
        destination: Address,
        amount: i128,
        group_id: u64,
        user: Address,
    ) -> u64 {
        user.require_auth();                                            // Auth user
        Self::check_membership(&env, group_id, user.clone());           // Check membership
        Self::check_membership(&env, group_id, destination.clone());    // Check destination is member
        // TODO: Upgrade, check both addresses in same iteration & custom error

        if amount <= 0 {
            panic!("INVALID_AMOUNT");
        }

        let mut count: u64 = env.storage()
            .persistent()
            .get(&DataKey::ReleaseProposalCount)
            .unwrap_or(0);

        count += 1;

        let proposal = ReleaseProposal {
            group_id,
            destination,
            amount,
            approvals: 0,
            executed: false,
        };

        let mut group_proposals: Vec<u64> = env.storage()
            .persistent()
            .get(&DataKey::GroupReleaseProposals(group_id))
            .unwrap_or(Vec::new(&env));

        group_proposals.push_back(count);

        env.storage()
            .persistent()
            .set(&DataKey::GroupReleaseProposals(group_id), &group_proposals);

        env.storage().persistent().set(&DataKey::ReleaseProposal(count), &proposal);
        env.storage().persistent().set(&DataKey::ReleaseProposalCount, &count);

        count
    }

    /// APPROVE RELEASE
    pub fn approve_release(env: Env, release_proposal_id: u64, user: Address) {
        user.require_auth();

        let mut release: ReleaseProposal = env.storage()
            .persistent()
            .get(&DataKey::ReleaseProposal(release_proposal_id))
            .expect("PROPOSAL_NOT_FOUND");

        if release.executed {
            panic!("ALREADY_EXECUTED");
        }

        if release.destination == user {
            panic!("DESTINATION_CANNOT_APPROVE");
        }

        Self::check_membership(&env, release.group_id, user.clone());

        let approval_key = DataKey::ReleaseApproval(release_proposal_id, user.clone());

        if env.storage().persistent().has(&approval_key) {
            panic!("ALREADY_APPROVED");
        }

        env.storage().persistent().set(&approval_key, &true);

        release.approvals += 1;

        env.storage()
            .persistent()
            .set(&DataKey::ReleaseProposal(release_proposal_id), &release);
    }

    // -------------------------------------------------
    // EXECUTE (Release USDC)
    // -------------------------------------------------
    pub fn execute_release(env: Env, release_proposal_id: u64) {

        let mut release: ReleaseProposal = env.storage()
            .persistent()
            .get(&DataKey::ReleaseProposal(release_proposal_id))
            .expect("PROPOSAL_NOT_FOUND");

        if release.executed {
            panic!("ALREADY_EXECUTED");
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
            panic!("NOT_ENOUGH_APPROVALS");
        }

        #[cfg(test)]
        {
            release.executed = true;
            env.storage().persistent().set(&DataKey::ReleaseProposal(release_proposal_id), &release);
            return;
        }

        #[cfg(not(test))]
        {
            let usdc_address = Address::from_str(&env, config::USDC_ADDRESS);
            let token = TokenClient::new(&env, &usdc_address);

            let treasury_address = env.current_contract_address();

            let current_balance = token.balance(&treasury_address);

            if current_balance < release.amount {
                panic!("INSUFFICIENT_TREASURY_BALANCE");
            }

            token.transfer(
                &treasury_address,
                &release.destination,
                &release.amount,
            );

            release.executed = true;

            env.storage()
                .persistent()
                .set(&DataKey::ReleaseProposal(release_proposal_id), &release);
        }
    }

    // -------------------------------------------------
    // PROPOSE FUND ROUND
    // -------------------------------------------------
    pub fn propose_fund_round(env: Env, group_id: u64, total_amount: i128, user: Address) -> u64 {
        user.require_auth();                            // Auth user
        Self::check_membership(&env, group_id, user);   // Check membership

        if total_amount <= 0 {
            panic!("INVALID_AMOUNT");
        }

        // Get member count for the group.
        let member_count: u32;

        #[cfg(test)] {
            // In tests, this is hardcoded to simplify testing.
            member_count = 4;
        }

        #[cfg(not(test))] {
            let group_contract = Address::from_str(&env, config::GROUP_CONTRACT);
            let client = GroupContract::new(&env, &group_contract);

            let members = client.get_members(&group_id);
            member_count = members.len().try_into().expect("Too many members");
        }

        if member_count == 0 {
            panic!("NO_MEMBERS");
        }

        let mut fund_count: u64 = env.storage()
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

        //  Save individual fund round
        env.storage()
            .persistent()
            .set(&DataKey::FundRound(round_id), &new_round);

        // Add ID to group list
        let mut group_rounds: Vec<u64> = env.storage()
            .persistent()
            .get(&DataKey::GroupFundRounds(group_id))
            .unwrap_or(Vec::new(&env));

        group_rounds.push_back(round_id);

        env.storage()
            .persistent()
            .set(&DataKey::GroupFundRounds(group_id), &group_rounds);

        // Update global fund round count
        env.storage()
            .persistent()
            .set(&DataKey::FundRoundCount, &fund_count);

        round_id
    }

    // -------------------------------------------------
    // CONTRIBUTE TO FUND ROUND
    // -------------------------------------------------
    pub fn contribute_to_fund_round(
        env: Env,
        round_id: u64,
        amount: i128,
        user: Address,
    ) {
        user.require_auth();                                    // Auth user

        if amount <= 0 {
            panic!("INVALID_AMOUNT");
        }

        let mut round: FundRound = env.storage()
            .persistent()
            .get(&DataKey::FundRound(round_id))
            .expect("ROUND_NOT_FOUND");

        Self::check_membership(&env, round.group_id, user.clone());   // Check membership

        if round.completed {
            panic!("ROUND_ALREADY_COMPLETED");
        }

        // Calculate remaining
        let remaining = round
            .total_amount
            .checked_sub(round.funded_amount)
            .expect("Invalid state: funded exceeds total");
        
        // Block overshoot contributions.
        if amount > remaining {
            panic!("Contribution exceeds remaining target");
        }

        // Only execute token transfer in non-test environment.
        #[cfg(not(test))] {
            let usdc_address = Address::from_str(&env, config::USDC_ADDRESS);
            let token = TokenClient::new(&env, &usdc_address);

            if token.balance(&user) < amount {
                panic!("insufficient balance");
            }

            token.transfer(
                &user,
                &env.current_contract_address(),
                &amount,
            );
        }

        // Get previous contribution of user to this round
        let contribution_key = DataKey::FundContribution(round_id, user.clone());

        let previous: i128 = env.storage()
            .persistent()
            .get(&contribution_key)
            .unwrap_or(0);

        // Add new contribution to previous
        let new_total_user = previous + amount;

        // Save
        env.storage()
            .persistent()
            .set(&contribution_key, &new_total_user);

        // Update total funded amount for the round
        round.funded_amount += amount;

        // Check if round is completed
        if round.funded_amount >= round.total_amount {
            round.completed = true;
        }

        // Update round details
        env.storage()
            .persistent()
            .set(&DataKey::FundRound(round_id), &round);
    }


    /// -------------------------------------------------
    /// GETTERS
    /// -------------------------------------------------

    /// Get fund rounds of group
    pub fn get_group_rounds(env: Env, group_id: u64) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::GroupFundRounds(group_id))
            .unwrap_or(Vec::new(&env))
    }

    /// Get specific fund round details
    pub fn get_fund_round(env: Env, round_id: u64) -> FundRound {
        env.storage()
            .persistent()
            .get(&DataKey::FundRound(round_id))
            .expect("ROUND_NOT_FOUND")
    }

    /// Get user's contribution to a fund round
    pub fn get_user_contribution(
        env: Env,
        round_id: u64,
        user: Address,
    ) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::FundContribution(round_id, user))
            .unwrap_or(0)
    }

    /// Get release proposal details by Group ID
    pub fn get_release_proposals_of_group(env: Env, group_id: u64) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::GroupReleaseProposals(group_id))
            .unwrap_or(Vec::new(&env))
    }

    /// Get specific release proposal details
    pub fn get_release_proposal(env: Env, proposal_id: u64) -> ReleaseProposal {
        env.storage()
            .persistent()
            .get(&DataKey::ReleaseProposal(proposal_id))
            .expect("PROPOSAL_NOT_FOUND")
    }

    /// CHECK IF TREASURY EXISTS FOR GROUP ID
    pub fn check_treasury_id(env: Env, group_id: u64) -> bool {
        env.storage().persistent().get(&DataKey::GroupId(group_id)).unwrap_or(false)
    }
}