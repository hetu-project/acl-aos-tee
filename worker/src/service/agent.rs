use std::sync::Arc;

use crate::service::llm::{TEEReq, TEEResp};
use actix_web::{middleware, web, App, HttpServer};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub mod router;
pub struct AgentStateData {
  answer_ok_receiver: UnboundedReceiver<TEEResp>,
  prompt_sender: UnboundedSender<TEEReq>,
}
pub async fn start_agent(
  answer_ok_receiver: UnboundedReceiver<TEEResp>,
  prompt_sender: UnboundedSender<TEEReq>,
){
  let agent_state = Arc::new(AgentStateData{
    answer_ok_receiver,
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