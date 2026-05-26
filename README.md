# HarvestPay

** On-chain harvest logging and instant USDC payouts for SEA farming cooperatives.

The Problem
A palay farmer in Bukidnon, Philippines delivers 150 kg of rice to the local cooperative depot. The field agent writes the delivery in a paper ledger. The farmer receives a handwritten receipt and waits 2–4 weeks for a cash payout — often subject to manual errors, middlemen skimming, and no audit trail. If the cooperative's treasurer is unavailable, payment can be delayed indefinitely.
The Solution
HarvestPay replaces the paper ledger with a Soroban smart contract. A field agent logs the harvest delivery on-chain (crop type, weight, agreed price). The cooperative treasury sends one transaction to settle the batch — USDC transfers directly to the farmer's Stellar wallet in under 5 seconds with a $0.0001 fee. The farmer sees the money arrive in their phone wallet before they leave the depot.
No banks. No middlemen. No paper. Full audit trail.

Vision & Purpose
HarvestPay is designed for the 10+ million smallholder farming families in Southeast Asia who are locked out of formal financial infrastructure. By anchoring payment to verifiable on-chain harvest data, cooperatives can also use HarvestBatch records as proof of income for lending — unlocking a future layer of DeFi credit for farmers who have never had a credit score.
The contract is intentionally minimal so it can be deployed and operated by a cooperative administrator with basic CLI skills, and extended later with Soroban multi-sig, oracle pricing feeds, and DEX-based hedging.

Stellar Features Used
FeatureWhyUSDC / Stellar Asset Contract (SAC)Stable settlement currency — farmers are paid in USDC, not volatile XLMSoroban Smart ContractsEnforce rules on-chain: only registered farmers get paid, batches can only be settled onceTrustlinesFarmer wallets must hold a USDC trustline before payment can landNative token speedLedger closes in ~5 seconds — payment confirmed before the farmer walks outOn-chain eventsharvest_logged and batch_settled events feed off-chain dashboards for cooperatives

Project Structure
harvest_pay/
├── src/
│   ├── lib.rs      # Contract: initialize, register_farmer, log_harvest, settle_batch
│   └── test.rs     # 5 tests covering happy path, edge cases, and state verification
├── Cargo.toml
└── README.md

Prerequisites
ToolVersionRust≥ 1.74 (install via rustup)Soroban CLI≥ 21.0.0wasm32-unknown-unknown targetrustup target add wasm32-unknown-unknown
Install Soroban CLI:
bashcargo install --locked soroban-cli --features opt

Build
bashsoroban contract build
# Output: target/wasm32-unknown-unknown/release/harvest_pay.wasm
Optimise the Wasm binary further:
bashsoroban contract optimize --wasm target/wasm32-unknown-unknown/release/harvest_pay.wasm

Test
Run all 5 tests locally (no network required):
bashcargo test --features testutils
Expected output:
running 5 tests
test tests::test_happy_path_register_log_settle ... ok
test tests::test_double_settlement_rejected ... ok
test tests::test_state_correct_after_settlement ... ok
test tests::test_unauthorized_settler_rejected ... ok
test tests::test_unregistered_farmer_harvest_rejected ... ok

test result: ok. 5 passed; 0 failed

Deploy to Testnet
1. Configure your identity
bashsoroban keys generate --global deployer --network testnet
soroban keys fund deployer --network testnet
2. Deploy the contract
bashsoroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/harvest_pay.optimized.wasm \
  --source deployer \
  --network testnet
# Outputs: <CONTRACT_ID>
3. Initialize
bashsoroban contract invoke \
  --id <CONTRACT_ID> \
  --source deployer \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --usdc_token <USDC_SAC_TESTNET_ADDRESS> \
  --treasury <TREASURY_ADDRESS>

Sample CLI Invocations (MVP Flow)
Register a farmer
bashsoroban contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network testnet \
  -- register_farmer \
  --caller <ADMIN_ADDRESS> \
  --farmer_wallet GBFR3KKVKBHB6WDSTQ7X2BKPZH3JNVXFCXLQ7FDVKJQMWP3STZXR2A \
  --name "Maria Santos" \
  --location "Brgy. Calaanan, Cagayan de Oro"
Log a harvest delivery
bashsoroban contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network testnet \
  -- log_harvest \
  --caller <ADMIN_ADDRESS> \
  --farmer GBFR3KKVKBHB6WDSTQ7X2BKPZH3JNVXFCXLQ7FDVKJQMWP3STZXR2A \
  --crop "palay" \
  --weight_kg 150 \
  --price_per_kg 15000000
# price_per_kg = 15_000_000 stroops = 1.50 USDC
Settle the batch (treasury releases USDC to farmer)
bashsoroban contract invoke \
  --id <CONTRACT_ID> \
  --source treasury \
  --network testnet \
  -- settle_batch \
  --caller <TREASURY_ADDRESS> \
  --batch_id 1
# Farmer receives 225 USDC (150 kg × 1.50 USDC) in ~5 seconds
Query a batch record
bashsoroban contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- get_batch \
  --batch_id 1

Timeline
MilestoneDescriptionDay 1Contract deployed to testnet, farmer registration workingDay 2Harvest logging + batch settlement tested end-to-endDay 3Simple React/Next.js front-end for field agentsDay 4Demo script: register → log → settle in < 2 minutesDay 5Polish, README, pitch deck

Roadmap (Post-Hackathon)

Multi-sig treasury — require 2-of-3 cooperative board signatures to settle
Oracle price feed — integrate a Soroban oracle for real-time crop prices instead of manual entry
Credit layer — use batch history as collateral proof for micro-loans via a Soroban lending pool
Offline-first mobile app — field agents queue logs offline; sync + submit when connectivity returns
DEX integration — cooperative can swap USDC ↔ PHP peso via Stellar's built-in DEX, settling in local currency

## Contract Details

- Contract Address: CBLU4IUASQ4WUMOXBFLZRSBBLILGOH33GS4LUPKFBCCCMJCDQNMF7G2M

  <img width="1920" height="946" alt="image" src="https://github.com/user-attachments/assets/356be630-ffc5-4afa-936d-f5b645bcfd07" />


