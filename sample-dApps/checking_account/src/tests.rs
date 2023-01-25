use super::*;
use crate::scripts::FakePullerValidator;
use naumachia::address::{Address, PolicyId};
use naumachia::ledger_client::test_ledger_client::TestBackendsBuilder;
use naumachia::smart_contract::{SmartContract, SmartContractTrait};

const NETWORK: u8 = 0;

#[tokio::test]
async fn init_creates_instance_with_correct_balance() {
    let me = Address::new("addr_test1qpuy2q9xel76qxdw8r29skldzc876cdgg9cugfg7mwh0zvpg3292mxuf3kq7nysjumlxjrlsfn9tp85r0l54l29x3qcs7nvyfm");
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::ADA, start_amount)
        .finish_output()
        .build_in_memory();

    let account_amount = 10_000_000;
    let endpoint = CheckingAccountEndpoints::InitAccount {
        starting_lovelace: account_amount,
    };
    let contract = SmartContract::new(&CheckingAccountLogic, &backend);
    contract.hit_endpoint(endpoint).await.unwrap();

    let script = checking_account_validator().unwrap();
    let address = script.address(NETWORK).unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let value = script_output.values().get(&PolicyId::ADA).unwrap();
    assert_eq!(value, account_amount);
}

#[tokio::test]
async fn add_puller_creates_new_datum_for_puller() {
    let me = Address::new("addr_test1qpuy2q9xel76qxdw8r29skldzc876cdgg9cugfg7mwh0zvpg3292mxuf3kq7nysjumlxjrlsfn9tp85r0l54l29x3qcs7nvyfm");
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::ADA, start_amount)
        .finish_output()
        .build_in_memory();

    let puller = Address::new("addr_test1qrmezjhpelwzvz83wjl0e6mx766de7j3nksu2338s00yzx870xyxfa97xyz2zn5rknyntu5g0c66s7ktjnx0p6f0an6s3dyxwr");
    let endpoint = CheckingAccountEndpoints::AddPuller {
        puller: puller.clone(),
        amount_lovelace: 15_000_000,
        period: 1000,
        next_pull: 0,
    };
    let contract = SmartContract::new(&CheckingAccountLogic, &backend);
    contract.hit_endpoint(endpoint).await.unwrap();
    let script = FakePullerValidator;
    let address = script.address(NETWORK).unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let value = script_output.values().get(&PolicyId::ADA).unwrap();
    assert_eq!(value, 0);
}

#[tokio::test]
async fn remove_puller__removes_the_allowed_puller() {
    let me = Address::new("addr_test1qpuy2q9xel76qxdw8r29skldzc876cdgg9cugfg7mwh0zvpg3292mxuf3kq7nysjumlxjrlsfn9tp85r0l54l29x3qcs7nvyfm");
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::ADA, start_amount)
        .finish_output()
        .build_in_memory();

    let puller = Address::new("addr_test1qrmezjhpelwzvz83wjl0e6mx766de7j3nksu2338s00yzx870xyxfa97xyz2zn5rknyntu5g0c66s7ktjnx0p6f0an6s3dyxwr");
    let add_endpoint = CheckingAccountEndpoints::AddPuller {
        puller: puller.clone(),
        amount_lovelace: 15_000_000,
        period: 1000,
        next_pull: 0,
    };

    let contract = SmartContract::new(&CheckingAccountLogic, &backend);
    contract.hit_endpoint(add_endpoint).await.unwrap();
    let script = FakePullerValidator;
    let address = script.address(NETWORK).unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let output_id = script_output.id().to_owned();

    let remove_endpoint = CheckingAccountEndpoints::RemovePuller { output_id };

    contract.hit_endpoint(remove_endpoint).await.unwrap();

    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    assert!(outputs_at_address.pop().is_none());
}

#[tokio::test]
async fn fund_account__replaces_existing_balance_with_updated_amount() {
    let me = Address::new("addr_test1qpuy2q9xel76qxdw8r29skldzc876cdgg9cugfg7mwh0zvpg3292mxuf3kq7nysjumlxjrlsfn9tp85r0l54l29x3qcs7nvyfm");
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::ADA, start_amount)
        .finish_output()
        .build_in_memory();

    let account_amount = 10_000_000;
    let fund_amount = 5_000_000;
    let init_endpoint = CheckingAccountEndpoints::InitAccount {
        starting_lovelace: account_amount,
    };

    let contract = SmartContract::new(&CheckingAccountLogic, &backend);
    contract.hit_endpoint(init_endpoint).await.unwrap();

    let script = checking_account_validator().unwrap();
    let address = script.address(NETWORK).unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let output_id = script_output.id().to_owned();

    let fund_endpoint = CheckingAccountEndpoints::FundAccount {
        output_id,
        fund_amount,
    };

    contract.hit_endpoint(fund_endpoint).await.unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let value = script_output.values().get(&PolicyId::ADA).unwrap();
    assert_eq!(value, account_amount + fund_amount);
}

