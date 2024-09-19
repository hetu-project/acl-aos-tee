use std::sync::Arc;

use actix_web::{web, get, post};
use serde_json::{json, Value};

use crate::service::agent::AgentStateData;
/// WRITE API
// question input a prompt, and async return success, the answer callback later
#[post("/api/v1/question")]
async fn question(
    quest: web::Json<Value>,
    agent_state: web::Data<Arc<AgentStateData>>,
) -> web::Json<Value> {
    tracing::info!("Receive request, body = {:?}", quest);
    agent_state.prompt_sender.send(crate::service::llm::TEEReq {  }).unwrap();
    let json_data = json!({
      "answer": ""
    });
    return web::Json(json_data);
}


pub fn service(cfg: &mut web::ServiceConfig) {
  cfg
  .service(question)
  ;
}
