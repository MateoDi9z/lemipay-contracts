mod group_balance_test {
    use soroban_sdk::{testutils::Address as _, Address, Env};

    use crate::errors::Error;
    use crate::{TreasuryContract, TreasuryContractClient};

    #[test]
    fn test_group_balance_initialized_to_zero() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let group_id = 1;

        client.create_treasury(&group_id, &user);

        let balance = client.get_group_balance(&group_id);
        assert_eq!(balance, 0, "new treasury group balance must be 0");

        assert!(
            client.has_sufficient_group_balance(&group_id, &0),
            "has_sufficient_group_balance(0) should be true when balance is 0"
        );
        assert!(
            !client.has_sufficient_group_balance(&group_id, &1),
            "has_sufficient_group_balance(1) should be false when balance is 0"
        );
    }

    #[test]
    fn test_group_balance_increases_on_contribute() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let group_id = 1;

        client.create_treasury(&group_id, &user);
        assert_eq!(client.get_group_balance(&group_id), 0);

        let round_id = client.propose_fund_round(&group_id, &1000, &user);
        client.contribute_to_fund_round(&round_id, &250, &user);

        assert_eq!(
            client.get_group_balance(&group_id),
            250,
            "group balance should be 250 after contributing 250"
        );
        assert!(client.has_sufficient_group_balance(&group_id, &250));
        assert!(!client.has_sufficient_group_balance(&group_id, &251));

        client.contribute_to_fund_round(&round_id, &750, &user);

        assert_eq!(
            client.get_group_balance(&group_id),
            1000,
            "group balance should be 1000 after total contributions"
        );
    }

    #[test]
    fn test_group_balance_decreases_on_execute_release() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let dest = Address::generate(&env);
        let group_id = 1;

        client.create_treasury(&group_id, &user);
        let round_id = client.propose_fund_round(&group_id, &1000, &user);
        client.contribute_to_fund_round(&round_id, &1000, &user);

        assert_eq!(client.get_group_balance(&group_id), 1000);

        let proposal_id = client.propose_release(&dest, &300, &group_id, &user);
        client.approve_release(&proposal_id, &user);
        client.execute_release(&proposal_id);

        assert_eq!(
            client.get_group_balance(&group_id),
            700,
            "group balance should be 700 after releasing 300"
        );
        assert!(client.has_sufficient_group_balance(&group_id, &700));
        assert!(!client.has_sufficient_group_balance(&group_id, &701));
    }

    #[test]
    fn test_execute_release_fails_insufficient_group_balance() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let dest = Address::generate(&env);
        let group_id = 1;

        client.create_treasury(&group_id, &user);
        let round_id = client.propose_fund_round(&group_id, &1000, &user);
        client.contribute_to_fund_round(&round_id, &100, &user);

        assert_eq!(client.get_group_balance(&group_id), 100);

        let proposal_id = client.propose_release(&dest, &300, &group_id, &user);
        client.approve_release(&proposal_id, &user);

        let result = client.try_execute_release(&proposal_id);
        assert_eq!(
            result,
            Err(Ok(Error::InsufficientGroupBalance)),
            "execute_release must fail when group balance (100) < release amount (300)"
        );

        assert_eq!(
            client.get_group_balance(&group_id),
            100,
            "balance must be unchanged after failed release"
        );
    }

    #[test]
    fn test_group_balances_isolated() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let group_id_a = 1u64;
        let group_id_b = 2u64;

        client.create_treasury(&group_id_a, &user1);
        client.create_treasury(&group_id_b, &user2);

        assert_eq!(client.get_group_balance(&group_id_a), 0);
        assert_eq!(client.get_group_balance(&group_id_b), 0);

        let round_a = client.propose_fund_round(&group_id_a, &500, &user1);
        client.contribute_to_fund_round(&round_a, &500, &user1);

        assert_eq!(client.get_group_balance(&group_id_a), 500);
        assert_eq!(
            client.get_group_balance(&group_id_b),
            0,
            "group B balance must stay 0"
        );

        let round_b = client.propose_fund_round(&group_id_b, &200, &user2);
        client.contribute_to_fund_round(&round_b, &200, &user2);

        assert_eq!(client.get_group_balance(&group_id_a), 500);
        assert_eq!(client.get_group_balance(&group_id_b), 200);
    }
}
