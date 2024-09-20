use std::sync::Arc;

use actix_web::{web, get, post};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tee_llm::nitro_llm::PromptReq;

use crate::service::agent::AgentStateData;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct InferParams {
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: u32,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct QuestionReq {
    pub request_id: String,
    pub node_id: String,
    pub model: String,
    pub prompt: String,
    pub params: InferParams,
    pub prompt_hash: String,
    pub signature: String,
}
/// WRITE API
// question input a prompt, and async return success, the answer callback later
#[post("/api/v1/question")]
async fn question(
    quest: web::Json<QuestionReq>,
    agent_state: web::Data<Arc<AgentStateData>>,
) -> web::Json<Value> {
    tracing::info!("Receive request, body = {:?}", quest);
    let prompt_req = PromptReq {
      request_id: quest.request_id.clone(),
      model_name: "./llama-2-7b-chat.Q4_0.gguf".to_owned(),
      prompt: quest.prompt.clone(),
      top_p: 0.95,
      temperature: 0.0,
      n_predict: 128,
      vrf_threshold: 16777215,
      vrf_precision: 6,
      vrf_prompt_hash: "sfas".to_owned(),
  };
    let prompt = tee_llm::nitro_llm::TEEReq::PromptReq(prompt_req);
    agent_state.prompt_sender.send(prompt).unwrap();
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
