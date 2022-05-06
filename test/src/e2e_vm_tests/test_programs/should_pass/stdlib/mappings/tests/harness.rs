use fuel_tx::{ContractId, Salt};
use fuels_abigen_macro::abigen;
use fuels::prelude::*;
use fuels::test_helpers;

// Load abi from json
abigen!(MyContract, "out/debug/mapping-abi.json");

async fn get_contract_instance() -> (MyContract, ContractId) {
    // Deploy the compiled contract
    let salt = Salt::from([0u8; 32]);
    let compiled = Contract::load_sway_contract("./out/debug/mapping.bin", salt).unwrap();

    // Launch a local network and deploy the contract
    let (provider, wallet) = test_helpers::setup_test_provider_and_wallet().await;

    let id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    let instance = MyContract::new(id.to_string(), provider, wallet);

    (instance, id)
}

#[tokio::test]
async fn can_get_contract_id() {
    let (instance, _id) = get_contract_instance().await;

    instance.insert_into_mapping1(1, 42).call().await;
    assert_eq!(instance.get_from_mapping1(1).call().await.unwrap().value, 42);

    instance.insert_into_mapping1(2, 77).call().await;
    assert_eq!(instance.get_from_mapping1(2).call().await.unwrap().value, 77);
    assert_eq!(instance.get_from_mapping1(1).call().await.unwrap().value, 42);
}
