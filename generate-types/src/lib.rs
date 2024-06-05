use program::{AuxData, UserConfig};
use schemars::schema_for;
use std::fs;

pub fn generate_types() {
    let schema_config = schema_for!(UserConfig);
    fs::write(
        "./faucet-program_serialized_config_type.txt",
        format!(
            "{:?}",
            serde_json::to_vec(&schema_config)
                .expect("error converting user config for device key proxy")
        ),
    )
    .expect("Failed to write to device key proxy config");

    let schema_aux_data = schema_for!(AuxData);
    fs::write(
        "./faucet-program_serialized_aux_data_type.txt",
        format!(
            "{:?}",
            serde_json::to_vec(&schema_aux_data)
                .expect("error converting user aux_data for device key proxy")
        ),
    )
    .expect("Failed to write to device key proxy aux_data");
}
