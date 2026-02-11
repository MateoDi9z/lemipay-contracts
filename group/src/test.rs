use soroban_sdk::{Vec, testutils::Address as _, Address, Env};


use crate::{GroupContract, GroupContractClient};

#[test]
fn test_create_group() {
    let env = Env::default();

    // Generar direcciones mock (forma correcta en v25)
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let members = Vec::from_array(&env, [user1.clone(), user2.clone()]);

    let contract = env.register(GroupContract, ());
    let client = GroupContractClient::new(&env, &contract);

    let group_id = client.create_group(&members, &2);

    let group = client.get_group(&group_id);

    assert_eq!(group.approvals_required, 2);
    assert_eq!(group.members.len(), 2);
}