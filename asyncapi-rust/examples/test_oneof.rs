use asyncapi_rust::{ToAsyncApiMessage, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum TestMessage {
    Echo { text: String },
    Broadcast { room: String, text: String },
}

fn main() {
    let messages = TestMessage::asyncapi_messages();

    println!("Number of messages: {}", messages.len());

    for (idx, msg) in messages.iter().enumerate() {
        println!("\n=== Message {} ===", idx);
        println!("Name: {:?}", msg.name);
        println!("Payload:");
        if let Some(ref payload) = msg.payload {
            let json = serde_json::to_string_pretty(payload).unwrap();
            println!("{}", json);
        }
    }
}
