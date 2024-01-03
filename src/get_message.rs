use crate::My_App;
use reqwest::Client;
use slint::Weak;
pub async fn chat_gemini_say(text: String, handle: Weak<My_App>) -> tokio::io::Result<()> {
    let key = "AIzaSyBjBAuaOiFn3edfMi0D3yX_PHYh8qkFhpA";
    let api_url =
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key=";
    let url = format!("{}{}", api_url, key);

    let client = Client::new();
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "contents": [{"parts":[{"text": text}]}]
        }))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let _ = handle.upgrade_in_event_loop(move |ui| {
        let response_json: serde_json::Value = serde_json::from_str(&response).unwrap();
        // println!("message = {:#?}",response_json);
        let choices = response_json["candidates"].as_array().unwrap();
        let replys = choices[0]["content"]["parts"].as_array().unwrap();
        let reply = replys[0]["text"].as_str().unwrap();
        ui.set_reply(reply.into())
    });
    Ok(())
}
