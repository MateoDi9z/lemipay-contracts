//! # Treasury contract
//!
//! Group-based treasury: fund rounds, release proposals, per-group balance.
//!
//! ## ⚠️ USDC inflow — único flujo soportado
//!
//! **El único flujo de entrada de USDC al contrato es `contribute_to_fund_round`.**
//!
//! - No se debe hacer **transfer directo** de USDC al contrato (desde billetera o otro contrato).
//! - En Soroban, una transferencia directa la ejecuta solo el contrato del token; este contrato
//!   **no se invoca** en ese caso, por lo que no es posible rechazar ni hacer panic ante un depósito
//!   directo desde el código del Treasury.
//! - Cualquier USDC enviado por transfer directo **no** se refleja en ningún `GroupBalance`, no se
//!   asigna a ningún grupo y con la lógica actual **no hay forma de asignarlo después**. Ese saldo
//!   queda en el contrato sin uso (execute_release sigue comprobando balance del token y del grupo,
//!   así que no se puede "robar", pero el saldo directo queda inutilizable).
//!
//! Integradores y frontends deben asegurar que el único depósito sea vía `contribute_to_fund_round`.

#![no_std]

mod config;
mod contract;
mod clients;
mod errors;
mod events;
mod helpers;
pub mod interface;
mod storage;
#[cfg(test)]
mod tests;
mod types;

pub use crate::errors::Error;
pub use crate::types::{FundRound, ReleaseProposal};

use crate::interface::ITreasuryContract;
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};

#[contract]
pub struct TreasuryContract;

#[contractimpl(contracttrait)]
impl ITreasuryContract for TreasuryContract {
    fn create_treasury(env: Env, group_id: u64, user: Address) -> Result<(), Error> {
        contract::create_treasury(env, group_id, user)
    }

    fn propose_release(
        env: Env,
        destination: Address,
        amount: i128,
        group_id: u64,
        user: Address,
    ) -> Result<u64, Error> {
        contract::propose_release(env, destination, amount, group_id, user)
    }

    fn approve_release(
        env: Env,
        release_proposal_id: u64,
        user: Address,
    ) -> Result<(), Error> {
        contract::approve_release(env, release_proposal_id, user)
    }

    fn execute_release(env: Env, release_proposal_id: u64) -> Result<(), Error> {
        contract::execute_release(env, release_proposal_id)
    }

    fn cancel_release_proposal(
        env: Env,
        release_proposal_id: u64,
        user: Address,
    ) -> Result<(), Error> {
        contract::cancel_release_proposal(env, release_proposal_id, user)
    }

    fn propose_fund_round(
        env: Env,
        group_id: u64,
        total_amount: i128,
        user: Address,
    ) -> Result<u64, Error> {
        contract::propose_fund_round(env, group_id, total_amount, user)
    }

    fn contribute_to_fund_round(
        env: Env,
        round_id: u64,
        amount: i128,
        user: Address,
    ) -> Result<(), Error> {
        contract::contribute_to_fund_round(env, round_id, amount, user)
    }

    fn withdraw_contribution(
        env: Env,
        round_id: u64,
        user: Address,
    ) -> Result<(), Error> {
        contract::withdraw_contribution(env, round_id, user)
    }

    fn get_group_rounds(env: Env, group_id: u64) -> Vec<u64> {
        contract::get_group_rounds(env, group_id)
    }

    fn get_fund_round(env: Env, round_id: u64) -> Result<crate::types::FundRound, Error> {
        contract::get_fund_round(env, round_id)
    }

    fn get_user_contribution(env: Env, round_id: u64, user: Address) -> i128 {
        contract::get_user_contribution(env, round_id, user)
    }

    fn get_release_proposals_of_group(env: Env, group_id: u64) -> Vec<u64> {
        contract::get_release_proposals_of_group(env, group_id)
    }

    fn get_release_proposal(
        env: Env,
        proposal_id: u64,
    ) -> Result<crate::types::ReleaseProposal, Error> {
        contract::get_release_proposal(env, proposal_id)
    }

    fn check_treasury_id(env: Env, group_id: u64) -> bool {
        contract::check_treasury_id(env, group_id)
    }

    fn get_group_balance(env: Env, group_id: u64) -> i128 {
        contract::get_group_balance(env, group_id)
    }

    fn has_sufficient_group_balance(env: Env, group_id: u64, amount: i128) -> bool {
        contract::has_sufficient_group_balance(env, group_id, amount)
    }
}
