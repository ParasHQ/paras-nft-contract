use paras_nft_contract::ContractContract as Contract;
use near_sdk_sim::{deploy, init_simulator, to_yocto, ContractAccount, UserAccount, DEFAULT_GAS, view};
use near_sdk::serde_json::json;
use near_sdk::test_utils::test_env::alice;
use near_sdk_sim::types::AccountId;

pub const NFT_CONTRACT_ID: &str = "nft";

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    NFT_WASM_BYTES => "out/main.wasm",
}

// Added after running simulation test -> with max token series id and 64 byte account
pub const STORAGE_MINT_ESTIMATE: u128 = 11280000000000000000000;
pub const STORAGE_CREATE_SERIES_ESTIMATE: u128 = 8540000000000000000000;
pub const STORAGE_APPROVE: u128 = 2610000000000000000000;

pub fn init() -> (UserAccount, UserAccount, UserAccount, UserAccount) {
    let root = init_simulator(None);

    let treasury = root.create_user(
        "treasury".to_string(),
        to_yocto("100"),
    );

    let nft_account_id = AccountId::from(NFT_CONTRACT_ID);
    let nft_contract = root.deploy(&NFT_WASM_BYTES, nft_account_id.clone(), to_yocto("500"));


    let owner=root.create_user(
        "h".repeat(64),
        to_yocto("100"),
    );

    nft_contract.call(
        nft_contract.account_id(),
        "new_default_meta",
        &json!({
            "owner_id": owner.account_id(),
            "treasury_id": treasury.account_id(),
        })
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        0,
    );


    (root, nft_contract, treasury, owner)
}

#[test]
fn simulate_create_new_series() {
    let (root, nft, _,owner) = init();

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
            "creator_id": owner.account_id(),
            "price": to_yocto("1").to_string(),
            "royalty": {
                owner.account_id() : 1000u32
            },
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        STORAGE_CREATE_SERIES_ESTIMATE*2
    );

    let storage_price_for_adding_series =
        (nft.account().unwrap().storage_usage - initial_storage_usage) as u128 * 10u128.pow(19);
    println!("[CREATE SERIES] Storage price: {} yoctoNEAR", storage_price_for_adding_series);
    println!("[CREATE SERIES] Gas burnt price: {} TeraGas", outcome.gas_burnt() as f64 / 1e12);
}

#[test]
fn simulate_mint() {
    let (root, nft, _, owner) = init();

    owner.call(
        nft.account_id(),
        "nft_create_series",
        &json!({
            "token_metadata": {
                "title": "A".repeat(200),
                "reference": "A".repeat(59),
                "media": "A".repeat(59),
                "copies": 3u64,
            },
            "creator_id": owner.account_id(),
            "price": to_yocto("1").to_string(),
            "royalty": {
                owner.account_id() : 1000u32
            },
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        STORAGE_CREATE_SERIES_ESTIMATE*2
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

    let view_price = nft.view(
        nft.account_id(),
        "nft_get_series_price",
        &json!({
            "token_series_id": "1"
        }).to_string().into_bytes()
    ).unwrap_json_value();
    println!("price: {}", view_price.to_string());
}

#[test]
fn simulate_approve() {
    let (root, nft, _,owner) = init();

    let trst = root.create_user("trst".repeat(16), to_yocto("100"));
    owner.call(
        nft.account_id(),
        "nft_create_series",
        &json!({
            "token_metadata": {
                "title": "A".repeat(200),
                "reference": "A".repeat(59),
                "media": "A".repeat(59),
                "copies": 100u64,
            },
            "creator_id": owner.account_id(),
            "price": to_yocto("1").to_string(),
            "royalty": {
                owner.account_id() : 1000u32
            },
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        STORAGE_CREATE_SERIES_ESTIMATE*2
    );

    root.call(
        nft.account_id(),
        "nft_buy",
        &json!({
            "token_series_id": "1",
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
            "account_id": trst.account_id(),
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
    let (root, nft, treasury, owner) = init();

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
            "creator_id": alice.account_id(),
            "price": to_yocto("1").to_string(),
            "royalty": {
                owner.account_id() : 1000u32
            },
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        STORAGE_CREATE_SERIES_ESTIMATE*2
    ).assert_success();

    let alice_balance = alice.account().unwrap().amount;

    root.call(
        nft.account_id(),
        "nft_buy",
        &json!({
            "token_series_id": "1",
            "receiver_id": root.account_id(),
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        to_yocto("1")+STORAGE_MINT_ESTIMATE
    ).assert_success();

    let for_treasury = (to_yocto("1") * 500) / 10_000;
    let for_seller = to_yocto("1") - for_treasury;

    let diff_after_sell_treasury = treasury.account().unwrap().amount - treasury_balance;
    let diff_after_sell_alice = alice.account().unwrap().amount - alice_balance;
    println!("treasury before : {}", treasury_balance);
    println!("alice before : {}", alice_balance);
    println!("treasury after : {}",treasury.account().unwrap().amount);
    println!("alice after : {}",alice.account().unwrap().amount);

    assert_eq!(for_treasury, diff_after_sell_treasury);
    assert_eq!(for_seller, diff_after_sell_alice);
}

#[test]
fn simulate_buy_change_transaction_fee() {
    let (root, nft, treasury,owner) = init();

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
            "creator_id": alice.account_id(),
            "price": to_yocto("1").to_string(),
            "royalty": {
                owner.account_id() : 1000u32
            },
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        STORAGE_CREATE_SERIES_ESTIMATE
    ).assert_success();

    let alice_balance = alice.account().unwrap().amount;

    owner.call(
        nft.account_id(),
        "set_transaction_fee",
        &json!({
            "next_fee": 100
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        1
    ).assert_success();

    root.call(
        nft.account_id(),
        "nft_buy",
        &json!({
            "token_series_id": "1",
            "receiver_id": root.account_id(),
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        to_yocto("1")+STORAGE_MINT_ESTIMATE
    ).assert_success();

    // the transaction fee still 500 (locked transaction fee)
    let for_treasury = (to_yocto("1") * 500) / 10_000;
    let for_seller = to_yocto("1") - for_treasury;

    let diff_after_sell_treasury = treasury.account().unwrap().amount - treasury_balance;
    let diff_after_sell_alice = alice.account().unwrap().amount - alice_balance;

    assert_eq!(for_treasury, diff_after_sell_treasury);
    assert_eq!(for_seller, diff_after_sell_alice);
}
