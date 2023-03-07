use super::*;
use naumachia::ledger_client::test_ledger_client::TestBackendsBuilder;
use naumachia::smart_contract::{SmartContract, SmartContractTrait};
use naumachia::Address;

#[tokio::test]
async fn lock_and_claim() {
    let me = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::Lovelace, start_amount)
        .finish_output()
        .build_in_memory();

    let amount = 10_000_000;
    let endpoint = AlwaysSucceedsEndpoints::Lock { amount };
    let script = get_script().unwrap();
    let contract = SmartContract::new(&AlwaysSucceedsLogic, &backend);
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
    let call = AlwaysSucceedsEndpoints::Claim {
        output_id: instance.id().clone(),
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
