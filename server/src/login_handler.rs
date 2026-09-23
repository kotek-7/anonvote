use std::collections::HashMap;

use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{self, Cookie},
};
use base64::{Engine, engine::general_purpose::URL_SAFE};

pub async fn login(jar: CookieJar) -> Result<(CookieJar, Redirect), StatusCode> {
    let state = URL_SAFE.encode(rand::random::<[u8; 32]>());
    let client_id = std::env::var("DISCORD_CLIENT_ID").map_err(|e| {
        tracing::error!("{e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let scopes = ["identify", "guilds", "guilds.members.read"];

    let web_url = std::env::var("WEB_URL").unwrap_or("http://127.0.0.1:8080".to_string());
    let redirect_uri = format!("{}{}", web_url, "/api/login/callback");

    let redirect_location = reqwest::Url::parse_with_params(
        "https://discord.com/oauth2/authorize",
        &[
            ("response_type", "code"),
            ("client_id", &client_id),
            ("scope", &scopes.join(" ")),
            ("state", &state),
            ("redirect_uri", &redirect_uri),
            ("prompt", "consent"),
            ("integration_type", "1"),
        ],
    )
    .map_err(|e| {
        tracing::error!("{e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let cookie = Cookie::build(("oauth2_state", state))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(cookie::SameSite::Lax)
        .build();

    Ok((
        jar.add(cookie),
        Redirect::to(&redirect_location.to_string()),
    ))
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct LoginCallbackParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

pub async fn login_callback(
    Query(query_params): Query<LoginCallbackParams>,
    jar: CookieJar,
) -> Result<impl IntoResponse, StatusCode> {
    // エラー確認
    if let Some(param_error) = query_params.error.as_deref() {
        let error_description = query_params.error_description;
        return response(jar, Some(param_error), error_description.as_deref());
    };

    // state 検証
    if verify_state(&jar, &query_params).is_err() {
        return response(jar, Some("auth failed"), Some("Auth client changed"));
    }

    // code 確認
    let Some(code) = query_params.code.as_deref() else {
        return response(
            jar,
            Some("auth failed"),
            Some("Invalid discord server response"),
        );
    };

    let _access_token = match exchange_code(code).await {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("{e}");
            return response(jar, Some("auth failed"), Some("Failed to auth"));
        },
    };

    response(jar, None, None)

}

fn response(
    jar: CookieJar,
    error: Option<&str>,
    error_description: Option<&str>,
) -> Result<(CookieJar, Redirect), StatusCode> {
    let web_url = std::env::var("WEB_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into());

    let mut web_url = reqwest::Url::parse(&web_url).map_err(|e| {
        tracing::error!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    {
        let mut query = web_url.query_pairs_mut();

        query.append_key_only("auth-completed");

        if let Some(error) = error {
            query.append_pair("error", error);
        }

        if let Some(description) = error_description {
            query.append_pair("error-desc", description);
        }
    }

    Ok((
        jar.remove(Cookie::from("oauth2_state")),
        Redirect::to(web_url.as_str()),
    ))
}

fn verify_state(jar: &CookieJar, query_params: &LoginCallbackParams) -> Result<(), ()> {
    let cookie_state = jar
        .get("oauth2_state")
        .map(|v| v.value().to_owned())
        .ok_or(())?;
    let param_state = query_params.state.as_ref().ok_or(())?;

    if cookie_state.trim() == param_state.trim() {
        Ok(())
    } else {
        Err(())
    }
}

async fn exchange_code(code: &str) -> anyhow::Result<String> {
    let web_url = std::env::var("WEB_URL").unwrap_or("http://127.0.0.1:8080".to_string());
    let redirect_uri = format!("{}{}", web_url, "/api/login/callback");

    let mut map = HashMap::new();
    map.insert("grant_type", "authorization_code");
    map.insert("code", code);
    map.insert("redirect_uri", redirect_uri.as_str());

    let api_endpoint =
        std::env::var("DISCORD_API_ENDPOINT").unwrap_or("https://discord.com/api/v10".to_string());
    let client_id = std::env::var("DISCORD_CLIENT_ID")?;
    let client_secret = std::env::var("DISCORD_CLIENT_SECRET")?;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Response {
        access_token: String,
        token_type: String,
        expires_in: i32,
        refresh_token: String,
        scope: String
    }

    let client = reqwest::Client::new();
    let res: Response = client
        .post(format!("{api_endpoint}/oauth2/token"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .basic_auth(client_id, Some(client_secret))
        .form(&map)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(res.access_token)
}
