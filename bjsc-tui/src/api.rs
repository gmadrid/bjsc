use bjsc::supabase::{
    coaching_request, fetch_answer_logs_request, fetch_deck_request, insert_answer_log_request,
    upsert_deck_request, AnswerLogEntry, AnswerLogRow, SupabaseConfig, UserDeckRow,
};
use bjsc::StudyMode;
use spaced_rep::Deck;

/// Fetch the user's deck from Supabase.
pub async fn fetch_user_deck(
    config: &SupabaseConfig,
    token: &str,
) -> Result<Option<UserDeckRow>, String> {
    let req = fetch_deck_request(config, token);
    let client = reqwest::Client::new();

    let mut builder = client.get(&req.url);
    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }

    let resp = builder.send().await.map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();

    if status == 406 || status == 404 {
        return Ok(None);
    }
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Fetch failed ({}): {}", status, text));
    }

    let row: UserDeckRow = resp.json().await.map_err(|e| e.to_string())?;
    Ok(Some(row))
}

/// Upsert the user's deck to Supabase.
pub async fn upsert_user_deck(
    config: &SupabaseConfig,
    token: &str,
    user_id: &str,
    mode: StudyMode,
    deck: &Deck,
) -> Result<(), String> {
    let req = upsert_deck_request(config, token, user_id, mode, deck);
    let client = reqwest::Client::new();

    let mut builder = client.post(&req.url);
    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }
    if let Some(body) = req.body {
        builder = builder.body(body);
    }

    let resp = builder.send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Upsert failed ({}): {}", status, text));
    }

    Ok(())
}

/// Log an answer to Supabase.
pub async fn insert_answer_log(
    config: &SupabaseConfig,
    token: &str,
    row: &AnswerLogRow,
) -> Result<(), String> {
    let req = insert_answer_log_request(config, token, row);
    let client = reqwest::Client::new();

    let mut builder = client.post(&req.url);
    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }
    if let Some(body) = req.body {
        builder = builder.body(body);
    }

    let resp = builder.send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Answer log failed ({}): {}", status, text));
    }

    Ok(())
}

/// Fetch recent answer logs from Supabase.
pub async fn fetch_answer_logs(
    config: &SupabaseConfig,
    token: &str,
    limit: u32,
) -> Result<Vec<AnswerLogEntry>, String> {
    let req = fetch_answer_logs_request(config, token, limit);
    let client = reqwest::Client::new();

    let mut builder = client.get(&req.url);
    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }

    let resp = builder.send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Fetch logs failed ({}): {}", status, text));
    }

    resp.json::<Vec<AnswerLogEntry>>()
        .await
        .map_err(|e| e.to_string())
}

/// Get coaching advice from the Claude-powered edge function.
pub async fn get_coaching(config: &SupabaseConfig, token: &str) -> Result<String, String> {
    let req = coaching_request(config, token);
    let client = reqwest::Client::new();

    let mut builder = client.post(&req.url);
    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }
    if let Some(body) = req.body {
        builder = builder.body(body);
    }

    let resp = builder.send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Coaching failed ({}): {}", status, text));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(json
        .get("coaching")
        .and_then(|v| v.as_str())
        .unwrap_or("No coaching available.")
        .to_string())
}
