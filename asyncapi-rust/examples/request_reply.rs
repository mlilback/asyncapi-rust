//! Request/reply operations
//!
//! Demonstrates `reply(...)` on `#[asyncapi_operation(...)]`: an operation that
//! sends a question on one channel and expects the answer on another.
//!
//! Run with: `cargo run --example request_reply`

use asyncapi_rust::{AsyncApi, ToAsyncApiMessage, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

/// Questions sent by the client
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum Question {
    /// Ask for the current status of a job
    #[serde(rename = "job.status")]
    #[asyncapi(summary = "Job status request")]
    JobStatus {
        /// Identifier of the job being queried
        job_id: String,
    },
}

/// Answers returned by the service
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum Answer {
    /// The job's current state
    #[serde(rename = "job.state")]
    #[asyncapi(summary = "Job status response")]
    JobState {
        /// Identifier of the job that was queried
        job_id: String,
        /// One of "queued", "running", "done"
        state: String,
    },
}

#[derive(AsyncApi)]
#[asyncapi(
    title = "Job Status API",
    version = "1.0.0",
    description = "Request/reply over two channels"
)]
#[asyncapi_server(
    name = "production",
    host = "api.example.com",
    protocol = "wss",
    description = "Production WebSocket server"
)]
#[asyncapi_channel(
    name = "questions",
    address = "/questions",
    description = "Client questions"
)]
#[asyncapi_channel(
    name = "answers",
    address = "/answers",
    description = "Service answers"
)]
#[asyncapi_operation(
    name = "askJobStatus",
    action = "send",
    channel = "questions",
    description = "Ask for a job's status and await the answer",
    messages = [Question],
    reply(
        channel = "answers",
        address(
            location = "$message.header#/replyTo",
            description = "Reply destination carried on the request"
        ),
        messages = [Answer]
    )
)]
pub struct JobStatusApi;

fn main() {
    let spec = JobStatusApi::asyncapi_spec();
    println!("{}", serde_json::to_string_pretty(&spec).unwrap());

    // The reply references the reply channel's messages, and those messages are
    // hoisted into components automatically.
    let operation = spec
        .operations
        .as_ref()
        .unwrap()
        .get("askJobStatus")
        .unwrap();
    let reply = operation.reply.as_ref().expect("reply is declared");
    assert_eq!(
        reply.channel.as_ref().unwrap().reference,
        "#/channels/answers"
    );
}
