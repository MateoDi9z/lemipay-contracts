//! Client for the Group contract.
//!
//! Interface aligned with `./group` contract.
//! Env is passed when creating the client (GroupContract::new(&env, &address));
//! generated methods take only the remaining arguments.

use soroban_sdk::{contractclient, contracttype, Address, Vec};

/// Group data returned by the Group contract (same layout as group::Group).
#[derive(Clone)]
#[contracttype]
pub struct Group {
    pub members: Vec<Address>,
    pub approvals_required: u32,
}

#[contractclient(name = "GroupContract")]
pub trait IGroupContract {
    fn create_group(members: Vec<Address>, approvals_required: u32) -> u64;

    fn get_group(group_id: u64) -> Group;

    fn get_members(group_id: u64) -> Vec<Address>;

    fn get_approval_rule(group_id: u64) -> u32;
}
