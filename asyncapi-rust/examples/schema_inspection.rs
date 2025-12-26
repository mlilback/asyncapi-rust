use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum TestMessage {
    Echo { text: String },
    Broadcast { room: String, text: String },
}

fn main() {
    // Get the schema for the full enum
    let root_schema = schema_for!(TestMessage);
    println!("=== Full Enum Schema ===");
    let schema_json = serde_json::to_string_pretty(&root_schema).unwrap();
    println!("{}", schema_json);

    // Parse back to see the structure
    let schema_value: serde_json::Value = serde_json::from_str(&schema_json).unwrap();

    if let Some(one_of_array) = schema_value.get("oneOf") {
        println!("\n=== Found oneOf ===");
        if let Some(variants) = one_of_array.as_array() {
            println!("Number of oneOf variants: {}", variants.len());
            for (idx, variant) in variants.iter().enumerate() {
                println!("\n--- Variant {} ---", idx);

                // Check if this variant has a const field for the type
                if let Some(props) = variant.get("properties") {
                    if let Some(type_prop) = props.get("type") {
                        if let Some(const_val) = type_prop.get("const") {
                            println!("Type const value: {}", const_val);
                        }
                    }
                }

                println!("{}", serde_json::to_string_pretty(variant).unwrap());
            }
        }
    }
}
