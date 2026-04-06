use bjsc::supabase::{parse_refresh_response, user_id_from_jwt, SupabaseConfig};
use std::fs;
use std::path::PathBuf;

pub type AuthTokens = bjsc::supabase::AuthSession;

fn auth_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".bjsc").join("auth.json")
}

pub fn load_stored_tokens() -> Option<AuthTokens> {
    let path = auth_path();
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_tokens(tokens: &AuthTokens) {
    use std::os::unix::fs::OpenOptionsExt;

    let path = auth_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(content) = serde_json::to_string_pretty(tokens) {
        let _ = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)
            .and_then(|mut f| {
                use std::io::Write;
                f.write_all(content.as_bytes())
            });
    }
}

#[allow(dead_code)]
pub fn clear_tokens() {
    let _ = fs::remove_file(auth_path());
}

/// Refresh the access token using the refresh token (blocking).
pub fn refresh_tokens(
    config: &SupabaseConfig,
    tokens: &AuthTokens,
    rt: &tokio::runtime::Runtime,
) -> Option<AuthTokens> {
    let req = bjsc::supabase::refresh_token_request(config, &tokens.refresh_token);
    let client = reqwest::Client::new();

    let result = rt.block_on(async {
        let mut builder = client.post(&req.url);
        for (k, v) in &req.headers {
            builder = builder.header(k, v);
        }
        if let Some(body) = req.body {
            builder = builder.body(body);
        }
        builder.send().await
    });

    let resp = result.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    let json: serde_json::Value = rt.block_on(resp.json()).ok()?;
    let new_tokens = parse_refresh_response(&json, &tokens.refresh_token)?;
    save_tokens(&new_tokens);
    Some(new_tokens)
}

/// Refresh the access token using the refresh token (async).
pub async fn refresh_tokens_async(
    config: &SupabaseConfig,
    tokens: &AuthTokens,
) -> Option<AuthTokens> {
    let req = bjsc::supabase::refresh_token_request(config, &tokens.refresh_token);
    let client = reqwest::Client::new();

    let mut builder = client.post(&req.url);
    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }
    if let Some(body) = req.body {
        builder = builder.body(body);
    }
    let resp = builder.send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    let json: serde_json::Value = resp.json().await.ok()?;
    let new_tokens = parse_refresh_response(&json, &tokens.refresh_token)?;
    save_tokens(&new_tokens);
    Some(new_tokens)
}

/// Run the browser-based OAuth flow.
/// Opens the user's browser, waits for the redirect, extracts tokens.
pub fn login(config: &SupabaseConfig) -> Result<AuthTokens, String> {
    // Start a local server on a random port
    let server =
        tiny_http::Server::http("127.0.0.1:0").map_err(|e| format!("Server error: {}", e))?;
    let port = server.server_addr().to_ip().unwrap().port();
    let redirect_url = format!("http://127.0.0.1:{}/callback", port);

    let auth_url = format!(
        "{}/auth/v1/authorize?provider=google&redirect_to={}",
        config.base_url, redirect_url
    );

    println!("Opening browser for Google sign-in...");
    println!("If the browser doesn't open, visit: {}", auth_url);
    let _ = open::that(&auth_url);

    // Serve the callback page that extracts hash fragments
    let callback_html = format!(
        r#"<!DOCTYPE html>
<html><body>
<p id="status">Processing...</p>
<script>
const hash = window.location.hash.substring(1);
if (hash) {{
    fetch('http://127.0.0.1:{}/tokens', {{
        method: 'POST',
        body: hash
    }}).then(() => {{
        document.getElementById('status').textContent = 'Signed in! You can close this tab and return to the terminal.';
    }});
}} else {{
    document.getElementById('status').textContent = 'No tokens found. Please try again.';
}}
</script>
</body></html>"#,
        port
    );

    // Wait for the callback request
    let mut access_token = None;
    let mut refresh_token = None;

    for _ in 0..10 {
        // Give up after 10 requests (shouldn't need more than 2)
        let request = server
            .recv_timeout(std::time::Duration::from_secs(120))
            .map_err(|e| format!("Timeout waiting for auth: {}", e))?;

        let Some(mut request) = request else {
            return Err("Timed out waiting for authentication.".to_string());
        };

        let url = request.url().to_string();

        if url.starts_with("/callback") {
            // Serve the HTML page that will POST the hash fragments back
            let response = tiny_http::Response::from_string(&callback_html).with_header(
                "Content-Type: text/html"
                    .parse::<tiny_http::Header>()
                    .unwrap(),
            );
            let _ = request.respond(response);
        } else if url == "/tokens" {
            // Receive the hash fragment data
            let mut body = String::new();
            let reader = request.as_reader();
            let _ = reader.read_to_string(&mut body);

            let params: std::collections::HashMap<String, String> = body
                .split('&')
                .filter_map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    Some((parts.next()?.to_string(), parts.next()?.to_string()))
                })
                .collect();

            access_token = params.get("access_token").cloned();
            refresh_token = params.get("refresh_token").cloned();

            let response = tiny_http::Response::from_string("OK").with_header(
                "Access-Control-Allow-Origin: *"
                    .parse::<tiny_http::Header>()
                    .unwrap(),
            );
            let _ = request.respond(response);
            break;
        } else if request.method() == &tiny_http::Method::Options {
            // Handle CORS preflight
            let response = tiny_http::Response::from_string("")
                .with_header(
                    "Access-Control-Allow-Origin: *"
                        .parse::<tiny_http::Header>()
                        .unwrap(),
                )
                .with_header(
                    "Access-Control-Allow-Methods: POST"
                        .parse::<tiny_http::Header>()
                        .unwrap(),
                )
                .with_header(
                    "Access-Control-Allow-Headers: *"
                        .parse::<tiny_http::Header>()
                        .unwrap(),
                );
            let _ = request.respond(response);
        } else {
            let response = tiny_http::Response::from_string("Not found").with_status_code(404);
            let _ = request.respond(response);
        }
    }

    let access_token = access_token.ok_or("No access token received")?;
    let refresh_token = refresh_token.unwrap_or_default();
    let user_id = user_id_from_jwt(&access_token).ok_or("Could not extract user ID from token")?;
    let email = bjsc::supabase::email_from_jwt(&access_token).unwrap_or_default();

    let tokens = AuthTokens {
        access_token,
        refresh_token,
        user_id,
        email,
    };
    save_tokens(&tokens);
    Ok(tokens)
}