#[tokio::test]
async fn withdraw_from_account__replaces_existing_balance_with_updated_amount() {
    let me = Address::new("addr_test1qpuy2q9xel76qxdw8r29skldzc876cdgg9cugfg7mwh0zvpg3292mxuf3kq7nysjumlxjrlsfn9tp85r0l54l29x3qcs7nvyfm");
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::ADA, start_amount)
        .finish_output()
        .build_in_memory();

    let account_amount = 10_000_000;
    let withdraw_amount = 5_000_000;
    let init_endpoint = CheckingAccountEndpoints::InitAccount {
        starting_lovelace: account_amount,
    };
    let contract = SmartContract::new(&CheckingAccountLogic, &backend);
    contract.hit_endpoint(init_endpoint).await.unwrap();

    let script = checking_account_validator().unwrap();
    let address = script.address(NETWORK).unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let output_id = script_output.id().to_owned();

    let fund_endpoint = CheckingAccountEndpoints::WithdrawFromAccount {
        output_id,
        withdraw_amount,
    };

    contract.hit_endpoint(fund_endpoint).await.unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let value = script_output.values().get(&PolicyId::ADA).unwrap();
    assert_eq!(value, account_amount - withdraw_amount);
}

#[tokio::test]
async fn pull_from_account__replaces_existing_balances_with_updated_amounts() {
    let owner = Address::new("addr_test1qpuy2q9xel76qxdw8r29skldzc876cdgg9cugfg7mwh0zvpg3292mxuf3kq7nysjumlxjrlsfn9tp85r0l54l29x3qcs7nvyfm");
    let puller = Address::new("addr_test1qrmezjhpelwzvz83wjl0e6mx766de7j3nksu2338s00yzx870xyxfa97xyz2zn5rknyntu5g0c66s7ktjnx0p6f0an6s3dyxwr");

    let allow_puller_script = FakePullerValidator;
    let allow_puller_address = allow_puller_script.address(NETWORK).unwrap();
    let account_script = checking_account_validator().unwrap();
    let spending_token_policy = vec![5, 5, 5, 5, 5];
    let account_address = account_script.address(NETWORK).unwrap();

    let account_amount = 100_000_000;
    let pull_amount = 15_000_000;
    let account_datum = CheckingAccountDatums::CheckingAccount {
        owner,
        spend_token_policy: "".to_string(),
    };
    let allow_puller_datum = CheckingAccountDatums::AllowedPuller {
        puller: puller.clone(),
        amount_lovelace: pull_amount,
        period: 1000,
        next_pull: 0,
    };
    let backend = TestBackendsBuilder::new(&puller)
        .start_output(&account_address)
        .with_datum(account_datum)
        .with_value(PolicyId::ADA, account_amount)
        .finish_output()
        .start_output(&allow_puller_address)
        .with_datum(allow_puller_datum)
        .with_value(
            PolicyId::NativeToken(hex::encode(spending_token_policy), None),
            0,
        )
        .finish_output()
        .build_in_memory();

    let contract = SmartContract::new(&CheckingAccountLogic, &backend);

    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&account_address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let checking_account_output_id = script_output.id().to_owned();

    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&allow_puller_address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let allow_pull_output_id = script_output.id().to_owned();

    // When
    let pull_endpoint = CheckingAccountEndpoints::PullFromCheckingAccount {
        allow_pull_output_id,
        checking_account_output_id,
        amount: pull_amount,
    };
    contract.hit_endpoint(pull_endpoint).await.unwrap();

    // Then
    let mut outputs_at_account_address = backend
        .ledger_client
        .all_outputs_at_address(&account_address)
        .await
        .unwrap();
    let script_output = outputs_at_account_address.pop().unwrap();
    let value = script_output.values().get(&PolicyId::ADA).unwrap();
    assert_eq!(value, account_amount - pull_amount);

    let mut outputs_at_puller_address = backend
        .ledger_client
        .all_outputs_at_address(&puller)
        .await
        .unwrap();
    let script_output = outputs_at_puller_address.pop().unwrap();
    let value = script_output.values().get(&PolicyId::ADA).unwrap();
    assert_eq!(value, pull_amount);
}
