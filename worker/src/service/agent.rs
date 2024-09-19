use std::sync::Arc;

use crate::service::llm::{TEEReq, TEEResp};
use actix_web::{middleware, web, App, HttpServer};
use tokio::{join, sync::mpsc::{UnboundedReceiver, UnboundedSender}};

pub mod router;
pub struct AgentStateData {
  prompt_sender: UnboundedSender<TEEReq>,
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

      }
  }
}

