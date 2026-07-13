use asyncapi_rust::{AsyncApi, ToAsyncApiMessage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ExampleResponseTopic {
    #[schemars(regex(pattern = "response/client/([a-z1-9]+)"))]
    pub response: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct A {
    pub correlation_data: u8,
}

#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[asyncapi(
    mqtt(
        payload_format_indicator = 1,
        content_type = "application/json",
        response_topic = ExampleResponseTopic,
        binding_version = "1.0",
        correlation_data = A
    )
)]
pub struct MqttMessage;

#[derive(AsyncApi)]
#[asyncapi(
    title = "Mqtt Api",
    version = "1.0.0",
    description = "Mqtt bindings example"
)]
#[asyncapi_server(
    name = "production",
    host = "api.example.com",
    protocol = "mqtt",
    description = "Production mqtt broker",
    mqtt(
        client_id = "client1",
        clean_session = false,
        last_will(
            topic = "devices/",
            qos = 0,
            message = "LastWillMessage",
            retain = false
        ),
        keep_alive = 60,
        session_expiry_interval = 3600,
        binding_version = "1.0"
    )
)]
#[asyncapi_channel(name = "test", address = "/test")]
#[asyncapi_operation(
    name = "sendMqttMessage",
    action = "send",
    channel = "test",
    mqtt(
        qos = 2,
        retain = true,
        message_expiry_interval = 300,
        binding_version = "1.0"
    )
)]
#[asyncapi_messages(MqttMessage)]
pub struct MqttApi;

fn main() {
    let spec = MqttApi::asyncapi_spec();

    // Display Servers
    if let Some(servers) = &spec.servers {
        println!("🖥️  Servers ({}):", servers.len());
        for (name, server) in servers {
            println!("  • {}", name);
            println!("    Host: {}", server.host);
            println!("    Protocol: {}", server.protocol);
            if let Some(desc) = &server.description {
                println!("    Description: {}", desc);
            }
            println!("      bindings: {:?}", &server.bindings)
        }
        println!();
    }

    // Display messages (automatically populated from ChatMessage and SystemMessage)
    if let Some(components) = &spec.components {
        if let Some(messages) = &components.messages {
            println!("Messages (automatically included from message types):");
            for (name, message) in messages {
                println!("  - {}", name);
                if let Some(mqtt) = &message.bindings.as_ref().unwrap().mqtt {
                    println!("    Mqtt Bindings: {:?}", mqtt);
                }
            }
            println!();
        }
    }

    // Display Operations
    if let Some(operations) = &spec.operations {
        println!("⚡ Operations ({}):", operations.len());
        for (name, operation) in operations {
            let action = match operation.action {
                asyncapi_rust::OperationAction::Send => "send",
                asyncapi_rust::OperationAction::Receive => "receive",
            };
            println!("  • {} ({})", name, action);
            println!("    Channel: {}", operation.channel.reference);
            println!("  Mqtt: {:?}", &operation.bindings.as_ref().unwrap().mqtt);
        }
        println!();
    }

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&spec).expect("Failed to serialize spec");

    println!("📄 Complete JSON Specification:\n");
    println!("{}", json);
}
