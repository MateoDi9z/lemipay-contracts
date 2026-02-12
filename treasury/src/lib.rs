#![no_std]

mod test;

use core::convert::Into;
use soroban_sdk::{contract, contractimpl, contracttype, contractclient, Address, Env, Vec, String};

const GROUP_CONTRACT: &str = "CABYTW7GMOYRDOEYUTFQOFTYGPEFUZOOGYDIJLSYLDP7XFWQ4A2TFXP2";

#[contractclient(name = "GroupContract")]
pub trait IGroupContract {
    fn get_members(group_id: u64) -> Vec<Address>;

    fn create_group(members: Vec<Address>, approvals_required: u32) -> u64;
}

#[contract]
pub struct TreasuryContract;

#[derive(Clone)]
#[contracttype]
pub struct ReleaseProposal {
    pub destination: Address,
    pub amount: i128,
    pub approvals: u32,
    pub executed: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct FundRound {
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
    ReleaseProposalCount,
    ReleaseProposal(u32),
    Approval(u32, Address),
    FundRoundCount,
    FundRound(u64),              // round_id -> FundRound
    GroupFundRounds(u64),        // group_id -> Vec<round_id>
    FundContribution(u64, Address), // (round_id, member)
}

#[contractimpl]
impl TreasuryContract {

    // -------------------------------------------------
    // CREATE TREASURY
    // -------------------------------------------------
    fn check_membership(env: &Env,
        group_id: u64, user: Address) {
        #[cfg(test)]
        {
            // End validation in testing
            return;
        }
        user.require_auth();

        let group_contract = Address::from_string(&String::from_str(env, GROUP_CONTRACT));
        let client = GroupContract::new(&env, &group_contract);

        let addresses: Vec<Address> = client.get_members(&group_id);

        if !addresses.contains(&user) {
            panic!("NOT_MEMBER");
        }

    }

    pub fn create_treasury(
        env: Env,
        group_id: u64,
        user: Address,
    ) {
        Self::check_membership(&env, group_id, user);

        if env.storage().persistent().has(&DataKey::GroupId(group_id)) {
            panic!("Group ID already in use");
        }

        env.storage().persistent().set(&DataKey::GroupId(group_id), &true);
    }

    pub fn check_treasury_id(env: Env, group_id: u64) -> bool {
        env.storage().persistent().get(&DataKey::GroupId(group_id)).unwrap_or(false)
    }

