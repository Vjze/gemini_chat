use crate::My_App;
use slint::Weak;
pub async fn chat_gemini_say(text: String, handle: Weak<My_App>) -> tokio::io::Result<()> {
    let key = "AIzaSyBjBAuaOiFn3edfMi0D3yX_PHYh8qkFhpA";
    let api_url =
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key=";
    let url = format!("{}{}", api_url, key);

    let body = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_json(ureq::json!({
            "contents": [{"parts":[{"text": text}]}]
        })).unwrap();
    let _ = handle.upgrade_in_event_loop(move |ui| {
        let response_json = body.into_json::<serde_json::Value>().unwrap();
        let choices = response_json["candidates"].as_array().unwrap();
        let replys = choices[0]["content"]["parts"].as_array().unwrap();
        let reply = replys[0]["text"].as_str().unwrap();
        ui.set_reply(reply.into())
    });
    Ok(())
}
