//! bjsc-specific Supabase schema and request builders.
//!
//! Generic primitives (config struct, request shape, auth session, JWT
//! helpers, refresh) live in the `leit-auth` crate; this module re-exports
//! them so callers can keep their existing `bjsc::supabase::*` imports.

use crate::studymode::StudyMode;
use serde::{Deserialize, Serialize};
use spaced_rep::Deck;
use std::borrow::Cow;

pub use leit_auth::{
    AuthSession, RequestDetails, SupabaseConfig, common_headers, email_from_jwt, is_jwt_expired,
    parse_refresh_response, refresh_token_request, user_id_from_jwt,
};

pub const SUPABASE_URL: &str = "https://pecwxusghnxlvzmfcqrj.supabase.co";
pub const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InBlY3d4dXNnaG54bHZ6bWZjcXJqIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NzUzNTY3MjUsImV4cCI6MjA5MDkzMjcyNX0.LwgaAHruQ8cA3mHrtCCB00WSqttpwRusAf0Y1WEFWuE";

/// Default config for the bjsc Supabase project.
pub fn default_config() -> SupabaseConfig {
    SupabaseConfig {
        base_url: SUPABASE_URL.to_string(),
        anon_key: SUPABASE_ANON_KEY.to_string(),
    }
}

/// Matches the `user_deck` table schema.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDeckRow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub user_id: String,
    pub study_mode: StudyMode,
    pub deck: Deck,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Build a request to fetch the user's deck.
pub fn fetch_deck_request(config: &SupabaseConfig, access_token: &str) -> RequestDetails {
    let mut headers = common_headers(config, access_token);
    // PostgREST: return a single object instead of an array.
    headers.push((
        Cow::Borrowed("Accept"),
        Cow::Borrowed("application/vnd.pgrst.object+json"),
    ));

    RequestDetails {
        url: format!("{}/rest/v1/user_deck?select=*", config.base_url),
        method: "GET".to_string(),
        headers,
        body: None,
    }
}

/// Build a request to upsert the user's deck.
pub fn upsert_deck_request(
    config: &SupabaseConfig,
    access_token: &str,
    user_id: &str,
    mode: StudyMode,
    deck: &Deck,
) -> Result<RequestDetails, String> {
    let mut headers = common_headers(config, access_token);
    headers.push((
        Cow::Borrowed("Content-Type"),
        Cow::Borrowed("application/json"),
    ));
    headers.push((
        Cow::Borrowed("Prefer"),
        Cow::Borrowed("resolution=merge-duplicates"),
    ));

    let row = UserDeckRow {
        id: None,
        user_id: user_id.to_string(),
        study_mode: mode,
        deck: deck.clone(),
        updated_at: None,
    };

    Ok(RequestDetails {
        url: format!("{}/rest/v1/user_deck?on_conflict=user_id", config.base_url),
        method: "POST".to_string(),
        headers,
        body: Some(serde_json::to_string(&row).map_err(|e| e.to_string())?),
    })
}

/// Row for the answer_log table.
#[derive(Debug, Serialize, Deserialize)]
pub struct AnswerLogRow {
    pub user_id: String,
    pub table_index: String,
    pub correct: bool,
    pub player_action: String,
    pub correct_action: String,
}

/// Build a request to insert an answer log entry.
pub fn insert_answer_log_request(
    config: &SupabaseConfig,
    access_token: &str,
    row: &AnswerLogRow,
) -> Result<RequestDetails, String> {
    let mut headers = common_headers(config, access_token);
    headers.push((
        Cow::Borrowed("Content-Type"),
        Cow::Borrowed("application/json"),
    ));

    Ok(RequestDetails {
        url: format!("{}/rest/v1/answer_log", config.base_url),
        method: "POST".to_string(),
        headers,
        body: Some(serde_json::to_string(row).map_err(|e| e.to_string())?),
    })
}

/// Build a request to fetch answer logs for stats (recent N entries).
pub fn fetch_answer_logs_request(
    config: &SupabaseConfig,
    access_token: &str,
    limit: u32,
) -> RequestDetails {
    let mut headers = common_headers(config, access_token);
    headers.push((Cow::Borrowed("Accept"), Cow::Borrowed("application/json")));

    RequestDetails {
        url: format!(
            "{}/rest/v1/answer_log?select=*&order=created_at.desc&limit={}",
            config.base_url, limit
        ),
        method: "GET".to_string(),
        headers,
        body: None,
    }
}

/// A log entry as returned from the API.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnswerLogEntry {
    pub table_index: String,
    pub correct: bool,
    pub player_action: String,
    pub correct_action: String,
    pub created_at: String,
}

/// Build a request to call the coaching edge function.
pub fn coaching_request(config: &SupabaseConfig, access_token: &str) -> RequestDetails {
    let mut headers = common_headers(config, access_token);
    headers.push((
        Cow::Borrowed("Content-Type"),
        Cow::Borrowed("application/json"),
    ));

    RequestDetails {
        url: format!("{}/functions/v1/coaching", config.base_url),
        method: "POST".to_string(),
        headers,
        body: Some("{}".to_string()),
    }
}
