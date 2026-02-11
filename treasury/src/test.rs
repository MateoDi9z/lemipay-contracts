use super::*;

use soroban_sdk::{
    testutils::{Address as _},
    Address, Bytes, Env,
};

use crate::{TreasuryContract, TreasuryContractClient};

#[test]
fn test_full_treasury_flow() {
    let env = Env::default();
    env.mock_all_auths(); // simplifica require_auth()

    // Setup contract
    let contract_id = env.register(TreasuryContract, ());
    let client = TreasuryContractClient::new(&env, &contract_id);

    let group_id: u64 = 1;

    // Create addresses
    let user1 = Address::generate(&env);

    // 1️⃣ Deposit 1000
    client.deposit(&group_id, &user1, &1000);

    // Set approvals required for group (on-chain threshold)
    client.set_approvals_required(&group_id, &user1, &1);

    // Check balance
    let balance = client.get_balance(&group_id);

    assert_eq!(balance, 1000);

    // 2️⃣ Propose expense of 400
    let description = Bytes::from_slice(&env, b"Server payment");
    let expense_id = client.propose_expense(
        &group_id,
        &user1,
        &400,
        &description,
    );

    assert_eq!(expense_id, 1);

    // 3️⃣ Approve expense (user1)
    client.approve_expense(&group_id, &expense_id, &user1);

    // 4️⃣ Execute payment (threshold 1 is stored on-chain; caller must be authorized)
    client.execute_payment(&group_id, &expense_id, &user1);

    // 5️⃣ Verify new balance
    let new_balance = client.get_balance(&group_id);

    assert_eq!(new_balance, 600);

    // 6️⃣ Verify expense marked as executed
    let expense: Expense = client.get_expense(&group_id, &1);

    assert!(expense.executed);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_propose_expense_rejects_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TreasuryContract, ());
    let client = TreasuryContractClient::new(&env, &contract_id);
    let group_id: u64 = 1;
    let user = Address::generate(&env);
    let description = Bytes::from_slice(&env, b"Zero expense");
    client.propose_expense(&group_id, &user, &0, &description);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_propose_expense_rejects_negative() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TreasuryContract, ());
    let client = TreasuryContractClient::new(&env, &contract_id);
    let group_id: u64 = 1;
    let user = Address::generate(&env);
    let description = Bytes::from_slice(&env, b"Negative expense");
    client.propose_expense(&group_id, &user, &-100, &description);
}