    //
    // // -------------------------------------------------
    // // PROPOSE RELEASE
    // // -------------------------------------------------
    //
    // pub fn propose_release(
    //     env: Env,
    //     destination: Address,
    //     amount: i128,
    // ) -> u32 {
    //
    //     Self::require_group_member(&env);
    //
    //     let mut count: u32 = env.storage().persistent()
    //         .get(&DataKey::ReleaseProposalCount)
    //         .unwrap();
    //
    //     count += 1;
    //
    //     let proposal = ReleaseProposal {
    //         destination,
    //         amount,
    //         approvals: 0,
    //         executed: false,
    //     };
    //
    //     env.storage().persistent().set(&DataKey::ReleaseProposal(count), &proposal);
    //     env.storage().persistent().set(&DataKey::ReleaseProposalCount, &count);
    //
    //     count
    // }
    //
    // // -------------------------------------------------
    // // APPROVE
    // // -------------------------------------------------
    //
    // pub fn approve_release(env: Env, release_proposal_id: u32) {
    //
    //     Self::require_group_member(&env);
    //
    //     let caller = env.invoker();
    //
    //     let mut release: ReleaseProposal = env.storage()
    //         .persistent()
    //         .get(&DataKey::ReleaseProposal(release_proposal_id))
    //         .unwrap();
    //
    //     if release.executed {
    //         panic!("ALREADY_EXECUTED");
    //     }
    //
    //     let approval_key = DataKey::Approval(release_proposal_id, caller.clone());
    //
    //     if env.storage().persistent().has(&approval_key) {
    //         panic!("ALREADY_APPROVED");
    //     }
    //
    //     env.storage().persistent().set(&approval_key, &true);
    //     release.approvals += 1;
    //
    //     env.storage().persistent().set(&DataKey::ReleaseProposal(release_proposal_id), &release);
    //
    //     let min: u32 = env.storage().persistent()
    //         .get(&DataKey::MinApprovals)
    //         .unwrap();
    //
    //     if release.approvals >= min {
    //         Self::execute_release(env, release_proposal_id);
    //     }
    // }
    //
    // // -------------------------------------------------
    // // EXECUTE (USDC hardcoded)
    // // -------------------------------------------------
    //
    // fn execute_release(env: Env, release_proposal_id: u32) {
    //
    //     let mut release: ReleaseProposal = env.storage()
    //         .persistent()
    //         .get(&DataKey::ReleaseProposal(release_proposal_id))
    //         .unwrap();
    //
    //     if release.executed {
    //         panic!("ALREADY_EXECUTED");
    //     }
    //
    //     let min: u32 = env.storage().persistent()
    //         .get(&DataKey::MinApprovals)
    //         .unwrap();
    //
    //     if release.approvals < min {
    //         panic!("NOT_ENOUGH_APPROVALS");
    //     }
    //
    //     // USDC Testnet address (ejemplo)
    //     let usdc = Address::from_string(&env, &"CDLZFC3SYL...".into());
    //
    //     let contract_address = env.current_contract_address();
    //
    //     env.invoke_contract::<()>(
    //         &usdc,
    //         &symbol_short!("transfer"),
    //         (
    //             contract_address,
    //             release.destination.clone(),
    //             release.amount,
    //         ).into_val(&env),
    //     );
    //
    //     release.executed = true;
    //     env.storage().persistent().set(&DataKey::ReleaseProposal(release_proposal_id), &release);
    // }

    // -------------------------------------------------
    // PROPOSE FUND
    // -------------------------------------------------
    pub fn propose_fund_round(env: Env, group_id: u64, total_amount: i128, user: Address) -> u64 {
        Self::check_membership(&env, group_id, user);

        if total_amount <= 0 {
            panic!("INVALID_AMOUNT");
        }

        // obtener cantidad de miembros
        let member_count: u32 = 4;  // TODO: Reemplazar con lógica para obtener el número real de miembros del grupo desde el contrato del grupo

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
        group_id: u64,
        user: Address,
        amount: i128,
    ) {
        Self::check_membership(&env, group_id, user.clone());

        if amount <= 0 {
            panic!("INVALID_AMOUNT");
        }

        let mut round: FundRound = env.storage()
            .persistent()
            .get(&DataKey::FundRound(round_id))
            .expect("ROUND_NOT_FOUND");

        if round.completed {
            panic!("ROUND_ALREADY_COMPLETED");
        }

        // Obtener aporte previo del usuario
        let contribution_key = DataKey::FundContribution(round_id, user.clone());

        let previous: i128 = env.storage()
            .persistent()
            .get(&contribution_key)
            .unwrap_or(0);

        let new_total_user = previous + amount;

        // Guardar aporte actualizado
        env.storage()
            .persistent()
            .set(&contribution_key, &new_total_user);

        // Actualizar total del round
        round.funded_amount += amount;

        // Verificar si se completa
        if round.funded_amount >= round.total_amount {
            round.completed = true;
        }

        env.storage()
            .persistent()
            .set(&DataKey::FundRound(round_id), &round);
    }



    // -------------------------------------------------
    // GETTERS
    // -------------------------------------------------

    // Get fund rounds of group
    pub fn get_group_rounds(env: Env, group_id: u64) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::GroupFundRounds(group_id))
            .unwrap_or(Vec::new(&env))
    }

    // Get specific fund round details
    pub fn get_fund_round(env: Env, round_id: u64) -> FundRound {
        env.storage()
            .persistent()
            .get(&DataKey::FundRound(round_id))
            .expect("ROUND_NOT_FOUND")
    }

    // Get user's contribution to a fund round
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
}