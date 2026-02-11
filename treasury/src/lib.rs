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
}

#[contract]
pub struct TreasuryContract;

#[contractimpl]
impl TreasuryContract {

    /// Deposit funds into group balance
    pub fn deposit(env: Env, group_id: u64, from: Address, amount: i128) {
        from.require_auth();

        let mut balance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(group_id))
            .unwrap_or(0);

        balance += amount;

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

    /// Approve an expense
    pub fn approve_expense(
        env: Env,
        group_id: u64,
        expense_id: u32,
        approver: Address,
    ) {
        approver.require_auth();

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

    /// Execute payment once approvals are enough
    /// (approval count validation happens off-chain or via GroupContract call later)
    pub fn execute_payment(
        env: Env,
        group_id: u64,
        expense_id: u32,
        approvals_required: u32,
    ) {
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

        balance -= expense.amount;
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