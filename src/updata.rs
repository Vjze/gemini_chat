use std::{
    fs::{self},
    io::{copy, self},
    path::{PathBuf, Path},
};
use retry::retry;
use crate::My_App;
use retry::{delay::Fibonacci, OperationResult};
use slint::Weak;

pub async fn get_last_release(handle: Weak<My_App>) -> tokio::io::Result<()> {
    let release_path = "https://api.github.com/repos/Vjze/gemini_chat/releases";
    let body = ureq::get(release_path).call().unwrap();
    let res: serde_json::Value = body.into_json::<serde_json::Value>().unwrap();
    let assets = res.as_array().unwrap().get(0).unwrap();
    let z = assets["assets"].as_array().unwrap().get(0).unwrap();
    let get_updata_path = &z["browser_download_url"].to_string();
    let updata_path_s: String = get_updata_path
        .chars()
        .map(|x| match x {
            '"' => ' ',
            '\\' => ' ',
            _ => x,
        })
        .collect();

    let get_version = &assets["tag_name"].to_string();
    let version_s: String = get_version
        .chars()
        .map(|x| match x {
            '"' => ' ',
            '\\' => ' ',
            _ => x,
        })
        .collect();
    let version_len = &version_s.len() - 1;
    let version = &version_s[1..version_len];
    let v = version.to_string();
    if version > env!("CARGO_PKG_VERSION") {
        let _ = handle.upgrade_in_event_loop(move |ui| {
            let path_len = &updata_path_s.len() - 1;
            let updata_path = &updata_path_s[1..path_len];
            let version = env!("CARGO_PKG_VERSION");
            ui.set_version(version.into());
            ui.set_new_version(v.into());
            ui.set_updata_path(updata_path.into());
            ui.set_updata(true);
        });
    } else {
        let _ = handle.upgrade_in_event_loop(move |ui| {
            let version = env!("CARGO_PKG_VERSION");
            ui.set_version(version.into());
        });
    };
    Ok(())
}

pub async fn download_file(
    download_url: String,
    handle: Weak<My_App>,
) -> tokio::io::Result<(PathBuf, PathBuf)> {
    let bin_name = "gemini_chat.exe";
    let current_bin_path = std::env::current_exe().map_err(|_| ()).unwrap();
    let download_path = current_bin_path
        .parent()
        .ok_or(())
        .unwrap()
        .join(format!("tmp_{bin_name}"));
    let tmp_path = current_bin_path
        .parent()
        .ok_or(())
        .unwrap()
        .join(format!("tmp2_{bin_name}"));
    if let Err(e) = download(download_url, download_path.clone(), handle.clone()).await {
        // error!("Couldn't download UAD update: {}", e);
        // return Err(());
        println!("d11{}", e.to_string())
    }
    if let Err(e) = rename(&current_bin_path, &tmp_path) {
        // error!("[SelfUpdate] Couldn't rename binary path: {}", e);
        // return Err(());
        println!("d12{}", e.to_string())
    }
    if let Err(e) = rename(&download_path, &current_bin_path) {
        // error!("[SelfUpdate] Couldn't rename binary path: {}", e);
        // return Err(());
        println!("d13{}", e.to_string())
    }
    Ok((current_bin_path, tmp_path))
}
pub async fn download<T: ToString + Send>(
    url: T,
    dest_file: PathBuf,
    handle: Weak<My_App>,
) -> tokio::io::Result<()> {
    let url = url.to_string();
    match ureq::get(&url).call() {
        Ok(res) => {
            let mut file = fs::File::create(dest_file)
                .map_err(|e| e.to_string())
                .unwrap();

            if let Err(e) = copy(&mut res.into_reader(), &mut file) {
                // return Err(e.to_string());
                println!("d1{}", e.to_string())
            }
        }
        Err(e) => println!("d2{}", e.to_string()),
    }
    Ok(())
}
pub fn updata(relaunch_path: PathBuf, cleanup_path: PathBuf) -> tokio::io::Result<()> {
    let mut args = std::env::args();
    args.next();
    let mut args: Vec<_> = args.collect();
    if let Some(idx) = args.iter().position(|a| a == "--self-update-temp") {
        args.remove(idx);
        // Remove path passed after this arg
        args.remove(idx);
    }

    match std::process::Command::new(relaunch_path)
        .args(args)
        .arg("--self-update-temp")
        .arg(&cleanup_path)
        .spawn()
    {
        Ok(_) => {
            if let Err(e) = remove_file(cleanup_path) {
                // error!("Could not remove temp update file: {}", e);
                println!("{}", e.to_string())
            }
            std::process::exit(0)
        }
        Err(error) => {
            if let Err(e) = remove_file(cleanup_path) {
                // error!("Could not remove temp update file: {}", e);
                println!("u1{}", e.to_string())
            }
            // error!("Failed to update UAD: {}", error);
            println!("u2{}", error.to_string())
        }
    }

    Ok(())
}

pub fn rename<F, T>(from: F, to: T) -> Result<(), String>
where
    F: AsRef<Path>,
    T: AsRef<Path>,
{
    // 21 Fibonacci steps starting at 1 ms is ~28 seconds total
    // See https://github.com/rust-lang/rustup/pull/1873 where this was used by Rustup to work around
    // virus scanning file locks
    let from = from.as_ref();
    let to = to.as_ref();

    retry(Fibonacci::from_millis(1).take(21), || {
        match fs::rename(from, to) {
            Ok(_) => OperationResult::Ok(()),
            Err(e) => match e.kind() {
                io::ErrorKind::PermissionDenied => OperationResult::Retry(e),
                _ => OperationResult::Err(e),
            },
        }
    })
    .map_err(|e| e.to_string())
}
pub fn remove_file<P>(path: P) -> Result<(), String>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    retry(
        Fibonacci::from_millis(1).take(21),
        || match fs::remove_file(path) {
            Ok(_) => OperationResult::Ok(()),
            Err(e) => match e.kind() {
                io::ErrorKind::PermissionDenied => OperationResult::Retry(e),
                _ => OperationResult::Err(e),
            },
        },
    )
    .map_err(|e| e.to_string())
}
