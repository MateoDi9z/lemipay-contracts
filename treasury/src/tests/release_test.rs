mod release_test {
    use soroban_sdk::{testutils::Address as _, Address, Env};

    use crate::errors::Error;
    use crate::{TreasuryContract, TreasuryContractClient};

    #[test]
    fn test_propose_release() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let destination = Address::generate(&env);
        let group_id = 1u64;

        client.create_treasury(&group_id, &user);
        let round_id = client.propose_fund_round(&group_id, &1000, &user);
        client.contribute_to_fund_round(&round_id, &1000, &user);

        let proposal_id = client.propose_release(&destination, &1000, &group_id, &user);

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
        let group_id = 1u64;

        client.create_treasury(&group_id, &user);
        let round_id = client.propose_fund_round(&group_id, &1000, &user);
        client.contribute_to_fund_round(&round_id, &1000, &user);
        let proposal_id = client.propose_release(&dest, &1000, &group_id, &user);

        client.approve_release(&proposal_id, &user);

        let proposal = client.get_release_proposal(&proposal_id);
        assert_eq!(proposal.approvals, 1);
        assert!(!proposal.executed);
    }

    #[test]
    fn test_destination_cannot_approve() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let group_id = 1u64;

        client.create_treasury(&group_id, &user);
        let round_id = client.propose_fund_round(&group_id, &1000, &user);
        client.contribute_to_fund_round(&round_id, &1000, &user);

        let proposal_id = client.propose_release(&user, &1000, &group_id, &user);

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
        let group_id = 1u64;

        client.create_treasury(&group_id, &user1);
        let round_id = client.propose_fund_round(&group_id, &1000, &user1);
        client.contribute_to_fund_round(&round_id, &1000, &user1);
        let proposal_id = client.propose_release(&dest, &1000, &group_id, &user1);

        let result = client.try_execute_release(&proposal_id);
        assert_eq!(result, Err(Ok(Error::NotEnoughApprovals)));
    }

    #[test]
    fn test_propose_release_rejects_when_insufficient_group_balance() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let dest = Address::generate(&env);
        let group_id = 1u64;

        client.create_treasury(&group_id, &user);

        let result = client.try_propose_release(&dest, &1000, &group_id, &user);
        assert_eq!(result, Err(Ok(Error::InsufficientGroupBalance)));
    }

    #[test]
    fn test_cancel_release_proposal() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let dest = Address::generate(&env);
        let group_id = 1u64;

        client.create_treasury(&group_id, &user);
        let round_id = client.propose_fund_round(&group_id, &1000, &user);
        client.contribute_to_fund_round(&round_id, &1000, &user);
        let proposal_id = client.propose_release(&dest, &1000, &group_id, &user);

        client.cancel_release_proposal(&proposal_id, &user);

        let result = client.try_approve_release(&proposal_id, &user);
        assert_eq!(result, Err(Ok(Error::ProposalCanceled)));

        let result = client.try_execute_release(&proposal_id);
        assert_eq!(result, Err(Ok(Error::ProposalCanceled)));
    }

    #[test]
    fn test_cancel_release_proposal_rejects_when_already_executed() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let dest = Address::generate(&env);
        let group_id = 1u64;

        client.create_treasury(&group_id, &user);
        let round_id = client.propose_fund_round(&group_id, &1000, &user);
        client.contribute_to_fund_round(&round_id, &1000, &user);
        let proposal_id = client.propose_release(&dest, &1000, &group_id, &user);
        client.approve_release(&proposal_id, &user);
        client.execute_release(&proposal_id);

        let result = client.try_cancel_release_proposal(&proposal_id, &user);
        assert_eq!(result, Err(Ok(Error::AlreadyExecuted)));
    }

    #[test]
    fn test_cancel_release_proposal_rejects_when_already_canceled() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let dest = Address::generate(&env);
        let group_id = 1u64;

        client.create_treasury(&group_id, &user);
        let round_id = client.propose_fund_round(&group_id, &1000, &user);
        client.contribute_to_fund_round(&round_id, &1000, &user);
        let proposal_id = client.propose_release(&dest, &1000, &group_id, &user);

        client.cancel_release_proposal(&proposal_id, &user);
        let result = client.try_cancel_release_proposal(&proposal_id, &user);
        assert_eq!(result, Err(Ok(Error::ProposalCanceled)));
    }
}
