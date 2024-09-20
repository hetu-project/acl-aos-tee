use std::sync::Arc;

use crate::service::llm::{TEEReq, TEEResp};
use actix_web::{middleware, web, App, HttpServer};
use operator_runer::api::request::{TEECredential, VRFProof};
use serde::{Deserialize, Serialize};
use tokio::{join, sync::mpsc::{UnboundedReceiver, UnboundedSender}};
use reqwest::Client;

pub mod router;
pub struct AgentStateData {
  prompt_sender: UnboundedSender<TEEReq>,
}



#[derive(Serialize, Deserialize, Default, Debug)]
pub struct AnswerCallbackReq {
    request_id: String,
    node_id: String,
    model: String,
    prompt: String,
    answer: String,
    elapsed: u64,
    selected: bool,
    vrf_proof: VRFProof,
    tee_credential: TEECredential,
}


pub async fn start_agent(
  answer_ok_receiver: UnboundedReceiver<TEEResp>,
  prompt_sender: UnboundedSender<TEEReq>,
){
  let server = tokio::spawn(start_agent_server(prompt_sender));
  let client = tokio::spawn(start_agent_client(answer_ok_receiver));
  let _s = join!(server, client);
}



pub async fn start_agent_server(
  prompt_sender: UnboundedSender<TEEReq>,
){
  let agent_state = Arc::new(AgentStateData{
    prompt_sender,
  });

  let app = move || {App::new()
    .app_data(web::Data::new(agent_state.clone()))
    .configure(router::service)
  };

    HttpServer::new(app)
    .bind(("0.0.0.0", 3000))
    .expect("Failed to bind address")
    .run()
    .await
    .expect("Failed to run server");
}



pub async fn start_agent_client(
  mut answer_ok_receiver: UnboundedReceiver<TEEResp>,
){
  loop {
      if let Some(res) = answer_ok_receiver.recv().await {
        tracing::info!("receive {:#?}", res);
        if let TEEResp::AnswerResp(answer) = res {

          let mut sig_hex = String::new();
          let base64_attest = base64::encode(answer.document.0.clone());
          let body = AnswerCallbackReq {
            node_id: "".into(),
            request_id: answer.request_id.clone(),
            model: answer.model_name.clone(),
            prompt: answer.prompt.clone(),
            answer: answer.answer.clone(),
            elapsed: answer.elapsed,
            selected: answer.selected,
            vrf_proof: VRFProof {
                vrf_prompt_hash: answer.vrf_prompt_hash.clone(),
                vrf_random_value: answer.vrf_random_value.clone(),
                vrf_verify_pubkey: answer.vrf_verify_pubkey.clone(),
                vrf_proof: answer.vrf_proof.clone(),
            },
            tee_credential: TEECredential {
                tee_attestation: base64_attest,
                tee_attest_signature: sig_hex,
            }
        };

          let client = Client::new();
          client
              .post(format!(
                  "{}{}",
                  "",
                  "/api/tee_callback"
              ))
              .header("Content-Type", "application/json; charset=utf-8")
              .json(&body)
              .send()
              .await.ok();
            
        }
      }
  }
}

