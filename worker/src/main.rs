// use operator_runer::cli::operator::run_cli;
use tee_worker::service::llm::connect_tee_llm_worker;
use tee_worker::service;
use tee_worker::service::llm::{TEEReq, TEEResp};
use tokio::sync::mpsc::unbounded_channel;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(rust_log))
        .init();

    tracing::info!("start worker agent server");
    let (prompt_sender, prompt_receiver) = unbounded_channel::<TEEReq>();
    let (answer_ok_sender, answer_ok_receiver) = unbounded_channel::<TEEResp>();
    let tee = tokio::spawn(connect_tee_llm_worker(prompt_receiver, answer_ok_sender));
    let agent = tokio::spawn(service::agent::start_agent(answer_ok_receiver, prompt_sender));

    let res = tokio::try_join!(
        tee,
        agent,
    );

    match res {
        Ok((t, a)) => {

        },
        Err(_) => {

        },   
    }
}