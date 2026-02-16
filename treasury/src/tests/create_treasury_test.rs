mod create_treasury_test {
    use soroban_sdk::{testutils::Address as _, Address, Env};

    use crate::{TreasuryContract, TreasuryContractClient};

    #[test]
    #[should_panic]
    fn test_duped_group_id() {
        let env = Env::default();
        env.mock_all_auths();

        let user1 = Address::generate(&env);
        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let group_id: u64 = 1;

        client.create_treasury(&group_id, &user1);
        client.create_treasury(&group_id, &user1);
    }

    #[test]
    fn test_create_treasury() {
        let env = Env::default();
        env.mock_all_auths();

        let user = Address::generate(&env);
        let group_id = 1;
        let treasury_contract_id = env.register(TreasuryContract, ());
        let treasury_client = TreasuryContractClient::new(&env, &treasury_contract_id);

        treasury_client.create_treasury(&group_id, &user);

        let exists = treasury_client.check_treasury_id(&group_id);
        assert!(exists);
    }
}
