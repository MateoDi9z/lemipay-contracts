mod fund_round_test {
    use soroban_sdk::{testutils::Address as _, Address, Env};

    use crate::errors::Error;
    use crate::{TreasuryContract, TreasuryContractClient};

    #[test]
    fn test_propose_fund_round_creates_round() {
        let env = Env::default();
        env.mock_all_auths();

        let user1 = Address::generate(&env);
        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let group_id: u64 = 2;

        client.create_treasury(&group_id, &user1);

        let fund_round_id = client.propose_fund_round(&group_id, &1000, &user1);
        let group_fund_rounds = client.get_group_rounds(&group_id);

        assert_eq!(group_fund_rounds.len(), 1);
        assert!(group_fund_rounds.contains(&fund_round_id));

        let fund_round = client.get_fund_round(&fund_round_id);
        assert_eq!(fund_round.total_amount, 1000);
        assert_eq!(fund_round.amount_of_members, 4);
        assert_eq!(fund_round.funded_amount, 0);
        assert_eq!(fund_round.completed, false);
    }

    #[test]
    fn test_propose_fund_round() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let group_id = 1;

        client.create_treasury(&group_id, &user);
        let round_id = client.propose_fund_round(&group_id, &1000, &user);

        assert_eq!(round_id, 1);

        let round = client.get_fund_round(&round_id);
        assert_eq!(round.total_amount, 1000);
        assert_eq!(round.funded_amount, 0);
        assert!(!round.completed);
    }

    #[test]
    fn test_contribute_rejects_invalid_amount() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let group_id = 1;

        client.create_treasury(&group_id, &user);
        let round_id = client.propose_fund_round(&group_id, &1000, &user);

        let result = client.try_contribute_to_fund_round(&round_id, &0, &user);
        assert_eq!(result, Err(Ok(Error::InvalidAmount)));
    }

    #[test]
    fn test_contribute_to_fund_round() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let group_id = 1;

        client.create_treasury(&group_id, &user);
        let round_id = client.propose_fund_round(&group_id, &1000, &user);

        client.contribute_to_fund_round(&round_id, &250, &user);

        let round = client.get_fund_round(&round_id);
        assert_eq!(round.funded_amount, 250);

        let user_contrib = client.get_user_contribution(&round_id, &user);
        assert_eq!(user_contrib, 250);
    }

    #[test]
    fn test_propose_expense_rejects_negative() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let group_id: u64 = 1;
        let user = Address::generate(&env);

        client.create_treasury(&group_id, &user);
        let result = client.try_propose_fund_round(&group_id, &-1000, &user);
        assert_eq!(result, Err(Ok(Error::InvalidAmount)));
    }
}
