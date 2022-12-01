use crate::scripts::raw_script::{PlutusScriptFile, RawPlutusScriptError, RawPlutusScriptResult};
use crate::scripts::raw_validator_script::RawPlutusValidator;
use crate::scripts::{as_failed_to_execute, MintingPolicy, ScriptResult, TxContext};
use crate::PolicyId;
use cardano_multiplatform_lib::address::{EnterpriseAddress, StakeCredential};
use cardano_multiplatform_lib::plutus::{PlutusScript, PlutusV1Script};
use minicbor::Decoder;
use std::marker::PhantomData;
use uplc::ast::{FakeNamedDeBruijn, NamedDeBruijn, Program};

pub struct RawPolicy {
    script_file: PlutusScriptFile,
    cml_script: PlutusScript,
}

impl RawPolicy {
    pub fn new_v1(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let script_bytes = hex::decode(&script_file.cborHex)
            .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let v1 = PlutusV1Script::from_bytes(script_bytes)
            .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let cml_script = PlutusScript::from_v1(&v1);
        let v1_policy = RawPolicy {
            script_file,
            cml_script,
        };
        Ok(v1_policy)
    }
}

impl<Redeemer> MintingPolicy<Redeemer> for RawPolicy {
    fn execute(&self, redeemer: Redeemer, ctx: TxContext) -> ScriptResult<()> {
        let cbor = hex::decode(&self.script_file.cborHex).map_err(as_failed_to_execute)?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder.bytes().map_err(as_failed_to_execute)?;
        let mut flat_decoder = Decoder::new(outer);
        let flat = flat_decoder.bytes().map_err(as_failed_to_execute)?;
        // println!("hex: {:?}", hex::encode(&flat));
        let program: Program<NamedDeBruijn> = Program::<FakeNamedDeBruijn>::from_flat(flat)
            .unwrap()
            .try_into()
            .map_err(as_failed_to_execute)?;
        println!("whole: {}", &program);
        todo!()
    }

    fn id(&self) -> String {
        let script_hash = self.cml_script.hash();
        script_hash.to_string()
    }

    fn script_hex(&self) -> ScriptResult<&str> {
        Ok(&self.script_file.cborHex)
    }
}
