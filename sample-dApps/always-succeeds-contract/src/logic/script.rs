use naumachia::scripts::{
    plutus_validator::PlutusValidator,
    raw_script::BlueprintFile,
    ScriptError,
    ScriptResult,
};

const BLUEPRINT: &str = include_str!("../../always_succeeds/plutus.json");
const VALIDATOR_NAME: &str = "always_true.spend";

pub fn get_script() -> ScriptResult<PlutusValidator<(), ()>> {
    let script_file: BlueprintFile = serde_json::from_str(BLUEPRINT)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let validator_blueprint = script_file.get_validator(VALIDATOR_NAME).ok_or(
        ScriptError::FailedToConstruct(format!(
            "Validator not listed in Blueprint: {:?}",
            VALIDATOR_NAME
        )),
    )?;
    let raw_script_validator = PlutusValidator::from_blueprint(validator_blueprint)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use naumachia::{
        scripts::{
            context::{
                pub_key_hash_from_address_if_available,
                ContextBuilder,
            },
            Validator,
        },
        Address,
    };

    #[test]
    fn test() {
        let script = get_script().unwrap();

        let owner = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();

        let owner_pkh = pub_key_hash_from_address_if_available(&owner).unwrap();
        let ctx = ContextBuilder::new(owner_pkh).build_spend(&vec![], 0);
        script.execute((), (), ctx).unwrap();
    }
}
