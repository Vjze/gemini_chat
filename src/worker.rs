use futures::{future::Fuse, FutureExt};
use slint::ComponentHandle;
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use  crate::{My_App, get_message::chat_gemini_say};
#[derive(Debug)]
pub enum WorkerMessage {
    SendMessage { action: String },
    Quit,
}

pub struct Worker {
    pub channel: UnboundedSender<WorkerMessage>,
    worker_thread: std::thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(query: &My_App) -> Self {
        let (channel, r) = tokio::sync::mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({
            let handle_weak = query.as_weak();
            move || {
                // tokio::runtime::Runtime::new()
                //     .unwrap()
                //     .block_on(query_worker_loop(r, handle_weak))
                //     .unwrap()
                let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
                rt.block_on(query_worker_loop(r, handle_weak))
                .unwrap()
                
            }
        });
        Self {
            channel,
            worker_thread,
        }
    }

    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.channel.send(WorkerMessage::Quit);
        self.worker_thread.join()
    }
}

async fn query_worker_loop(
    mut r: UnboundedReceiver<WorkerMessage>,
    handle: slint::Weak<My_App>,
) -> tokio::io::Result<()> {
    let run_send_message_future = Fuse::terminated();
    tokio::pin!(
        run_send_message_future
    );
    loop {
        let m = futures::select! {
                res = run_send_message_future => {
                    res?;
                    continue;
                }
                

            m = r.recv().fuse() => {
                match m {
                    None => return Ok(()),
                    Some(m) => m,
                }
            }
        };
        match m {
            WorkerMessage::SendMessage { action } => {
                run_send_message_future.set(chat_gemini_say(action,handle.clone()).fuse())
            }
            WorkerMessage::Quit => return Ok(()),
           
        }
    }
}