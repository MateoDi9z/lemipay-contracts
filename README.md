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
- Holding funds
- Managing deposits
- Executing treasury actions
- Enforcing rules

### 🔵 GroupContract
Responsible for:
- Creating and managing groups
- Linking groups to a treasury
- Tracking group state

---

## 📂 Project Structure

```
lemipay-contracts/
│
├── Cargo.toml (workspace)
├── treasury/
│   ├── Cargo.toml
│   └── src/lib.rs
└── group/
    ├── Cargo.toml
    └── src/lib.rs
```

This repository uses a Cargo workspace to manage multiple contracts cleanly.

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

Build Contracts

From the root directory:
```bash
cargo build --target wasm32-unknown-unknown -p treasury --release
cargo build --target wasm32-unknown-unknown -p group --release
```

Compiled WASM files will be located in:

```
target/wasm32-unknown-unknown/release/
```

## 🚀 Deploy to Stellar Testnet

Example (replace paths and keys accordingly):

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/treasury.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```
Repeat for group.wasm.

Save deployed contract IDs for frontend interaction.

## 🔁 Example Flow (MVP Demo)

1. Deploy TreasuryContract
2. Deploy GroupContract
3. Create a group
4. Deposit funds into treasury
5. Execute treasury action 
6. Verify on-chain state 

This flow is used in the live demo presentation.

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