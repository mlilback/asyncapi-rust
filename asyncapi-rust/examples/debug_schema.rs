use asyncapi_rust::Schema;
use chrono::NaiveDateTime;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum TestMessage {
    #[serde(rename = "test")]
    Test {
        id: i64,
        name: String,
        timestamp: Option<NaiveDateTime>,
    },
}

fn main() {
    let schema = schema_for!(TestMessage);

    println!("=== Schemars Schema as JSON ===");
    let json = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", json);

    println!("\n=== Attempting to deserialize as asyncapi_rust::Schema ===");
    let schema_value = serde_json::to_value(&schema).unwrap();

    match serde_json::from_value::<Schema>(schema_value.clone()) {
        Ok(asyncapi_schema) => {
            println!("✅ SUCCESS! Deserialized as: {:#?}", asyncapi_schema);
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
            println!("\nTrying to understand the error...");

            // Try deserializing as SchemaObject directly
            println!("\n=== Trying as SchemaObject directly ===");
            match serde_json::from_value::<asyncapi_rust::SchemaObject>(schema_value.clone()) {
                Ok(_obj) => println!("✅ SchemaObject deserialization worked!"),
                Err(e) => println!("❌ SchemaObject also failed: {}", e),
            }
        }
    }
}
