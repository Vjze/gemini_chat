#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod worker;
mod get_message;
mod updata;
slint::include_modules!();
fn main(){
    let app = My_App::new().unwrap();
    let worker = worker::Worker::new(&app);
    app.on_send_message({
        let worker_channel = worker.channel.clone();
        move|action|{
            worker_channel.send(worker::WorkerMessage::SendMessage { action: action.to_string() }).unwrap()
        }
    });
    app.on_updata_btn({
        let worker_channel = worker.channel.clone();
        move|download_path|{
            worker_channel.send(worker::WorkerMessage::DownMessage { download_path: download_path.to_string() }).unwrap()
        }
    });
    app.run().unwrap();
    worker.join().unwrap();
}
