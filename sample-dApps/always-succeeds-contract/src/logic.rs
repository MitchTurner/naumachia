use crate::logic::script::get_script;
use async_trait::async_trait;
use naumachia::{
    address::PolicyId,
    ledger_client::LedgerClient,
    logic::{SCLogic, SCLogicError, SCLogicResult},
    output::{Output, OutputId},
    scripts::ValidatorCode,
    transaction::TxActions,
    values::Values,
};
use thiserror::Error;

pub mod script;
#[cfg(test)]
mod tests;

// TODO: Pass through someplace, do not hardcode!
const NETWORK: u8 = 0;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AlwaysSucceedsLogic;

pub enum AlwaysSucceedsEndpoints {
    Lock { amount: u64 },
    Claim { output_id: OutputId },
}

pub enum AlwaysSucceedsLookups {
    ListActiveContracts { count: usize },
}

pub enum AlwaysSucceedsLookupResponses {
    ActiveContracts(Vec<Output<()>>),
}

#[derive(Debug, Error)]
pub enum AlwaysSucceedsError {
    #[error("Could not find an output with id: {0:?}")]
    OutputNotFound(OutputId),
}

#[async_trait]
impl SCLogic for AlwaysSucceedsLogic {
    type Endpoints = AlwaysSucceedsEndpoints;
    type Lookups = AlwaysSucceedsLookups;
    type LookupResponses = AlwaysSucceedsLookupResponses;
    type Datums = ();
    type Redeemers = ();

    async fn handle_endpoint<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        ledger_client: &LC,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>> {
        match endpoint {
            AlwaysSucceedsEndpoints::Lock { amount } => impl_lock(amount),
            AlwaysSucceedsEndpoints::Claim { output_id } => {
                impl_claim(ledger_client, output_id).await
            }
        }
    }

    async fn lookup<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        query: Self::Lookups,
        ledger_client: &LC,
    ) -> SCLogicResult<Self::LookupResponses> {
        match query {
            AlwaysSucceedsLookups::ListActiveContracts { count } => {
                impl_list_active_contracts(ledger_client, count).await
            }
        }
    }
}

fn impl_lock(amount: u64) -> SCLogicResult<TxActions<(), ()>> {
    let mut values = Values::default();
    values.add_one_value(&PolicyId::Lovelace, amount);
    let script = get_script().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(NETWORK)
        .map_err(SCLogicError::ValidatorScript)?;
    let tx_actions = TxActions::v2().with_script_init((), values, address);
    Ok(tx_actions)
}

async fn impl_claim<LC: LedgerClient<(), ()>>(
    ledger_client: &LC,
    output_id: OutputId,
) -> SCLogicResult<TxActions<(), ()>> {
    let script = get_script().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(NETWORK)
        .map_err(SCLogicError::ValidatorScript)?;
    let output = ledger_client
        .all_outputs_at_address(&address)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
        .into_iter()
        .find(|o| o.id() == &output_id)
        .ok_or(AlwaysSucceedsError::OutputNotFound(output_id))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let redeemer = ();
    let script_box = Box::new(script);
    let tx_actions = TxActions::v2().with_script_redeem(output, redeemer, script_box);
    Ok(tx_actions)
}

async fn impl_list_active_contracts<LC: LedgerClient<(), ()>>(
    ledger_client: &LC,
    count: usize,
) -> SCLogicResult<AlwaysSucceedsLookupResponses> {
    let script = get_script().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(NETWORK)
        .map_err(SCLogicError::ValidatorScript)?;
    let outputs = ledger_client
        .outputs_at_address(&address, count)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?;
    let subset = outputs.into_iter().take(count).collect();
    let res = AlwaysSucceedsLookupResponses::ActiveContracts(subset);
    Ok(res)
}
