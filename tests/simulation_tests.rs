use paras_nft_contract::ContractContract as Contract;
use near_sdk_sim::{
    deploy, init_simulator, to_yocto, ContractAccount, UserAccount, DEFAULT_GAS
};
use near_sdk::serde_json::json;

pub const NFT_CONTRACT_ID: &str = "nft";

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    NFT_WASM_BYTES => "out/main.wasm",
}

// Added after running simulation test -> with max token series id and 64 byte account
pub const STORAGE_MINT_ESTIMATE: u128 = 11280000000000000000000;
pub const STORAGE_CREATE_SERIES_ESTIMATE: u128 = 8540000000000000000000;
pub const STORAGE_APPROVE: u128 = 2610000000000000000000;

pub fn init() -> (UserAccount, ContractAccount<Contract>, UserAccount) {
    let root = init_simulator(None);

    let treasury = root.create_user(
        "treasury".to_string(),
        to_yocto("100"),
    );

    let nft_contract = deploy!(
        contract: Contract,
        contract_id: NFT_CONTRACT_ID,
        bytes: &NFT_WASM_BYTES,
        signer_account: root,
        init_method: new_default_meta(
            root.valid_account_id(),
            treasury.valid_account_id()
        )
    );

    root.create_user(
        "test".repeat(16),
        to_yocto("100"),
    );

    (root, nft_contract, treasury)
}

#[test]
fn simulate_create_new_series() {
    let (root, nft, _) = init();

    let initial_storage_usage = nft.account().unwrap().storage_usage;

    let outcome = root.call(
        nft.account_id(),
        "nft_create_series",
        &json!({
            "token_metadata": {
                "title": "A".repeat(200),
                "reference": "A".repeat(59),
                "media": "A".repeat(59),
                "copies": 100u64,
            },
            "price": to_yocto("1").to_string(),
            "royalty": {
                "0".repeat(64): 1000u32
            },
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        to_yocto("2")
    );

    let storage_price_for_adding_series =
        (nft.account().unwrap().storage_usage - initial_storage_usage) as u128 * 10u128.pow(19);
    println!("[CREATE SERIES] Storage price: {} yoctoNEAR", storage_price_for_adding_series);
    println!("[CREATE SERIES] Gas burnt price: {} TeraGas", outcome.gas_burnt() as f64 / 1e12);
}

#[test]
fn simulate_mint() {
    let (root, nft, _) = init();

    root.call(
        nft.account_id(),
        "nft_create_series",
        &json!({
            "token_metadata": {
                "title": "A".repeat(200),
                "reference": "A".repeat(59),
                "media": "A".repeat(59),
                "copies": 100u64,
            },
            "price": to_yocto("1").to_string(),
            "royalty": {
                "0".repeat(64): 1000u32
            },
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        to_yocto("1")
    );

    let initial_storage_usage = nft.account().unwrap().storage_usage;

    let outcome = root.call(
        nft.account_id(),
        "nft_buy",
        &json!({
            "token_series_id": "1",
            "receiver_id": "a".repeat(64),
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        to_yocto("2")
    );

    let storage_price_for_mint =
        (nft.account().unwrap().storage_usage - initial_storage_usage) as u128 * 10u128.pow(19);
    println!("[MINT] Storage price: {} yoctoNEAR", storage_price_for_mint);
    println!("[MINT] Gas burnt price: {} TeraGas", outcome.gas_burnt() as f64 / 1e12);

    let initial_storage_usage = nft.account().unwrap().storage_usage;

    let outcome = root.call(
        nft.account_id(),
        "nft_buy",
        &json!({
            "token_series_id": "1",
            "receiver_id": "b".repeat(64),
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        to_yocto("2")
    );

    let storage_price_for_mint: u128 =
        (nft.account().unwrap().storage_usage - initial_storage_usage) as u128 * 10u128.pow(19);
    println!("[MINT 2nd] Storage price: {} yoctoNEAR", storage_price_for_mint);
    println!("[MINT 2nd] Gas burnt price: {} TeraGas", outcome.gas_burnt() as f64 / 1e12);

    let initial_storage_usage = nft.account().unwrap().storage_usage;

    let outcome = root.call(
        nft.account_id(),
        "nft_buy",
        &json!({
            "token_series_id": "1",
            "receiver_id": "c".repeat(64),
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        to_yocto("2")
    );

    let storage_price_for_mint =
        (nft.account().unwrap().storage_usage - initial_storage_usage) as u128 * 10u128.pow(19);
    println!("[MINT 3nd] Storage price: {} yoctoNEAR", storage_price_for_mint);
    println!("[MINT 3nd] Gas burnt price: {} TeraGas", outcome.gas_burnt() as f64 / 1e12);
    
}

#[test]
fn simulate_approve() {
    let (root, nft, _) = init();

    root.call(
        nft.account_id(),
        "nft_create_series",
        &json!({
            "token_metadata": {
                "title": "A".repeat(200),
                "reference": "A".repeat(59),
                "media": "A".repeat(59),
                "copies": 100u64,
            },
            "price": to_yocto("1").to_string(),
            "royalty": {
                "0".repeat(64): 1000u32
            },
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        to_yocto("1")
    );

    root.call(
        nft.account_id(),
        "nft_buy",
        &json!({
            "token_series_id": u128::MAX.to_string(),
            "receiver_id": root.account_id(),
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        to_yocto("2")
    );


    let initial_storage_usage = nft.account().unwrap().storage_usage;

    let outcome = root.call(
        nft.account_id(),
        "nft_approve",
        &json!({
            "token_id": format!("1:1"),
            "account_id": "test".repeat(16),
            "msg": "{\"price\":\"3000000000000000000000000\",\"ft_token_id\":\"near\"}",
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        to_yocto("2")
    );

    let storage_price_for_approve =
        (nft.account().unwrap().storage_usage - initial_storage_usage) as u128 * 10u128.pow(19);
    println!("[APPROVE] Storage price: {} yoctoNEAR", storage_price_for_approve);
    println!("[APPROVE] Gas burnt price: {} TeraGas", outcome.gas_burnt() as f64 / 1e12);
}

#[test]
fn simulate_buy() {
    let (root, nft, treasury) = init();

    let alice = root.create_user("alice".to_string(), to_yocto("100"));

    let treasury_balance = treasury.account().unwrap().amount;

    alice.call(
        nft.account_id(),
        "nft_create_series",
        &json!({
            "token_metadata": {
                "title": "A".repeat(200),
                "reference": "A".repeat(59),
                "media": "A".repeat(59),
                "copies": 100u64,
            },
            "price": to_yocto("1").to_string(),
            "royalty": {
                "0".repeat(64): 1000u32
            },
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        to_yocto("1")
    );

    let alice_balance = alice.account().unwrap().amount;

    root.call(
        nft.account_id(),
        "nft_buy",
        &json!({
            "token_series_id": "1",
            "receiver_id": root.account_id(),
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        to_yocto("1") + STORAGE_MINT_ESTIMATE
    );

    let for_treasury = (to_yocto("1") * 500) / 10_000;
    let for_seller = to_yocto("1") - for_treasury;

    let diff_after_sell_treasury = treasury.account().unwrap().amount - treasury_balance;
    let diff_after_sell_alice = alice.account().unwrap().amount - alice_balance;

    assert_eq!(for_treasury, diff_after_sell_treasury);
    assert_eq!(for_seller, diff_after_sell_alice);
}