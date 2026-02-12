#![no_std]

const GROUP_CONTRACT: &str = "CCXXXXX";

use soroban_sdk::{
    contract, contractimpl, contracttype, contractclient,
    Address, Env, Vec, Map, Bytes, Symbol, symbol_short, IntoVal
};

// #[contractclient(name = "Group-Contract")] // Esto genera automáticamente un "ContratoB_Client"
// pub trait IContrato {
//     fn consultar_saldo(env: Env, cuenta: Address) -> i128;
// }

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
    FundRound(u64),
    FundContribution(u32, Address), // (round_id, member)
}

#[contractimpl]
impl TreasuryContract {

    // -------------------------------------------------
    // INITIALIZE (constructor)
    // -------------------------------------------------

    pub fn create_treasury(
        env: Env,
        group_id: u64,
    ) {
        Self::require_group_member(&env, group_id);

        if env.storage().persistent().has(&DataKey::GroupId(group_id)) {
            panic!("Group ID already in use");
        }

        env.storage().persistent().set(&DataKey::GroupId(group_id), &true);
    }

    pub fn check_treasury_id(env: Env, group_id: u64) -> bool {
        env.storage().persistent().get(&DataKey::GroupId(group_id)).unwrap_or(false)
    }

    // -------------------------------------------------
    // TODO: AUTH CHECK
    // -------------------------------------------------
    //
    fn require_group_member(env: &Env, group_id: u64) {
        // TODO: Implementar lógica de verificación de membresía en el grupo

        // let caller = env.invoker();
        // caller.require_auth();
        //
        // let group_contract: Address = env.storage().persistent()
        //     .get(&DataKey::GroupContract)
        //     .unwrap();
        //
        // let group_id: u32 = env.storage().persistent()
        //     .get(&DataKey::GroupId)
        //     .unwrap();
        //
        // let is_member: bool = env.invoke_contract(
        //     &group_contract,
        //     &symbol_short!("is_member"),
        //     (group_id, caller.clone()).into_val(env),
        // );
        //
        // if !is_member {
        //     panic!("NOT_IN_GROUP");
        // }
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
    //
    // -------------------------------------------------
    // PROPOSE FUND
    // -------------------------------------------------
    pub fn propose_fund(env: Env, group_id: u64, total_amount: i128) -> u64 {
        Self::require_group_member(&env, group_id);

        // obtener cantidad de miembros
        let member_count: u32 = 4;  // TODO: Reemplazar con lógica para obtener el número real de miembros del grupo desde el contrato del grupo

        if member_count == 0 {
            panic!("NO_MEMBERS");
        }

        let mut fund_count: u64 = env.storage().persistent()
            .get(&DataKey::FundRoundCount)
            .unwrap_or(0);

        fund_count += 1;

        let round = FundRound {
            total_amount,
            amount_of_members: member_count,
            funded_amount: 0i128,
            completed: false,
        };

        env.storage().persistent().set(&DataKey::FundRound(fund_count), &round);
        env.storage().persistent().set(&DataKey::FundRoundCount, &fund_count);

        fund_count
    }

    // Get fund rounds of group
    pub fn get_fund_rounds(env: Env, group_id: u64) -> Vec<FundRound> {
        let fund_count: Vec<FundRound> = env.storage().persistent()
            .get(&DataKey::FundRound(group_id)).unwrap();

        fund_count
    }
}

mod test;