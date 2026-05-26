#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, String, Symbol,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

#[contracttype]
pub enum DataKey {
    Admin,
    Farmer(Address),
    Batch(u64),
    BatchCounter,
    UsdcToken,
    Treasury,
}

// ─── Data Structures ─────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct FarmerProfile {
    pub wallet: Address,
    pub name: String,
    pub location: String,
    pub total_earned: i128,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct HarvestBatch {
    pub batch_id: u64,
    pub farmer: Address,
    pub crop: String,
    pub weight_kg: u32,
    pub price_per_kg: i128,
    pub amount_due: i128,
    pub delivered_at: u64,
    pub paid: bool,
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct HarvestPayContract;

#[contractimpl]
impl HarvestPayContract {
    /// Called once at deploy time. Sets admin, USDC token contract, and treasury wallet.
    pub fn initialize(env: Env, admin: Address, usdc_token: Address, treasury: Address) -> bool {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
        env.storage().instance().set(&DataKey::Treasury, &treasury);
        env.storage().instance().set(&DataKey::BatchCounter, &0u64);
        env.events().publish((Symbol::new(&env, "initialized"),), (admin, usdc_token, treasury));
        true
    }

    /// Admin registers a farmer wallet with name and location.
    pub fn register_farmer(
        env: Env,
        caller: Address,
        farmer_wallet: Address,
        name: String,
        location: String,
    ) -> bool {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        caller.require_auth();
        if caller != admin {
            panic!("unauthorized: only admin can register farmers");
        }
        if env.storage().persistent().has(&DataKey::Farmer(farmer_wallet.clone())) {
            panic!("farmer already registered");
        }
        let profile = FarmerProfile {
            wallet: farmer_wallet.clone(),
            name,
            location,
            total_earned: 0,
            active: true,
        };
        env.storage().persistent().set(&DataKey::Farmer(farmer_wallet.clone()), &profile);
        env.events().publish((Symbol::new(&env, "farmer_registered"),), (farmer_wallet,));
        true
    }

    /// Field agent logs a harvest delivery. Returns the new batch_id.
    pub fn log_harvest(
        env: Env,
        caller: Address,
        farmer: Address,
        crop: String,
        weight_kg: u32,
        price_per_kg: i128,
    ) -> u64 {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        caller.require_auth();
        if caller != admin {
            panic!("unauthorized: only admin can log harvests");
        }
        let profile: FarmerProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Farmer(farmer.clone()))
            .expect("farmer not registered");
        if !profile.active {
            panic!("farmer account is inactive");
        }
        if weight_kg == 0 {
            panic!("weight must be greater than zero");
        }
        if price_per_kg <= 0 {
            panic!("price must be positive");
        }
        let amount_due = (weight_kg as i128) * price_per_kg;
        let batch_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::BatchCounter)
            .unwrap_or(0u64)
            + 1;
        env.storage().instance().set(&DataKey::BatchCounter, &batch_id);
        let batch = HarvestBatch {
            batch_id,
            farmer: farmer.clone(),
            crop: crop.clone(),
            weight_kg,
            price_per_kg,
            amount_due,
            delivered_at: env.ledger().timestamp(),
            paid: false,
        };
        env.storage().persistent().set(&DataKey::Batch(batch_id), &batch);
        env.events().publish(
            (Symbol::new(&env, "harvest_logged"),),
            (batch_id, farmer, crop, weight_kg, amount_due),
        );
        batch_id
    }

    /// Treasury releases USDC directly to the farmer. Core MVP transaction.
    pub fn settle_batch(env: Env, caller: Address, batch_id: u64) -> i128 {
        let treasury: Address = env.storage().instance().get(&DataKey::Treasury).unwrap();
        caller.require_auth();
        if caller != treasury {
            panic!("unauthorized: only treasury can settle batches");
        }
        let mut batch: HarvestBatch = env
            .storage()
            .persistent()
            .get(&DataKey::Batch(batch_id))
            .expect("batch not found");
        if batch.paid {
            panic!("batch already settled");
        }
        let mut profile: FarmerProfile = env
            .storage()
            .persistent()
            .get(&DataKey::Farmer(batch.farmer.clone()))
            .expect("farmer profile missing");
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&treasury, &batch.farmer, &batch.amount_due);
        batch.paid = true;
        env.storage().persistent().set(&DataKey::Batch(batch_id), &batch);
        profile.total_earned += batch.amount_due;
        env.storage().persistent().set(&DataKey::Farmer(batch.farmer.clone()), &profile);
        env.events().publish(
            (Symbol::new(&env, "batch_settled"),),
            (batch_id, batch.farmer.clone(), batch.amount_due),
        );
        batch.amount_due
    }

    pub fn get_farmer(env: Env, farmer: Address) -> FarmerProfile {
        env.storage().persistent().get(&DataKey::Farmer(farmer)).expect("farmer not found")
    }

    pub fn get_batch(env: Env, batch_id: u64) -> HarvestBatch {
        env.storage().persistent().get(&DataKey::Batch(batch_id)).expect("batch not found")
    }

    pub fn batch_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::BatchCounter).unwrap_or(0)
    }
}

// Link to the separate test file
#[cfg(test)]
mod test;