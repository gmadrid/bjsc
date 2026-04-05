use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

const STORAGE_KEY: &str = "bjsc_auth";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
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

    // Clear the hash from the URL
    let _ = location.set_hash("");

    let state = AuthState {
        access_token,
        refresh_token,
        user_id,
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

/// Build the Google OAuth login URL via Supabase.
pub fn google_login_url(supabase_url: &str) -> String {
    let redirect = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();
    format!(
        "{}/auth/v1/authorize?provider=google&redirect_to={}",
        supabase_url, redirect
    )
}
