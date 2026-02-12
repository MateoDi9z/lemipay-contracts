#[cfg(test)]
mod test {
    use soroban_sdk::{
        testutils::{Address as _},
        Address, Env
    };

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

        let round_id = client.propose_fund_round(&group_id, &1000, &user);

        assert_eq!(round_id, 1);

        let round = client.get_fund_round(&round_id);

        assert_eq!(round.total_amount, 1000);
        assert_eq!(round.funded_amount, 0);
        assert!(!round.completed);
    }

    #[test]
    #[should_panic(expected = "INVALID_AMOUNT")]
    fn test_contribute_rejects_invalid_amount() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let group_id = 1;

        let round_id = client.propose_fund_round(&group_id, &1000, &user);

        client.contribute_to_fund_round(&round_id, &group_id, &user, &0);
    }

    #[test]
    #[should_panic(expected = "INVALID_AMOUNT")]
    fn test_propose_expense_rejects_negative() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);

        let group_id: u64 = 1;
        let user = Address::generate(&env);

        client.propose_fund_round(&group_id, &-1000, &user);
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

        let proposal_id = client.propose_release(&dest, &1000, &group_id, &user);

        client.approve_release(&proposal_id, &user);

        let proposal = client.get_release_proposal(&proposal_id);

        assert_eq!(proposal.approvals, 1);
        assert!(!proposal.executed); // 🔥 importante
    }

    #[test]
    #[should_panic(expected = "DESTINATION_CANNOT_APPROVE")]
    fn test_destination_cannot_approve() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let group_id = 1;

        // destination == user
        let proposal_id = client.propose_release(
            &user,      // 👈 mismo address
            &1000,
            &group_id,
            &user,
        );

        // intenta aprobar el mismo que es destino
        client.approve_release(&proposal_id, &user);
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

        let proposal_id = client.propose_release(&dest, &1000, &group_id, &user);

        client.approve_release(&proposal_id, &user);

        client.execute_release(&proposal_id);

        let proposal = client.get_release_proposal(&proposal_id);

        assert!(proposal.executed);
    }

    #[test]
    #[should_panic(expected = "NOT_ENOUGH_APPROVALS")]
    fn test_execute_without_threshold_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);

        let user1 = Address::generate(&env);
        let dest = Address::generate(&env);
        let group_id = 1;

        let proposal_id = client.propose_release(&dest, &1000, &group_id, &user1);

        // no approve
        client.execute_release(&proposal_id);
    }
}

