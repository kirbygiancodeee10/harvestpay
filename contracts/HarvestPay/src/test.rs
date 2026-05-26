se soroban_sdk::{testutils::Address as _, token, Address, Env, String};
use crate::{HarvestPayContract, HarvestPayContractClient};

// ─── Shared setup ────────────────────────────────────────────────────────────

fn setup() -> (
    Env,
    HarvestPayContractClient<'static>,
    Address,
    Address,
    token::StellarAssetClient<'static>,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let usdc_token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let usdc_asset_client = token::StellarAssetClient::new(&env, &usdc_token_addr);
    usdc_asset_client.mint(&treasury, &1_000_000_000i128);
    let contract_id = env.register(HarvestPayContract, ());
    let client = HarvestPayContractClient::new(&env, &contract_id);
    client.initialize(&admin, &usdc_token_addr, &treasury);
    (env, client, admin, treasury, usdc_asset_client, usdc_token_addr)
}

// ─── Test 1: Happy path ───────────────────────────────────────────────────────

#[test]
fn test_happy_path_register_log_settle() {
    let (env, client, admin, treasury, usdc_client, usdc_addr) = setup();
    let farmer = Address::generate(&env);
    let registered = client.register_farmer(
        &admin, &farmer,
        &String::from_str(&env, "Maria Santos"),
        &String::from_str(&env, "Brgy. Calaanan, Cagayan de Oro"),
    );
    assert!(registered);
    let price_per_kg: i128 = 15_000_000;
    let weight: u32 = 150;
    let expected_amount: i128 = weight as i128 * price_per_kg;
    usdc_client.mint(&treasury, &expected_amount);
    let batch_id = client.log_harvest(
        &admin, &farmer,
        &String::from_str(&env, "palay"),
        &weight, &price_per_kg,
    );
    assert_eq!(batch_id, 1);
    let amount_paid = client.settle_batch(&treasury, &batch_id);
    assert_eq!(amount_paid, expected_amount);
    let token_client = token::Client::new(&env, &usdc_addr);
    assert_eq!(token_client.balance(&farmer), expected_amount);
}

// ─── Test 2: Double settlement rejected ──────────────────────────────────────

#[test]
#[should_panic(expected = "batch already settled")]
fn test_double_settlement_rejected() {
    let (env, client, admin, treasury, usdc_client, _) = setup();
    let farmer = Address::generate(&env);
    client.register_farmer(
        &admin, &farmer,
        &String::from_str(&env, "Juan dela Cruz"),
        &String::from_str(&env, "Brgy. Poblacion, Bukidnon"),
    );
    let amount = 5u32 as i128 * 10_000_000i128;
    usdc_client.mint(&treasury, &(amount * 2));
    let batch_id = client.log_harvest(
        &admin, &farmer,
        &String::from_str(&env, "cacao"),
        &5u32, &10_000_000i128,
    );
    client.settle_batch(&treasury, &batch_id);
    client.settle_batch(&treasury, &batch_id); // must panic
}

// ─── Test 3: State verification after settlement ──────────────────────────────

#[test]
fn test_state_correct_after_settlement() {
    let (env, client, admin, treasury, usdc_client, _) = setup();
    let farmer = Address::generate(&env);
    client.register_farmer(
        &admin, &farmer,
        &String::from_str(&env, "Ana Reyes"),
        &String::from_str(&env, "Brgy. San Jose, Nueva Ecija"),
    );
    let price_per_kg: i128 = 20_000_000;
    let weight: u32 = 200;
    let expected_amount = weight as i128 * price_per_kg;
    usdc_client.mint(&treasury, &expected_amount);
    let batch_id = client.log_harvest(
        &admin, &farmer,
        &String::from_str(&env, "banana"),
        &weight, &price_per_kg,
    );
    assert!(!client.get_batch(&batch_id).paid);
    client.settle_batch(&treasury, &batch_id);
    let batch_after = client.get_batch(&batch_id);
    assert!(batch_after.paid);
    assert_eq!(batch_after.amount_due, expected_amount);
    assert_eq!(client.get_farmer(&farmer).total_earned, expected_amount);
}

// ─── Test 4: Unauthorized settler rejected ────────────────────────────────────

#[test]
#[should_panic(expected = "unauthorized: only treasury can settle batches")]
fn test_unauthorized_settler_rejected() {
    let (env, client, admin, treasury, usdc_client, _) = setup();
    let farmer = Address::generate(&env);
    client.register_farmer(
        &admin, &farmer,
        &String::from_str(&env, "Pedro Cruz"),
        &String::from_str(&env, "Brgy. Magsaysay, Davao del Sur"),
    );
    usdc_client.mint(&treasury, &50_000_000i128);
    let batch_id = client.log_harvest(
        &admin, &farmer,
        &String::from_str(&env, "palay"),
        &5u32, &10_000_000i128,
    );
    let attacker = Address::generate(&env);
    client.settle_batch(&attacker, &batch_id); // must panic
}

// ─── Test 5: Unregistered farmer harvest rejected ─────────────────────────────

#[test]
#[should_panic(expected = "farmer not registered")]
fn test_unregistered_farmer_harvest_rejected() {
    let (env, client, admin, _, _, _) = setup();
    let ghost = Address::generate(&env);
    client.log_harvest(
        &admin, &ghost,
        &String::from_str(&env, "palay"),
        &100u32, &10_000_000i128,
    );
}
