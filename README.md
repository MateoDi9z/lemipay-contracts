# 🚀 LemiPay — Smart Contracts MVP (Stellar Testnet)

LemiPay is a decentralized group treasury system built on **Stellar Soroban**.

This MVP demonstrates how groups can coordinate funds transparently and execute programmable treasury logic on-chain.

Built for rapid validation, clarity, and real blockchain interaction.

---

## 🌍 Vision

LemiPay enables:

- 🧑‍🤝‍🧑 Group-based fund coordination
- 🔐 Trustless treasury management
- ⚙️ Programmable execution logic

This repository contains the smart contracts powering the MVP.

---

## 🏗 Architecture

The system is composed of two main contracts:

### 🟡 TreasuryContract
Responsible for:
- Holding funds (USDC) with **per-group balance tracking**
- Fund rounds (propose, contribute, complete)
- Release proposals (propose, approve, execute) with configurable approval rules
- Enforcing group membership via GroupContract

### 🔵 GroupContract
Responsible for:
- Creating and managing groups (members, approvals required)
- Providing membership and approval-rule data to the Treasury

---

## 📂 Project Structure

```
lemipay/
├── Cargo.toml              (workspace)
├── treasury/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          (contract entrypoint)
│       ├── config.rs      (testnet/mainnet addresses)
│       ├── errors.rs       (custom Error enum)
│       ├── helpers.rs      (membership, treasury checks)
│       ├── storage.rs      (DataKey)
│       ├── types.rs        (ReleaseProposal, FundRound)
│       ├── clients/        (Group contract client)
│       └── tests/          (*_test.rs modules)
└── group/
    ├── Cargo.toml
    └── src/lib.rs
```

The workspace manages both contracts. Treasury tests live under `treasury/src/tests/`.

---

## ⚙️ Requirements

- Rust (latest stable)
- Soroban CLI
- Stellar testnet account funded
- wasm target installed

Install wasm target:

```bash
rustup target add wasm32-unknown-unknown
```

Build contracts (from repo root):

```bash
cargo build --target wasm32-unknown-unknown -p treasury_contract --release
cargo build --target wasm32-unknown-unknown -p group_contract --release
```

Run tests:

```bash
cargo test -p treasury_contract
cargo test -p group_contract
```

Compiled WASM files:

```
target/wasm32-unknown-unknown/release/
  treasury_contract.wasm
  group_contract.wasm
```

## 🚀 Deploy to Stellar Testnet

Example (replace paths and keys accordingly):

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/treasury_contract.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

Repeat for `group_contract.wasm`. Save deployed contract IDs for frontend and configure Treasury’s `config` (e.g. testnet feature) with the Group contract ID.

## 🔁 Example Flow (MVP Demo)

1. Deploy GroupContract, then TreasuryContract (Treasury needs Group’s contract ID in config).
2. Create a group (GroupContract).
3. Create a treasury for that group (TreasuryContract).
4. Propose a fund round, contribute USDC (per-group balance increases).
5. Propose a release, gather approvals, execute release (per-group balance decreases).
6. Verify on-chain state (e.g. `get_group_balance`, proposals).

## 🎯 MVP Scope

This is a minimal implementation focused on:

- Demonstrating real on-chain interaction 
- Validating group treasury coordination 
- Enabling live demo execution

Production features such as security hardening, governance layers, and advanced validation are intentionally out of scope for this MVP.

## 🧠 Why Stellar Soroban?

- Fast execution 
- Low fees 
- Native asset support 
- Built for financial infrastructure

## 👥 Team

LemiPay Core Team

## ⚠️ Disclaimer

This code is experimental and built for demonstration purposes.
Do not use in production environments.