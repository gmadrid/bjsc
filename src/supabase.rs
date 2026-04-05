use crate::studymode::StudyMode;
use serde::{Deserialize, Serialize};
use spaced_rep::Deck;

/// Supabase project configuration.
#[derive(Debug, Clone)]
pub struct SupabaseConfig {
    pub base_url: String,
    pub anon_key: String,
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

/// Everything needed to make an HTTP request (no async, no HTTP client).
pub struct RequestDetails {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

fn common_headers(config: &SupabaseConfig, access_token: &str) -> Vec<(String, String)> {
    vec![
        ("apikey".to_string(), config.anon_key.clone()),
        (
            "Authorization".to_string(),
            format!("Bearer {}", access_token),
        ),
    ]
}

/// Build a request to refresh an access token.
pub fn refresh_token_request(config: &SupabaseConfig, refresh_token: &str) -> RequestDetails {
    RequestDetails {
        url: format!("{}/auth/v1/token?grant_type=refresh_token", config.base_url),
        method: "POST".to_string(),
        headers: vec![
            ("apikey".to_string(), config.anon_key.clone()),
            ("Content-Type".to_string(), "application/json".to_string()),
        ],
        body: Some(format!(r#"{{"refresh_token":"{}"}}"#, refresh_token)),
    }
}

/// Build a request to fetch the user's deck.
pub fn fetch_deck_request(config: &SupabaseConfig, access_token: &str) -> RequestDetails {
    let mut headers = common_headers(config, access_token);
    // Return a single object instead of an array
    headers.push((
        "Accept".to_string(),
        "application/vnd.pgrst.object+json".to_string(),
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
) -> RequestDetails {
    let mut headers = common_headers(config, access_token);
    headers.push(("Content-Type".to_string(), "application/json".to_string()));
    // Upsert: insert or update on conflict
    headers.push((
        "Prefer".to_string(),
        "resolution=merge-duplicates".to_string(),
    ));

    let row = UserDeckRow {
        id: None,
        user_id: user_id.to_string(),
        study_mode: mode,
        deck: deck.clone(),
        updated_at: None, // let the DB set this
    };

    RequestDetails {
        url: format!("{}/rest/v1/user_deck?on_conflict=user_id", config.base_url),
        method: "POST".to_string(),
        headers,
        body: Some(serde_json::to_string(&row).unwrap_or_default()),
    }
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
) -> RequestDetails {
    let mut headers = common_headers(config, access_token);
    headers.push(("Content-Type".to_string(), "application/json".to_string()));

    RequestDetails {
        url: format!("{}/rest/v1/answer_log", config.base_url),
        method: "POST".to_string(),
        headers,
        body: Some(serde_json::to_string(row).unwrap_or_default()),
    }
}

/// Build a request to fetch answer logs for stats (recent N entries).
pub fn fetch_answer_logs_request(
    config: &SupabaseConfig,
    access_token: &str,
    limit: u32,
) -> RequestDetails {
    let mut headers = common_headers(config, access_token);
    headers.push(("Accept".to_string(), "application/json".to_string()));

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

/// Decode the JWT payload as a JSON Value.
fn jwt_payload(token: &str) -> Option<serde_json::Value> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload = parts[1];
    let padded = match payload.len() % 4 {
        2 => format!("{}==", payload),
        3 => format!("{}=", payload),
        _ => payload.to_string(),
    };
    let decoded = base64_decode(&padded)?;
    serde_json::from_slice(&decoded).ok()
}

/// Extract the `sub` (user ID) from a JWT without verification.
pub fn user_id_from_jwt(token: &str) -> Option<String> {
    jwt_payload(token)?
        .get("sub")?
        .as_str()
        .map(|s| s.to_string())
}

/// Extract the email from a JWT without verification.
pub fn email_from_jwt(token: &str) -> Option<String> {
    jwt_payload(token)?
        .get("email")?
        .as_str()
        .map(|s| s.to_string())
}

/// Minimal base64 URL-safe decoder (no external crate needed).
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    // Convert URL-safe base64 to standard base64
    let standard: String = input
        .chars()
        .map(|c| match c {
            '-' => '+',
            '_' => '/',
            c => c,
        })
        .collect();

    // Simple base64 decode
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0u32;

    for c in standard.bytes() {
        let val = if c == b'=' {
            break;
        } else if let Some(pos) = TABLE.iter().position(|&b| b == c) {
            pos as u32
        } else {
            return None;
        };
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id_from_jwt() {
        // A fake JWT with sub claim
        // Header: {"alg":"HS256","typ":"JWT"}
        // Payload: {"sub":"12345678-abcd-1234-abcd-123456789abc","role":"authenticated"}
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3OC1hYmNkLTEyMzQtYWJjZC0xMjM0NTY3ODlhYmMiLCJyb2xlIjoiYXV0aGVudGljYXRlZCJ9.fake_signature";
        let uid = user_id_from_jwt(token);
        assert_eq!(
            uid,
            Some("12345678-abcd-1234-abcd-123456789abc".to_string())
        );
    }

    #[test]
    fn test_user_id_from_invalid_jwt() {
        assert_eq!(user_id_from_jwt("not-a-jwt"), None);
        assert_eq!(user_id_from_jwt(""), None);
    }
}
