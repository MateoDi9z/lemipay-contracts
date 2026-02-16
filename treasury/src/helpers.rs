//! Internal helpers for the Treasury contract.

use soroban_sdk::{Address, Env};

use crate::errors::Error;
use crate::storage::DataKey;

#[cfg(not(test))]
use crate::clients::GroupContract;
#[cfg(not(test))]
use crate::config;

/// Check if user is member of group. In tests, this is bypassed to simplify testing.
#[allow(unused_variables)]
pub(crate) fn check_membership(env: &Env, group_id: u64, user: Address) -> Result<(), Error> {
    #[cfg(test)]
    return Ok(());

    #[cfg(not(test))]
    {
        let group_contract = Address::from_str(env, config::GROUP_CONTRACT);
        let client = GroupContract::new(env, &group_contract);

        let addresses = client.get_members(&group_id);

        if !addresses.contains(&user) {
            return Err(Error::NotMember);
        }
        Ok(())
    }
}

/// Check if treasury exists for group ID.
pub(crate) fn assert_treasury_exists(env: &Env, group_id: u64) -> Result<(), Error> {
    let exists = env
        .storage()
        .persistent()
        .get(&DataKey::GroupId(group_id))
        .unwrap_or(false);

    if !exists {
        return Err(Error::TreasuryNotInitialized);
    }
    Ok(())
}
