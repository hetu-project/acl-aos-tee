use bincode::Options;
use  tee_llm::nitro_llm;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use serde::{Deserialize, Serialize};

pub use nitro_llm::{TEEReq, TEEResp};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct TEEReq {

// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct TEEResp {
  
// }

pub fn try_connection(cid: u32, port: u32) -> anyhow::Result<tokio::net::UnixStream> {
  use nix::sys::socket::{connect, socket, AddressFamily, SockFlag, SockType, VsockAddr};
  use std::os::fd::AsRawFd;

  let fd = socket(
      AddressFamily::Vsock,
      SockType::Stream,
      SockFlag::empty(),
      None,
  )?;

  {
      let _span = tracing::debug_span!("connect").entered();
      connect(fd.as_raw_fd(), &VsockAddr::new(cid, port))?
  }

  let stream = std::os::unix::net::UnixStream::from(fd);
  stream.set_nonblocking(true)?;

  let stream = tokio::net::UnixStream::from_std(stream)?;
  Ok(stream)
}


pub async fn tee_start_listening(
  stream: tokio::net::UnixStream,
  mut events: UnboundedReceiver<TEEReq>,
  sender: UnboundedSender<TEEResp>,
) -> anyhow::Result<()> {
  use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

  let (mut read_half, mut write_half) = stream.into_split();

  let write_session = tokio::spawn(async move {
      while let Some(prompt) = events.recv().await {
          let buf = bincode::options().serialize(&prompt)?;
          write_half.write_u64_le(buf.len() as _).await?;
          write_half.write_all(&buf).await?;
      }
      anyhow::Ok(())
  });

  let read_session = tokio::spawn(async move {
      loop {
          let len = read_half.read_u64_le().await?;
          let mut buf = vec![0; len as _];
          read_half.read_exact(&mut buf).await?;
          sender.send(bincode::options().deserialize(&buf)?)?
      }
      #[allow(unreachable_code)] // for type hinting
      anyhow::Ok(())
  });

  tokio::select! {
      result = write_session => return result?,
      result = read_session => result??
  }

  anyhow::bail!("unreachable")
}




pub async fn connect_tee_llm_worker(
  prompt_receiver: UnboundedReceiver<TEEReq>,
  answer_ok_sender: UnboundedSender<TEEResp>,
) -> anyhow::Result<()>  {
  let cid = 15;
  let port = 5005;
  let stream = try_connection(cid, port)?;
  // let (answer_ok_sender, answer_ok_receiver) = unbounded_channel::<TEEResp>();

  tee_start_listening(stream, prompt_receiver, answer_ok_sender).await?;
  Ok(())
}