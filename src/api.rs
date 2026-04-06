use crate::supabase::{
    coaching_request, fetch_answer_logs_request, fetch_deck_request, insert_answer_log_request,
    upsert_deck_request, AnswerLogEntry, AnswerLogRow, SupabaseConfig, UserDeckRow,
};
use crate::StudyMode;
use spaced_rep::Deck;

/// A minimal HTTP response abstraction shared across frontends.
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

/// Trait for sending HTTP requests, implemented per-platform.
pub trait HttpClient {
    fn request(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: Option<&str>,
    ) -> impl std::future::Future<Output = Result<HttpResponse, String>>;
}

/// Fetch the user's deck from Supabase.
pub async fn fetch_user_deck(
    client: &(impl HttpClient + ?Sized),
    config: &SupabaseConfig,
    token: &str,
) -> Result<Option<UserDeckRow>, String> {
    let req = fetch_deck_request(config, token);
    let resp = client
        .request(&req.method, &req.url, &req.headers, req.body.as_deref())
        .await?;

    if resp.status == 406 || resp.status == 404 {
        return Ok(None);
    }
    if !(200..300).contains(&resp.status) {
        return Err(format!("Fetch failed ({}): {}", resp.status, resp.body));
    }

    let row: UserDeckRow = serde_json::from_str(&resp.body).map_err(|e| e.to_string())?;
    Ok(Some(row))
}

/// Upsert the user's deck to Supabase.
pub async fn upsert_user_deck(
    client: &(impl HttpClient + ?Sized),
    config: &SupabaseConfig,
    token: &str,
    user_id: &str,
    mode: StudyMode,
    deck: &Deck,
) -> Result<(), String> {
    let req = upsert_deck_request(config, token, user_id, mode, deck);
    let resp = client
        .request(&req.method, &req.url, &req.headers, req.body.as_deref())
        .await?;

    if !(200..300).contains(&resp.status) {
        return Err(format!("Upsert failed ({}): {}", resp.status, resp.body));
    }
    Ok(())
}

/// Log an answer to Supabase.
pub async fn insert_answer_log(
    client: &(impl HttpClient + ?Sized),
    config: &SupabaseConfig,
    token: &str,
    row: &AnswerLogRow,
) -> Result<(), String> {
    let req = insert_answer_log_request(config, token, row);
    let resp = client
        .request(&req.method, &req.url, &req.headers, req.body.as_deref())
        .await?;

    if !(200..300).contains(&resp.status) {
        return Err(format!(
            "Answer log failed ({}): {}",
            resp.status, resp.body
        ));
    }
    Ok(())
}

/// Fetch recent answer logs from Supabase.
pub async fn fetch_answer_logs(
    client: &(impl HttpClient + ?Sized),
    config: &SupabaseConfig,
    token: &str,
    limit: u32,
) -> Result<Vec<AnswerLogEntry>, String> {
    let req = fetch_answer_logs_request(config, token, limit);
    let resp = client
        .request(&req.method, &req.url, &req.headers, req.body.as_deref())
        .await?;

    if !(200..300).contains(&resp.status) {
        return Err(format!(
            "Fetch logs failed ({}): {}",
            resp.status, resp.body
        ));
    }

    serde_json::from_str(&resp.body).map_err(|e| e.to_string())
}

/// Get coaching advice from the Claude-powered edge function.
pub async fn get_coaching(
    client: &(impl HttpClient + ?Sized),
    config: &SupabaseConfig,
    token: &str,
) -> Result<String, String> {
    let req = coaching_request(config, token);
    let resp = client
        .request(&req.method, &req.url, &req.headers, req.body.as_deref())
        .await?;

    if !(200..300).contains(&resp.status) {
        return Err(format!("Coaching failed ({}): {}", resp.status, resp.body));
    }

    let json: serde_json::Value = serde_json::from_str(&resp.body).map_err(|e| e.to_string())?;
    Ok(json
        .get("coaching")
        .and_then(|v| v.as_str())
        .unwrap_or("No coaching available.")
        .to_string())
}

/// Refresh an auth session using the refresh token.
pub async fn refresh_session(
    client: &(impl HttpClient + ?Sized),
    config: &SupabaseConfig,
    session: &crate::supabase::AuthSession,
) -> Option<crate::supabase::AuthSession> {
    let req = crate::supabase::refresh_token_request(config, &session.refresh_token);
    let resp = client
        .request(&req.method, &req.url, &req.headers, req.body.as_deref())
        .await
        .ok()?;

    if !(200..300).contains(&resp.status) {
        return None;
    }

    let json: serde_json::Value = serde_json::from_str(&resp.body).ok()?;
    crate::supabase::parse_refresh_response(&json, &session.refresh_token)
}
