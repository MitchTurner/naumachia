use crate::scripts::ValidatorCode;
use crate::{error::Result, ledger_client::LedgerClient, output::Output, TxActions};
use pallas_addresses::Address;
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

pub mod selection;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Backend<Datum, Redeemer, LC>
where
    Redeemer: Clone + Eq,
    LC: LedgerClient<Datum, Redeemer>,
{
    pub(crate) _datum: PhantomData<Datum>,
    pub(crate) _redeemer: PhantomData<Redeemer>,
    pub(crate) ledger_client: LC,
}

pub type RedemptionDetails<Datum, Redeemer> = (
    Output<Datum>,
    Redeemer,
    Box<dyn ValidatorCode<Datum, Redeemer>>,
);

impl<Datum, Redeemer, LC> Backend<Datum, Redeemer, LC>
where
    Datum: Clone + Eq + Debug,
    Redeemer: Clone + Eq + Hash,
    LC: LedgerClient<Datum, Redeemer>,
{
    pub fn new(txo_record: LC) -> Self {
        Backend {
            _datum: PhantomData,
            _redeemer: PhantomData,
            ledger_client: txo_record,
        }
    }

    pub async fn process(&self, actions: TxActions<Datum, Redeemer>) -> Result<()> {
        let tx = actions.to_unbuilt_tx()?;
        let tx_id = self.ledger_client.issue(tx).await?;
        println!("Transaction Submitted: {:?}", &tx_id);
        Ok(())
    }

    pub fn ledger_client(&self) -> &LC {
        &self.ledger_client
    }

    pub async fn signer(&self) -> Result<Address> {
        let addr = self.ledger_client.signer_base_address().await?;
        Ok(addr)
    }
}
