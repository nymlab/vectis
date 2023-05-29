use cosmwasm_std::coins;
use vectis_contract_tests::common::base_common::HubChainSuite;
use vectis_contract_tests::common::common::*;
use vectis_contract_tests::common::plugins::*;

#[test]
fn cannot_register_plugins_without_fee() {
    let mut suite = HubChainSuite::init().unwrap();
    let funds = &[];

    let err = suite
        .register_plugin_mocked(&suite.controller.clone(), funds)
        .unwrap_err();

    assert_eq!(
        err,
        PRegistryContractError::IncorrectFees(coin(REGISTRY_FEE, DENOM))
    );
}

#[test]
fn no_reviewer_cannot_register_plugins() {
    let mut suite = HubChainSuite::init().unwrap();

    let err = suite
        .register_plugin_mocked(&suite.controller.clone(), &[coin(REGISTRY_FEE, DENOM)])
        .unwrap_err();

    assert_eq!(err, PRegistryContractError::Unauthorized);
}

#[test]
fn no_reviewer_cannot_unregister_plugins() {
    let mut suite = HubChainSuite::init().unwrap();

    let err = suite
        .unregister_plugin(&suite.controller.clone(), 1)
        .unwrap_err();

    assert_eq!(err, PRegistryContractError::Unauthorized);
}

#[test]
fn no_reviewer_cannot_update_plugins() {
    let mut suite = HubChainSuite::init().unwrap();

    let err = suite
        .update_plugin(
            &suite.controller.clone(),
            1,
            None,
            None,
            None,
            None,
            "v-new-version".into(),
        )
        .unwrap_err();

    assert_eq!(err, PRegistryContractError::Unauthorized);
}

#[test]
fn no_deployer_cannot_update_registry_fee() {
    let mut suite = HubChainSuite::init().unwrap();

    let err = suite
        .update_registry_fee(&suite.controller.clone(), coin(100_000, DENOM))
        .unwrap_err();

    assert_eq!(err, PRegistryContractError::Unauthorized);
}

#[test]
fn can_update_registry_fees_to_zero_registration_fee_works() {
    let mut suite = HubChainSuite::init().unwrap();

    suite
        .update_registry_fee(&suite.deployer.clone(), coin(0, DENOM))
        .unwrap();

    suite
        .register_plugin_mocked(&suite.plugin_committee.clone(), &[])
        .unwrap();

    let resp = suite.query_registered_plugins(None, None).unwrap();

    // check there is a plugin
    assert_eq!(resp.total, 1);
}

#[test]
fn can_update_install_fees() {
    let mut suite = HubChainSuite::init().unwrap();

    suite
        .update_install_fee(&suite.deployer.clone(), coin(0, DENOM))
        .unwrap();
}
#[test]
fn not_deployer_cannot_update_deployer_addr() {
    let mut suite = HubChainSuite::init().unwrap();

    let err = suite
        .update_deployer_addr(&suite.controller.clone(), "test")
        .unwrap_err();

    assert_eq!(err, PRegistryContractError::Unauthorized);
}

#[test]
fn reviewer_should_be_able_to_register_plugins() {
    let mut suite = HubChainSuite::init().unwrap();

    let deployer_previous_balance = suite.query_balance(&suite.deployer.clone()).unwrap();

    suite
        .register_plugin_mocked(&suite.plugin_committee.clone(), &coins(REGISTRY_FEE, DENOM))
        .unwrap();

    let deployer_current_balance = suite.query_balance(&suite.deployer.clone()).unwrap();
    let resp = suite.query_registered_plugins(None, None).unwrap();

    // check there is a plugin
    assert_eq!(resp.total, 1);

    // check the deployer received the register fee;
    assert_eq!(
        deployer_current_balance.amount,
        deployer_previous_balance
            .amount
            .checked_add(Uint128::from(REGISTRY_FEE))
            .unwrap()
    )
}

