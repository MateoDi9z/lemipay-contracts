#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, Vec,
};

#[contracttype]
#[derive(Clone)]
pub struct Group {
    pub members: Vec<Address>,
    pub approvals_required: u32,
}

#[contracttype]
pub enum DataKey {
    Group(u64),
    GroupCounter,
}

#[contract]
pub struct GroupContract;

#[contractimpl]
impl GroupContract {

    /// Creates a new group and returns group_id
    pub fn create_group(
        env: Env,
        members: Vec<Address>,
        approvals_required: u32,
    ) -> u64 {
        if approvals_required == 0 {
            panic!("Invalid approval rule");
        }

        // Ensure there are no duplicate members
        for i in 0..members.len() {
            for j in (i + 1)..members.len() {
                if members.get(i).unwrap() == members.get(j).unwrap() {
                    panic!("Duplicate members detected");
                }
            }
        }

        if approvals_required > members.len() as u32 {
            panic!("Invalid approval rule");
        }

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::GroupCounter)
            .unwrap_or(0);

        counter += 1;

        let group = Group {
            members,
            approvals_required,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Group(counter), &group);

        env.storage()
            .instance()
            .set(&DataKey::GroupCounter, &counter);

        counter
    }

    /// Returns group data
    pub fn get_group(env: Env, group_id: u64) -> Group {
        env.storage()
            .persistent()
            .get(&DataKey::Group(group_id))
            .expect("Group not found")
    }

    /// Returns members of a group
    pub fn get_members(env: Env, group_id: u64) -> Vec<Address> {
        let group = Self::get_group(env, group_id);
        group.members
    }

    /// Returns approval rule
    pub fn get_approval_rule(env: Env, group_id: u64) -> u32 {
        let group = Self::get_group(env, group_id);
        group.approvals_required
    }
}

#[cfg(test)]
mod test;