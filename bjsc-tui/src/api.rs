use bjsc::api::{HttpClient, HttpResponse};
use std::sync::LazyLock;

pub(crate) static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub struct ReqwestClient;

impl HttpClient for ReqwestClient {
    async fn request(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: Option<&str>,
    ) -> Result<HttpResponse, String> {
        let mut builder = match method {
            "GET" => CLIENT.get(url),
            "POST" => CLIENT.post(url),
            _ => return Err(format!("Unsupported method: {}", method)),
        };
        for (k, v) in headers {
            builder = builder.header(k, v);
        }
        if let Some(body) = body {
            builder = builder.body(body.to_string());
        }

        let resp = builder.send().await.map_err(|e| e.to_string())?;
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        Ok(HttpResponse { status, body })
    }
}
