#[cfg(test)]
mod test {
    use soroban_sdk::{
        testutils::{Address as _},
        Address, Env
    };

    use crate::errors::Error;
    use crate::{TreasuryContract, TreasuryContractClient};

    #[test]
    #[should_panic]
    fn test_duped_group_id() {
        let env = Env::default();
        env.mock_all_auths(); // require_auth()

        let user1 = Address::generate(&env);

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);

        let group_id: u64 = 1;

        // First creation should succeed
        client.create_treasury(&group_id, &user1);

        // Second creation with same group ID should panic
        client.create_treasury(&group_id, &user1);
    }

    #[test]
    fn test_create_treasury() {
        let env = Env::default();
        env.mock_all_auths(); // require_auth()

        let user = Address::generate(&env);


        let group_id = 1;

        // Registrar Treasury
        let treasury_contract_id = env.register(TreasuryContract, ());
        let treasury_client = TreasuryContractClient::new(&env, &treasury_contract_id);

        // Create treasury
        treasury_client.create_treasury(&group_id, &user);

        // Verify group ID is stored
        let exists = treasury_client.check_treasury_id(&group_id);
        assert!(exists);
    }

    /// FUND ROUND

    #[test]
    fn test_propose_found_round() {
        let env = Env::default();
        env.mock_all_auths(); // require_auth()

        let user1 = Address::generate(&env);

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);

        let group_id: u64 = 2;

        // Create treasury
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

    /// RELEASE

    #[test]
    fn test_propose_release() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let destination = Address::generate(&env);
        let group_id = 1;

        client.create_treasury(&group_id, &user);
        let proposal_id = client.propose_release(
            &destination,
            &1000,
            &group_id,
            &user,
        );

        assert_eq!(proposal_id, 1);

        let proposal = client.get_release_proposal(&proposal_id);

        assert_eq!(proposal.group_id, group_id);
        assert_eq!(proposal.amount, 1000);
        assert_eq!(proposal.approvals, 0);
        assert!(!proposal.executed);
    }

    #[test]
    fn test_approve_does_not_execute() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let dest = Address::generate(&env);
        let group_id = 1;

        client.create_treasury(&group_id, &user);
        let proposal_id = client.propose_release(&dest, &1000, &group_id, &user);

        client.approve_release(&proposal_id, &user);

        let proposal = client.get_release_proposal(&proposal_id);

        assert_eq!(proposal.approvals, 1);
        assert!(!proposal.executed); // 🔥 importante
    }

    #[test]
    fn test_destination_cannot_approve() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let group_id = 1;

        client.create_treasury(&group_id, &user);

        // destination == user
        let proposal_id = client.propose_release(
            &user,
            &1000,
            &group_id,
            &user,
        );

        // intenta aprobar el mismo que es destino
        let result = client.try_approve_release(&proposal_id, &user);
        assert_eq!(result, Err(Ok(Error::DestinationCannotApprove)));
    }

    #[test]
    fn test_execute_after_approval() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let dest = Address::generate(&env);
        let group_id = 1;

        client.create_treasury(&group_id, &user);
        // Fund the group so execute_release can debit group balance
        let round_id = client.propose_fund_round(&group_id, &1000, &user);
        client.contribute_to_fund_round(&round_id, &1000, &user);

        let proposal_id = client.propose_release(&dest, &1000, &group_id, &user);

        client.approve_release(&proposal_id, &user);

        client.execute_release(&proposal_id);

        let proposal = client.get_release_proposal(&proposal_id);

        assert!(proposal.executed);
    }

    #[test]
    fn test_execute_without_threshold_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);

        let user1 = Address::generate(&env);
        let dest = Address::generate(&env);
        let group_id = 1;

        client.create_treasury(&group_id, &user1);
        let proposal_id = client.propose_release(&dest, &1000, &group_id, &user1);

        // no approve
        let result = client.try_execute_release(&proposal_id);
        assert_eq!(result, Err(Ok(Error::NotEnoughApprovals)));
    }

    // -------------------------------------------------------------------------
    // GROUP BALANCE TRACKING
    // -------------------------------------------------------------------------

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
        assert_eq!(client.get_group_balance(&group_id_b), 0, "group B balance must stay 0");

        let round_b = client.propose_fund_round(&group_id_b, &200, &user2);
        client.contribute_to_fund_round(&round_b, &200, &user2);

        assert_eq!(client.get_group_balance(&group_id_a), 500);
        assert_eq!(client.get_group_balance(&group_id_b), 200);
    }
}

