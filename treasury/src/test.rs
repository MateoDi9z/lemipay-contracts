#[cfg(test)]
mod test {

    use soroban_sdk::{
        testutils::{Address as _},
        Address, Env, Vec
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
    fn test_contribute_updates_amount() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let group_id = 1;

        let round_id = client.propose_fund_round(&group_id, &1000, &user);

        client.contribute_to_fund_round(&round_id, &group_id, &user, &400);

        let round = client.get_fund_round(&round_id);

        assert_eq!(round.funded_amount, 400);
        assert!(!round.completed);
    }

    #[test]
    fn test_round_completes_when_target_reached() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TreasuryContract, ());
        let client = TreasuryContractClient::new(&env, &contract_id);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        let group_id = 1;

        let round_id = client.propose_fund_round(&group_id, &1000, &user1);

        client.contribute_to_fund_round(&round_id, &group_id, &user1, &600);
        client.contribute_to_fund_round(&round_id, &group_id, &user2, &400);

        let round = client.get_fund_round(&round_id);

        assert_eq!(round.funded_amount, 1000);
        assert!(round.completed);
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

    // #[test]
    // fn test_full_treasury_flow() {
    //     let env = Env::default();
    //     env.mock_all_auths(); // simplifica require_auth()
    //
    //     // Setup contract
    //     let contract_id = env.register(TreasuryContract, ());
    //     let client = TreasuryContractClient::new(&env, &contract_id);
    //
    //     let group_id: u64 = 1;
    //
    //     // Create addresses
    //     let user1 = Address::generate(&env);
    //
    //     // 1️⃣ Deposit 1000
    //     client.deposit(&group_id, &user1, &1000);
    //
    //     // Set approvals required for group (on-chain threshold); user1 becomes group admin
    //     client.set_approvals_required(&group_id, &user1, &1);
    //
    //     // Admin sets who may approve expenses (user1 in this case)
    //     let approvers = soroban_sdk::vec![&env, user1.clone()];
    //     client.set_group_approvers(&group_id, &user1, &approvers);
    //
    //     // Check balance
    //     let balance = client.get_balance(&group_id);
    //
    //     assert_eq!(balance, 1000);
    //
    //     // 2️⃣ Propose expense of 400
    //     let description = Bytes::from_slice(&env, b"Server payment");
    //     let expense_id = client.propose_expense(
    //         &group_id,
    //         &user1,
    //         &400,
    //         &description,
    //     );
    //
    //     assert_eq!(expense_id, 1);
    //
    //     // 3️⃣ Approve expense (user1)
    //     client.approve_expense(&group_id, &expense_id, &user1);
    //
    //     // 4️⃣ Execute payment (threshold 1 is stored on-chain; caller must be authorized)
    //     client.execute_payment(&group_id, &expense_id, &user1);
    //
    //     // 5️⃣ Verify new balance
    //     let new_balance = client.get_balance(&group_id);
    //
    //     assert_eq!(new_balance, 600);
    //
    //     // 6️⃣ Verify expense marked as executed
    //     let expense: Expense = client.get_expense(&group_id, &1);
    //
    //     assert!(expense.executed);
    // }
    //
    // #[test]
    // #[should_panic(expected = "only group admin can set approvals required")]
    // fn test_set_approvals_required_requires_admin() {
    //     let env = Env::default();
    //     env.mock_all_auths();
    //     let contract_id = env.register(TreasuryContract, ());
    //     let client = TreasuryContractClient::new(&env, &contract_id);
    //     let group_id: u64 = 1;
    //     let admin = Address::generate(&env);
    //     let other = Address::generate(&env);
    //
    //     // Admin sets approvals first (becomes group admin)
    //     client.set_approvals_required(&group_id, &admin, &1);
    //
    //     // Other address must not be able to change it
    //     client.set_approvals_required(&group_id, &other, &0);
    // }
    //
    // #[test]
    // #[should_panic(expected = "caller is not an approved approver for this group")]
    // fn test_approve_expense_requires_approver_membership() {
    //     let env = Env::default();
    //     env.mock_all_auths();
    //     let contract_id = env.register(TreasuryContract, ());
    //     let client = TreasuryContractClient::new(&env, &contract_id);
    //     let group_id: u64 = 1;
    //     let admin = Address::generate(&env);
    //     let outsider = Address::generate(&env);
    //
    //     client.set_approvals_required(&group_id, &admin, &1);
    //     let approvers = soroban_sdk::vec![&env, admin.clone()];
    //     client.set_group_approvers(&group_id, &admin, &approvers);
    //
    //     let description = Bytes::from_slice(&env, b"Expense");
    //     client.propose_expense(&group_id, &admin, &100, &description);
    //
    //     // Outsider is not in approvers list
    //     client.approve_expense(&group_id, &1, &outsider);
    // }

    // #[test]
    // #[should_panic(expected = "amount must be positive")]
    // fn test_propose_expense_rejects_negative() {
    //     let env = Env::default();
    //     env.mock_all_auths();
    //     let contract_id = env.register(TreasuryContract, ());
    //     let client = TreasuryContractClient::new(&env, &contract_id);
    //     let group_id: u64 = 1;
    //     let user = Address::generate(&env);
    //     let description = Bytes::from_slice(&env, b"Negative expense");
    //     client.propose_expense(&group_id, &user, &-100, &description);
    // }

}

