use std::sync::{Arc, Mutex};

use crate::service::llm::{TEEReq, TEEResp};
use actix_web::{middleware, web, App, HttpServer};
use operator_runer::api::request::{TEECredential, VRFProof};
use serde::{Deserialize, Serialize};
use tokio::{join, sync::mpsc::{UnboundedReceiver, UnboundedSender}};
use reqwest::Client;
use tokio::sync::mpsc::unbounded_channel;

pub mod router;
pub struct AgentStateData {
  pub prompt_sender: UnboundedSender<TEEReq>,
  pub remain_task: i32,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerReq {
    pub request_id: String,
    pub node_id: String,
    pub model: String,
    pub prompt: String,
    pub answer: String,
    pub attestation: String,
    pub attest_signature: String,
    pub elapsed: i32,
}


pub async fn start_agent(
  answer_ok_receiver: UnboundedReceiver<TEEResp>,
  prompt_sender: UnboundedSender<TEEReq>,
){
  let (remain_task_tx, remain_task_rx) = unbounded_channel::<i32>();
  let server = tokio::spawn(start_agent_server(prompt_sender, remain_task_rx));
  let client = tokio::spawn(start_agent_client(answer_ok_receiver, remain_task_tx));
  let _s = join!(server, client);
}



pub async fn start_agent_server(
  prompt_sender: UnboundedSender<TEEReq>,
  mut remain_task_rx: UnboundedReceiver<i32>,
){

  let agent_state = web::Data::new(Mutex::new(AgentStateData{
    prompt_sender: prompt_sender.clone(),
    remain_task: 1,
  }));
  let s = agent_state.clone();

  tokio::spawn(async move{
    loop {
      if let Some(_) = remain_task_rx.recv().await {
        if let Ok(mut a)  = s.get_ref().lock() {
          a.remain_task = 1;
        }
      }
    }
  });

  let app = move || {
    App::new()
    .app_data(agent_state.clone())
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
  remain_task_tx: UnboundedSender<i32>,
){
  loop {
      if let Some(res) = answer_ok_receiver.recv().await {
        tracing::info!("receive {:#?}", res);
        if let TEEResp::AnswerResp(answer) = res {

          let mut sig_hex = String::new();
          let base64_attest = base64::encode(answer.document.0.clone());
          let body = AnswerReq {
            node_id: "".into(),
            request_id: answer.request_id.clone(),
            model: answer.model_name.clone(),
            prompt: answer.prompt.clone(),
            answer: answer.answer.clone(),
            elapsed: answer.elapsed as _,
            attestation: base64_attest,
            attest_signature: sig_hex,
        };
        if let Ok(_) = remain_task_tx.send(1) {
            tracing::debug!("remain task add 1");
        };

        tracing::info!("receive {:#?}", body);

          let client = Client::new();
          let result = client
              .post(format!(
                  "{}{}",
                  "http://127.0.0.1:21001",
                  "/api/tee_callback"
              ))
              .header("Content-Type", "application/json; charset=utf-8")
              .json(&body)
              .send()
              .await;

            match result {
                Ok(res) => {
                  tracing::debug!("{}", res.status());

                },
                Err(err) => {
                  tracing::error!("{}", err);
                },
            }
            
        }
      }
  }
}

