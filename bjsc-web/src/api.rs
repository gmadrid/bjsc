use bjsc::api::{HttpClient, HttpResponse};
use gloo_net::http;
use std::borrow::Cow;

pub struct GlooClient;

impl HttpClient for GlooClient {
    async fn request(
        &self,
        method: &str,
        url: &str,
        headers: &[(Cow<'static, str>, Cow<'static, str>)],
        body: Option<&str>,
    ) -> Result<HttpResponse, String> {
        let mut builder = match method {
            "GET" => http::Request::get(url),
            "POST" => http::Request::post(url),
            _ => return Err(format!("Unsupported method: {}", method)),
        };
        for (k, v) in headers {
            builder = builder.header(k.as_ref(), v.as_ref());
        }
        let request = if let Some(body) = body {
            builder.body(body).map_err(|e| format!("{}", e))?
        } else {
            builder.build().map_err(|e| format!("{}", e))?
        };

        let resp = request.send().await.map_err(|e| format!("{}", e))?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Ok(HttpResponse { status, body })
    }
}
