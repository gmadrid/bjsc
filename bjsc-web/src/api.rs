use bjsc::supabase::{fetch_deck_request, upsert_deck_request, SupabaseConfig, UserDeckRow};
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
