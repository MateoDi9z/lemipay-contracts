#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, Vec, Map, Bytes
};

#[contracttype]
#[derive(Clone)]
pub struct Expense {
    pub amount: i128,
    pub description: Bytes,
    pub approvals: Vec<Address>,
    pub executed: bool,
}

#[contracttype]
pub enum DataKey {
    Balance(u64),
    Expenses(u64),
    /// Required number of approvals to execute a payment (per group). Must be set before execute_payment.
    ApprovalsRequired(u64),
    /// Group admin: only this address can set_approvals_required and set_group_approvers for the group.
    GroupAdmin(u64),
    /// Addresses allowed to call approve_expense for this group. Set by group admin.
    GroupApprovers(u64),
}

/// Maximum allowed expense amount (in smallest units). Kept finite so the cap is enforceable; reduce for deployment if desired.
const MAX_EXPENSE_AMOUNT: i128 = 1_000_000_000_000_000_000_000; // 10^21

#[contract]
pub struct TreasuryContract;

#[contractimpl]
impl TreasuryContract {

    /// Deposit funds into group balance
    pub fn deposit(env: Env, group_id: u64, from: Address, amount: i128) {
        from.require_auth();
        
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let mut balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(group_id))
            .unwrap_or(0);

        balance = balance
            .checked_add(amount)
            .expect("balance overflow");

        env.storage()
            .persistent()
            .set(&DataKey::Balance(group_id), &balance);
    }

    /// Propose a new expense
    pub fn propose_expense(
        env: Env,
        group_id: u64,
        proposer: Address,
        amount: i128,
        description: Bytes,
    ) -> u32 {
        proposer.require_auth();

        if amount <= 0 {
            panic!("amount must be positive");
        }
        if amount > MAX_EXPENSE_AMOUNT {
            panic!("amount exceeds maximum allowed");
        }

        let mut expenses: Map<u32, Expense> = env
            .storage()
            .persistent()
            .get(&DataKey::Expenses(group_id))
            .unwrap_or(Map::new(&env));

        let expense_id = expenses.len() + 1;

        let expense = Expense {
            amount,
            description,
            approvals: Vec::new(&env),
            executed: false,
        };

        expenses.set(expense_id, expense);

        env.storage()
            .persistent()
            .set(&DataKey::Expenses(group_id), &expenses);

        expense_id
    }

    /// Approve an expense. Only addresses in the group's approvers list may approve.
    pub fn approve_expense(
        env: Env,
        group_id: u64,
        expense_id: u32,
        approver: Address,
    ) {
        approver.require_auth();

        let approvers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::GroupApprovers(group_id))
            .expect("Group approvers not set for group");
        if !approvers.contains(&approver) {
            panic!("caller is not an approved approver for this group");
        }

        let mut expenses: Map<u32, Expense> = env
            .storage()
            .persistent()
            .get(&DataKey::Expenses(group_id))
            .expect("No expenses");

        let mut expense = expenses
            .get(expense_id)
            .expect("Expense not found");

        if expense.executed {
            panic!("Expense already executed");
        }

        if expense.approvals.contains(&approver) {
            panic!("Already approved");
        }

        expense.approvals.push_back(approver);

        expenses.set(expense_id, expense);

        env.storage()
            .persistent()
            .set(&DataKey::Expenses(group_id), &expenses);
    }

    /// Set the number of approvals required to execute a payment for a group.
    /// Only the group admin may call this. The first caller for a group becomes its admin.
    pub fn set_approvals_required(env: Env, group_id: u64, setter: Address, required: u32) {
        setter.require_auth();

        let admin_key = DataKey::GroupAdmin(group_id);
        if let Some(admin) = env.storage().persistent().get::<_, Address>(&admin_key) {
            if setter != admin {
                panic!("only group admin can set approvals required");
            }
        } else {
            env.storage().persistent().set(&admin_key, &setter);
        }

        env.storage()
            .persistent()
            .set(&DataKey::ApprovalsRequired(group_id), &required);
    }

    /// Set the list of addresses that may call approve_expense for this group. Only the group admin may call this.
    pub fn set_group_approvers(env: Env, group_id: u64, admin: Address, approvers: Vec<Address>) {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::GroupAdmin(group_id))
            .expect("Group admin not set for group");
        if admin != stored_admin {
            panic!("only group admin can set group approvers");
        }

        env.storage()
            .persistent()
            .set(&DataKey::GroupApprovers(group_id), &approvers);
    }

    /// Execute payment once the on-chain approval threshold for the group is met.
    /// Caller must be authorized. Threshold is read from storage, not from the caller.
    pub fn execute_payment(
        env: Env,
        group_id: u64,
        expense_id: u32,
        caller: Address,
    ) {
        caller.require_auth();

        let approvals_required: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::ApprovalsRequired(group_id))
            .expect("Approvals required not set for group");

        let mut expenses: Map<u32, Expense> = env
            .storage()
            .persistent()
            .get(&DataKey::Expenses(group_id))
            .expect("No expenses");

        let mut expense = expenses
            .get(expense_id)
            .expect("Expense not found");

        if expense.executed {
            panic!("Already executed");
        }

        if expense.amount <= 0 {
            panic!("Expense amount must be positive");
        }

        if expense.approvals.len() < approvals_required {
            panic!("Not enough approvals");
        }

        let mut balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(group_id))
            .unwrap_or(0);

        if balance < expense.amount {
            panic!("Insufficient balance");
        }

        balance = balance
            .checked_sub(expense.amount)
            .expect("balance underflow");
        expense.executed = true;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(group_id), &balance);

        expenses.set(expense_id, expense);
        env.storage()
            .persistent()
            .set(&DataKey::Expenses(group_id), &expenses);
    }

    /// Get Group Balance
    pub fn get_balance(env: Env, group_id: u64) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(group_id))
            .unwrap_or(0)
    }

    /// Get required number of approvals for a group (None if not set).
    pub fn get_approvals_required(env: Env, group_id: u64) -> Option<u32> {
        env.storage()
            .persistent()
            .get(&DataKey::ApprovalsRequired(group_id))
    }

    /// Get the group admin (None if not set).
    pub fn get_group_admin(env: Env, group_id: u64) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::GroupAdmin(group_id))
    }

    /// Get the list of addresses that may approve expenses for this group (None if not set).
    pub fn get_group_approvers(env: Env, group_id: u64) -> Option<Vec<Address>> {
        env.storage()
            .persistent()
            .get(&DataKey::GroupApprovers(group_id))
    }

    /// Get expense details
    pub fn get_expense(env: Env, group_id: u64, expense_id: u32) -> Expense {
        let expenses: Map<u32, Expense> = env
            .storage()
            .persistent()
            .get(&DataKey::Expenses(group_id))
            .expect("No expenses");

        expenses.get(expense_id).expect("Expense not found")
    }
}

#[cfg(test)]
mod test;