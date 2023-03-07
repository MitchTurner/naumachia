use super::*;
use naumachia::ledger_client::test_ledger_client::TestBackendsBuilder;
use naumachia::smart_contract::{SmartContract, SmartContractTrait};
use naumachia::Address;

// Ignore because the game script is funky with Aiken
#[ignore]
#[tokio::test]
async fn lock_and_claim() {
    let me = Address::from_bech32("addr_test1qpuy2q9xel76qxdw8r29skldzc876cdgg9cugfg7mwh0zvpg3292mxuf3kq7nysjumlxjrlsfn9tp85r0l54l29x3qcs7nvyfm").unwrap();
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::Lovelace, start_amount)
        .finish_output()
        .build_in_memory();

    let amount = 10_000_000;
    let secret = "my secret";
    let endpoint = GameEndpoints::Lock {
        amount,
        secret: secret.to_string(),
    };
    let script = get_script().unwrap();
    let contract = SmartContract::new(&GameLogic, &backend);
    contract.hit_endpoint(endpoint).await.unwrap();
    {
        let expected = amount;
        let actual = backend
            .ledger_client
            .balance_at_address(&script.address(0).unwrap(), &PolicyId::Lovelace)
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }

    {
        let expected = start_amount - amount;
        let actual = backend
            .ledger_client
            .balance_at_address(&me, &PolicyId::Lovelace)
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }
    let instance = backend
        .ledger_client
        .all_outputs_at_address(&script.address(0).unwrap())
        .await
        .unwrap()
        .pop()
        .unwrap();
    let call = GameEndpoints::Guess {
        output_id: instance.id().clone(),
        guess: secret.to_string(),
    };

    contract.hit_endpoint(call).await.unwrap();
    {
        let actual = backend
            .ledger_client
            .balance_at_address(&me, &PolicyId::Lovelace)
            .await
            .unwrap();
        assert_eq!(actual, start_amount);
    }
    {
        let script_balance = backend
            .ledger_client
            .balance_at_address(&script.address(0).unwrap(), &PolicyId::Lovelace)
            .await
            .unwrap();
        assert_eq!(script_balance, 0);
    }
}