#[test]
fn reviewer_should_be_able_to_unregister_plugins() {
    let mut suite = HubChainSuite::init().unwrap();

    suite
        .register_plugin_mocked(&suite.plugin_committee.clone(), &coins(REGISTRY_FEE, DENOM))
        .unwrap();

    let resp = suite.query_registered_plugins(None, None).unwrap();

    assert_eq!(resp.total, 1);

    suite
        .unregister_plugin(&suite.plugin_committee.clone(), 1)
        .unwrap();

    let resp = suite.query_registered_plugins(None, None).unwrap();

    assert_eq!(resp.total, 0);
    assert_eq!(resp.current_plugin_id, 1);
}

#[test]
fn after_unregister_current_id_does_not_change() {
    let mut suite = HubChainSuite::init().unwrap();

    // should have 2 plugins
    suite
        .register_plugin_mocked(&suite.plugin_committee.clone(), &coins(REGISTRY_FEE, DENOM))
        .unwrap();
    let resp = suite.query_registered_plugins(None, None).unwrap();
    assert_eq!(resp.total, 1);
    suite
        .unregister_plugin(&suite.plugin_committee.clone(), 1)
        .unwrap();
    let mut resp = suite.query_registered_plugins(None, None).unwrap();
    assert_eq!(resp.total, 0);
    assert_eq!(resp.current_plugin_id, 1);

    suite
        .register_plugin_mocked(&suite.plugin_committee.clone(), &coins(REGISTRY_FEE, DENOM))
        .unwrap();
    resp = suite.query_registered_plugins(None, None).unwrap();
    assert_eq!(resp.total, 1);
    assert_eq!(resp.plugins[0].id, 2)
}

#[test]
fn reviewer_should_be_able_to_update_plugins() {
    let mut suite = HubChainSuite::init().unwrap();

    suite
        .register_plugin_mocked(&suite.plugin_committee.clone(), &coins(REGISTRY_FEE, DENOM))
        .unwrap();

    let plugin = suite.query_plugin(1).unwrap().unwrap();

    let new_code_id = 2;
    let new_creator = "creator";
    let new_ipfs_hash = "new_ipfs_hash";
    let new_checksum = "new_checksum";
    let new_version = "new_version";

    suite
        .update_plugin(
            &suite.plugin_committee.clone(),
            plugin.id,
            Some(new_code_id),
            Some(new_creator.to_string()),
            Some(new_ipfs_hash.to_string()),
            Some(new_checksum.to_string()),
            new_version.to_string(),
        )
        .unwrap();

    let plugin_after = suite.query_plugin(1).unwrap().unwrap();

    let plugin_version_details = plugin.versions.get(&plugin.latest_version).unwrap();
    assert_ne!(plugin_version_details.ipfs_hash, new_ipfs_hash);
    assert_ne!(plugin_version_details.checksum, new_checksum);
    assert_ne!(plugin_version_details.code_id, new_code_id);
    assert_ne!(plugin.latest_version, new_version);

    let new_plugin_version_details = plugin_after
        .versions
        .get(&plugin_after.latest_version)
        .unwrap();
    assert_eq!(new_plugin_version_details.code_id, new_code_id);
    assert_eq!(new_plugin_version_details.ipfs_hash, new_ipfs_hash);
    assert_eq!(new_plugin_version_details.checksum, new_checksum);
    assert_eq!(plugin_after.latest_version, new_version);
}

#[test]
fn deployer_should_be_able_to_update_registry_fee() {
    let mut suite = HubChainSuite::init().unwrap();

    let new_registry_fee = coin(100_000, DENOM);

    suite
        .update_registry_fee(&suite.deployer.clone(), new_registry_fee.clone())
        .unwrap();

    let config = suite.query_config().unwrap();

    assert_eq!(config.registry_fee, new_registry_fee);
}

#[test]
fn deployer_should_be_able_to_update_deployer_addr() {
    let mut suite = HubChainSuite::init().unwrap();

    let new_deployer_addr = "new_deployer_addr";

    suite
        .update_deployer_addr(&suite.deployer.clone(), new_deployer_addr)
        .unwrap();

    let config = suite.query_config().unwrap();

    assert_eq!(config.deployer_addr, new_deployer_addr);
}
