use std::ops::RangeBounds;

use futures::{future::Fuse, FutureExt};
use slint::ComponentHandle;
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use  crate::{My_App, get_message::chat_gemini_say, updata::{get_last_release, download_file, updata}};
#[derive(Debug)]
pub enum WorkerMessage {
    SendMessage { action: String },
    DownMessage { download_path: String},
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
    let run_get_updata_future = get_last_release(handle.clone()).fuse();
    let run_send_message_future = Fuse::terminated();
    let run_download_message_future = Fuse::terminated();
    tokio::pin!(
        run_send_message_future,run_get_updata_future,run_download_message_future
    );
    loop {
        let m = futures::select! {
                res = run_send_message_future => {
                    res?;
                    continue;
                }
                res = run_get_updata_future => {
                    res?;
                    continue;
                }
                res = run_download_message_future => {
                    // res?;
                    let _ = if let Ok((relaunch_path, cleanup_path)) = res {
                        updata(relaunch_path,cleanup_path)
                    }else {
                        continue;
                    };
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
            WorkerMessage::DownMessage { download_path } => {
                run_download_message_future.set(download_file(download_path,handle.clone()).fuse())
            },
           
        }
    }
}