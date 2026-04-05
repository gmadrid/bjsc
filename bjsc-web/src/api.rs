use bjsc::supabase::{
    fetch_answer_logs_request, fetch_deck_request, insert_answer_log_request, upsert_deck_request,
    AnswerLogEntry, AnswerLogRow, SupabaseConfig, UserDeckRow,
};
use bjsc::StudyMode;
use gloo_net::http;
use spaced_rep::Deck;

/// Fetch the user's deck from Supabase. Returns None if no row exists yet.
pub async fn fetch_user_deck(
    config: &SupabaseConfig,
    token: &str,
) -> Result<Option<UserDeckRow>, String> {
    let req = fetch_deck_request(config, token);

    let mut builder = http::Request::get(&req.url);
    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }
    let request = builder.build().map_err(|e| format!("{}", e))?;
    let resp = request.send().await.map_err(|e| format!("{}", e))?;

    if resp.status() == 406 || resp.status() == 404 {
        return Ok(None);
    }
    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Fetch failed ({}): {}", resp.status(), text));
    }

    let row: UserDeckRow = resp
        .json::<UserDeckRow>()
        .await
        .map_err(|e| format!("{}", e))?;
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

    let mut builder = http::Request::post(&req.url);
    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }
    let request = if let Some(body) = &req.body {
        builder.body(body.as_str()).map_err(|e| format!("{}", e))?
    } else {
        builder.build().map_err(|e| format!("{}", e))?
    };
    let resp = request.send().await.map_err(|e| format!("{}", e))?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Upsert failed ({}): {}", resp.status(), text));
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

    let mut builder = http::Request::post(&req.url);
    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }
    let request = if let Some(body) = &req.body {
        builder.body(body.as_str()).map_err(|e| format!("{}", e))?
    } else {
        builder.build().map_err(|e| format!("{}", e))?
    };
    let resp = request.send().await.map_err(|e| format!("{}", e))?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Answer log failed ({}): {}", resp.status(), text));
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

    let mut builder = http::Request::get(&req.url);
    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }
    let request = builder.build().map_err(|e| format!("{}", e))?;
    let resp = request.send().await.map_err(|e| format!("{}", e))?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Fetch logs failed ({}): {}", resp.status(), text));
    }

    resp.json::<Vec<AnswerLogEntry>>()
        .await
        .map_err(|e| format!("{}", e))
}
