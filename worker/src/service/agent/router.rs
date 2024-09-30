use std::sync::{Arc, Mutex};

use actix_web::{get, post, web, HttpResponse, Responder};
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
    agent_state: web::Data<Mutex<AgentStateData>>,
) -> impl Responder {
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
     let mut ag = agent_state.lock().unwrap();
     tracing::debug!("remain_task start {}", ag.remain_task);
     if ag.remain_task <= 0 {
      return HttpResponse::ServiceUnavailable().json(json!({}));
     }
     ag.remain_task  = ag.remain_task - 1;
    if let Err(err) = ag.prompt_sender.send(prompt){
      tracing::error!("send prompt req to tee error {:#?}", err);
      ag.remain_task  = ag.remain_task + 1;
      return HttpResponse::ServiceUnavailable().json(json!(""));
    }
    tracing::debug!("remain_task end {}", ag.remain_task);
    let json_data = json!({
      "answer": ""
    });
    return HttpResponse::Ok().json(json_data);
}


// question input a prompt, and async return success, the answer callback later
#[get("/api/v1/status")]
async fn status(
    // quest: web::Json<QuestionReq>,
    agent_state: web::Data<Mutex<AgentStateData>>,
) -> impl Responder {
  let ag = agent_state.lock().unwrap();
    let json_data = json!({
      "remain_task": ag.remain_task,
    });
    return HttpResponse::Ok().json(json_data);
}



pub fn service(cfg: &mut web::ServiceConfig) {
  cfg
  .service(question)
  .service(status)
  ;
}
