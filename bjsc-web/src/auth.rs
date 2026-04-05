use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

const STORAGE_KEY: &str = "bjsc_auth";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
    #[serde(default)]
    pub email: String,
}

/// Check the URL hash for OAuth tokens (after Supabase redirect).
/// Returns Some(AuthState) if tokens were found, and clears the hash.
pub fn check_url_for_tokens() -> Option<AuthState> {
    let window = web_sys::window()?;
    let location = window.location();
    let hash = location.hash().ok()?;

    if !hash.contains("access_token=") {
        return None;
    }

    // Parse hash fragment: #access_token=...&refresh_token=...&...
    let params: std::collections::HashMap<String, String> = hash
        .trim_start_matches('#')
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            Some((parts.next()?.to_string(), parts.next()?.to_string()))
        })
        .collect();

    let access_token = params.get("access_token")?.clone();
    let refresh_token = params.get("refresh_token").cloned().unwrap_or_default();
    let user_id = bjsc::supabase::user_id_from_jwt(&access_token)?;
    let email = bjsc::supabase::email_from_jwt(&access_token).unwrap_or_default();

    // Clear the hash from the URL
    let _ = location.set_hash("");

    let state = AuthState {
        access_token,
        refresh_token,
        user_id,
        email,
    };
    save_to_storage(&state);
    Some(state)
}

/// Load auth state from localStorage.
pub fn load_from_storage() -> Option<AuthState> {
    LocalStorage::get::<AuthState>(STORAGE_KEY).ok()
}

/// Save auth state to localStorage.
pub fn save_to_storage(state: &AuthState) {
    let _ = LocalStorage::set(STORAGE_KEY, state);
}

/// Clear auth state from localStorage.
pub fn clear_storage() {
    LocalStorage::delete(STORAGE_KEY);
}

/// Refresh the access token using the refresh token.
pub async fn refresh_session(
    config: &bjsc::supabase::SupabaseConfig,
    state: &AuthState,
) -> Option<AuthState> {
    let req = bjsc::supabase::refresh_token_request(config, &state.refresh_token);

    let builder = gloo_net::http::Request::post(&req.url);
    let mut b = builder;
    for (k, v) in &req.headers {
        b = b.header(k, v);
    }
    let request = b.body(req.body?.as_str()).ok()?;
    let resp = request.send().await.ok()?;

    if !resp.ok() {
        return None;
    }

    let json: serde_json::Value = resp.json().await.ok()?;
    let access_token = json.get("access_token")?.as_str()?.to_string();
    let refresh_token = json
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .unwrap_or(&state.refresh_token)
        .to_string();
    let user_id = bjsc::supabase::user_id_from_jwt(&access_token)?;
    let email = bjsc::supabase::email_from_jwt(&access_token).unwrap_or_default();

    let new_state = AuthState {
        access_token,
        refresh_token,
        user_id,
        email,
    };
    save_to_storage(&new_state);
    Some(new_state)
}

/// Build the Google OAuth login URL via Supabase.
pub fn google_login_url(supabase_url: &str) -> String {
    let redirect = web_sys::window()
        .and_then(|w| {
            let loc = w.location();
            let origin = loc.origin().ok()?;
            let pathname = loc.pathname().ok().unwrap_or_default();
            Some(format!("{}{}", origin, pathname))
        })
        .unwrap_or_default();
    format!(
        "{}/auth/v1/authorize?provider=google&redirect_to={}",
        supabase_url, redirect
    )
}
