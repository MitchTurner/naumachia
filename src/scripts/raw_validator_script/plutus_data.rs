use crate::scripts::context::{
    CtxDatum, CtxOutput, CtxOutputReference, CtxScriptPurpose, CtxValue, Input, PubKeyHash,
    TxContext, ValidRange,
};
use crate::scripts::ScriptError;
use cardano_multiplatform_lib::ledger::common::hash::hash_plutus_data;
use pallas_addresses::{Address, ShelleyDelegationPart, ShelleyPaymentPart};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum PlutusData {
    Constr(Constr<PlutusData>),
    Map(BTreeMap<PlutusData, PlutusData>),
    BigInt(BigInt),
    BoundedBytes(Vec<u8>),
    Array(Vec<PlutusData>),
}

impl PlutusData {
    pub fn hash(&self) -> Vec<u8> {
        // TODO: move this maybe
        use crate::trireme_ledger_client::cml_client::plutus_data_interop::PlutusDataInterop;
        let cml_data = self.to_plutus_data();
        hash_plutus_data(&cml_data).to_bytes().to_vec()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Constr<T> {
    pub constr: u64,
    pub fields: Vec<T>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum BigInt {
    Int { neg: bool, val: u64 },
    BigUInt(Vec<u8>),
    BigNInt(Vec<u8>),
}

impl From<i64> for BigInt {
    fn from(num: i64) -> Self {
        let neg = num.is_negative();
        let val = num.unsigned_abs();
        BigInt::Int { neg, val }
    }
}

impl From<BigInt> for i64 {
    fn from(big_int: BigInt) -> Self {
        match big_int {
            BigInt::Int { neg, val } => {
                let value = val as i64;
                if neg {
                    -value
                } else {
                    value
                }
            }
            BigInt::BigUInt(_) => todo!(),
            BigInt::BigNInt(_) => todo!(),
        }
    }
}

impl From<i64> for PlutusData {
    fn from(num: i64) -> Self {
        let neg = num.is_negative();
        let val = num.unsigned_abs();
        PlutusData::BigInt(BigInt::Int { neg, val })
    }
}

impl TryFrom<PlutusData> for i64 {
    type Error = ScriptError;

    fn try_from(data: PlutusData) -> Result<Self, Self::Error> {
        match data {
            PlutusData::BigInt(inner) => Ok(inner.into()),
            _ => Err(ScriptError::DatumDeserialization(format!("{data:?}"))),
        }
    }
}

// TODO: Don't hardcode values!
// TODO: THIS IS V2 only right now! Add V1!
impl From<TxContext> for PlutusData {
    fn from(ctx: TxContext) -> Self {
        let inputs = PlutusData::Array(ctx.inputs.into_iter().map(Into::into).collect());
        let reference_inputs = PlutusData::Array(vec![]);
        let outputs = PlutusData::Array(ctx.outputs.into_iter().map(Into::into).collect());
        let fee = PlutusData::Map(BTreeMap::from([(
            PlutusData::BoundedBytes(Vec::new()),
            PlutusData::Map(BTreeMap::from([(
                PlutusData::BoundedBytes(Vec::new()),
                PlutusData::BigInt(999_i64.into()),
            )])),
        )]));
        let mint = PlutusData::Map(BTreeMap::from([(
            PlutusData::BoundedBytes(Vec::new()),
            PlutusData::Map(BTreeMap::from([(
                PlutusData::BoundedBytes(Vec::new()),
                PlutusData::BigInt(0_i64.into()),
            )])),
        )]));
        let dcert = PlutusData::Array(vec![]);
        let wdrl = PlutusData::Map(BTreeMap::new());
        let valid_range = ctx.range.into();
        let mut signers: Vec<_> = ctx.extra_signatories.into_iter().map(Into::into).collect();
        signers.push(ctx.signer.into());
        let signatories = PlutusData::Array(signers);
        let redeemers = PlutusData::Map(BTreeMap::new());
        let data = PlutusData::Map(
            ctx.datums
                .into_iter()
                .map(|(hash, data)| (PlutusData::BoundedBytes(hash), data))
                .collect(),
        );
        // TODO this id should be computed!
        let id = wrap_with_constr(0, PlutusData::BoundedBytes(Vec::new()));
        let tx_info = PlutusData::Constr(Constr {
            constr: 0,
            fields: vec![
                inputs,
                reference_inputs,
                outputs,
                fee,
                mint,
                dcert,
                wdrl,
                valid_range,
                signatories,
                redeemers,
                data,
                id,
            ],
        });
        // Spending
        let purpose = match ctx.purpose {
            CtxScriptPurpose::Mint(policy_id) => {
                let policy_id_data = PlutusData::BoundedBytes(policy_id);
                wrap_with_constr(0, policy_id_data)
            }
            CtxScriptPurpose::Spend(out_ref) => {
                let out_ref_data = out_ref.into();
                wrap_with_constr(1, out_ref_data)
            }
            _ => {
                todo!()
            }
        };

        PlutusData::Constr(Constr {
            constr: 0,
            fields: vec![tx_info, purpose],
        })
    }
}

impl From<PubKeyHash> for PlutusData {
    fn from(value: PubKeyHash) -> Self {
        PlutusData::BoundedBytes(value.bytes())
    }
}

impl From<Address> for PlutusData {
    fn from(value: Address) -> Self {
        match value {
            Address::Shelley(shelley_address) => {
                let payment_part = shelley_address.payment();
                let stake_part = shelley_address.delegation();

                let payment_part_plutus_data = match payment_part {
                    ShelleyPaymentPart::Key(payment_keyhash) => {
                        let inner = PlutusData::BoundedBytes(payment_keyhash.to_vec());
                        wrap_with_constr(0, inner)
                    }
                    ShelleyPaymentPart::Script(script_hash) => {
                        let inner = PlutusData::BoundedBytes(script_hash.to_vec());
                        wrap_with_constr(1, inner)
                    }
                };

                let stake_part_plutus_data = match stake_part {
                    ShelleyDelegationPart::Key(stake_keyhash) => {
                        let bytes_data = PlutusData::BoundedBytes(stake_keyhash.to_vec());
                        let inner = wrap_with_constr(0, bytes_data);
                        wrap_with_constr(0, inner)
                    }
                    ShelleyDelegationPart::Script(script_keyhash) => {
                        let bytes_data = PlutusData::BoundedBytes(script_keyhash.to_vec());
                        let inner = wrap_with_constr(1, bytes_data);
                        wrap_with_constr(0, inner)
                    }
                    ShelleyDelegationPart::Pointer(pointer) => {
                        let inner = wrap_multiple_with_constr(
                            1,
                            vec![
                                pointer.slot().into(),
                                pointer.tx_idx().into(),
                                pointer.cert_idx().into(),
                            ],
                        );
                        wrap_with_constr(0, inner)
                    }
                    ShelleyDelegationPart::Null => empty_constr(1),
                };

                wrap_multiple_with_constr(0, vec![payment_part_plutus_data, stake_part_plutus_data])
            }
            _ => todo!(),
        }
    }
}

fn wrap_with_constr(index: u64, data: PlutusData) -> PlutusData {
    PlutusData::Constr(Constr {
        constr: constr_index(index),
        fields: vec![data],
    })
}

fn wrap_multiple_with_constr(index: u64, data: Vec<PlutusData>) -> PlutusData {
    PlutusData::Constr(Constr {
        constr: constr_index(index),
        fields: data,
    })
}

fn empty_constr(index: u64) -> PlutusData {
    PlutusData::Constr(Constr {
        constr: constr_index(index),
        fields: vec![],
    })
}

/// Translate constructor index to cbor tag.
fn constr_index(index: u64) -> u64 {
    index
}

impl From<ValidRange> for PlutusData {
    fn from(value: ValidRange) -> Self {
        match (value.lower, value.upper) {
            (None, None) => no_time_bound(),
            (Some((bound, is_inclusive)), None) => lower_bound(bound, is_inclusive),
            (None, Some(_)) => todo!(),
            (Some(_), Some(_)) => todo!(),
        }
    }
}

fn no_time_bound() -> PlutusData {
    PlutusData::Constr(Constr {
        constr: 0,
        fields: vec![
            PlutusData::Constr(Constr {
                constr: 0,
                fields: vec![
                    // NegInf
                    PlutusData::Constr(Constr {
                        constr: 0,
                        fields: vec![],
                    }),
                    // Closure
                    PlutusData::Constr(Constr {
                        constr: 1,
                        fields: vec![],
                    }),
                ],
            }),
            PlutusData::Constr(Constr {
                constr: 0,
                fields: vec![
                    // PosInf
                    PlutusData::Constr(Constr {
                        constr: 2,
                        fields: vec![],
                    }),
                    // Closure
                    PlutusData::Constr(Constr {
                        constr: 1,
                        fields: vec![],
                    }),
                ],
            }),
        ],
    })
}

fn lower_bound(bound: i64, is_inclusive: bool) -> PlutusData {
    let closure = if is_inclusive {
        // True
        PlutusData::Constr(Constr {
            constr: 1,
            fields: vec![],
        })
    } else {
        // False
        PlutusData::Constr(Constr {
            constr: 0,
            fields: vec![],
        })
    };
    PlutusData::Constr(Constr {
        constr: 0,
        fields: vec![
            PlutusData::Constr(Constr {
                constr: 0,
                fields: vec![
                    // Finite
                    PlutusData::Constr(Constr {
                        constr: 1,
                        fields: vec![PlutusData::BigInt(bound.into())],
                    }),
                    // Closure
                    closure,
                ],
            }),
            PlutusData::Constr(Constr {
                constr: 0,
                fields: vec![
                    // PosInf
                    PlutusData::Constr(Constr {
                        constr: 2,
                        fields: vec![],
                    }),
                    // Closure
                    PlutusData::Constr(Constr {
                        constr: 1,
                        fields: vec![],
                    }),
                ],
            }),
        ],
    })
}

impl From<Input> for PlutusData {
    fn from(input: Input) -> Self {
        let output_reference = CtxOutputReference {
            transaction_id: input.transaction_id,
            output_index: input.output_index,
        }
        .into();
        let output = CtxOutput {
            address: input.address,
            value: input.value,
            datum: input.datum,
            reference_script: input.reference_script,
        }
        .into();
        PlutusData::Constr(Constr {
            constr: 0,
            fields: vec![output_reference, output],
        })
    }
}

impl From<CtxOutputReference> for PlutusData {
    fn from(out_ref: CtxOutputReference) -> Self {
        let tx_id_bytes = out_ref.transaction_id;
        let transaction_id = wrap_with_constr(0, PlutusData::BoundedBytes(tx_id_bytes));
        let output_index = PlutusData::BigInt((out_ref.output_index as i64).into()); // TODO: panic
        PlutusData::Constr(Constr {
            constr: 0,
            fields: vec![transaction_id, output_index],
        })
    }
}

impl From<CtxOutput> for PlutusData {
    fn from(output: CtxOutput) -> Self {
        let address = output.address.into();
        let value = output.value.into();
        let datum = output.datum.into();
        let reference_script = output.reference_script.into();
        PlutusData::Constr(Constr {
            constr: 0,
            fields: vec![address, value, datum, reference_script],
        })
    }
}

impl From<CtxValue> for PlutusData {
    fn from(value: CtxValue) -> Self {
        let converted_inner = value
            .inner
            .iter()
            .map(|(p, a)| {
                let policy_id = PlutusData::BoundedBytes(hex::decode(p).unwrap()); // TODO
                let assets = a
                    .iter()
                    .map(|(an, amt)| {
                        let asset_name = PlutusData::BoundedBytes(an.as_bytes().to_vec()); // TODO: Should this be bytes? or hex decoded?
                        let amount = PlutusData::BigInt((*amt as i64).into()); // TODO
                        (asset_name, amount)
                    })
                    .collect();
                (policy_id, PlutusData::Map(assets))
            })
            .collect();
        PlutusData::Map(converted_inner)
    }
}

impl From<CtxDatum> for PlutusData {
    fn from(value: CtxDatum) -> Self {
        match value {
            CtxDatum::NoDatum => PlutusData::Constr(Constr {
                constr: 0,
                fields: vec![],
            }),
            CtxDatum::DatumHash(hash) => PlutusData::Constr(Constr {
                constr: 1,
                fields: vec![PlutusData::BoundedBytes(hash)],
            }),
            CtxDatum::InlineDatum(data) => PlutusData::Constr(Constr {
                constr: 2,
                fields: vec![data],
            }),
        }
    }
}

// ref
// https://github.com/aiken-lang/aiken/blob/9f587e802c74531471cfbb3fd0d3baea1a8a62b3/crates/uplc/src/tx/to_plutus_data.rs#L157
impl<T: Into<PlutusData>> From<Option<T>> for PlutusData {
    fn from(value: Option<T>) -> Self {
        match value {
            None => PlutusData::Constr(Constr {
                constr: 1,
                fields: vec![],
            }),
            Some(inner) => PlutusData::Constr(Constr {
                constr: 0,
                fields: vec![inner.into()],
            }),
        }
    }
}

impl From<Vec<u8>> for PlutusData {
    fn from(value: Vec<u8>) -> Self {
        PlutusData::BoundedBytes(value)
    }
}

impl From<()> for PlutusData {
    fn from(_: ()) -> Self {
        PlutusData::Constr(Constr {
            constr: 0,
            fields: Vec::new(),
        })
    }
}

impl From<PlutusData> for () {
    fn from(_: PlutusData) -> Self {}
}

impl From<u64> for PlutusData {
    fn from(value: u64) -> Self {
        PlutusData::BigInt((value as i64).into()) // TODO: unwrap
    }
}
