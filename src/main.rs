use axum::{
    routing::get,
    Router,
    http::StatusCode,
    extract::Query
};

mod k8s;

#[tokio::main]
async fn main() {
    let client_id = std::env::var("CLIENT_ID").unwrap_or("".to_owned());
    let client_secret = std::env::var("CLIENT_SECRET").unwrap_or("".to_owned());
    let redirect_uri = std::env::var("REDIRECT_URI").unwrap_or("".to_owned());
    let oauth_url = format!(
        "https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=chat:read+chat:edit+channel:read:redemptions",
        &client_id,
        &redirect_uri
    );
    let app = Router::new()
        .route("/", get(|| async {
            (
                StatusCode::SEE_OTHER,
                [(
                    "Location", oauth_url
                )]
            )
        }))
        .route(
            "/auth/",
            get(|query: Query<std::collections::HashMap<String,String>>| async move {
                if let Some(code) = query.0.get("code"){
                    let access_token = get_access_token(&code, &client_id, &client_secret, &redirect_uri).await;
                    if let Err(e) = crate::k8s::add_secret(&access_token).await {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            e
                        )
                    } else {
                        (
                            StatusCode::OK,
                            "success to store to k8s secret".to_owned()
                        )
                    }
                } else {
                    (
                        StatusCode::BAD_REQUEST,
                        "missing code".to_owned()
                    )
                }
            })
        );

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_access_token(code: &str, client_id: &str, client_secret: &str, redirect_uri: &str) -> String {
    let url = "https://id.twitch.tv/oauth2/token";

    if let Ok(res) = reqwest::Client::new().post(url)
        .query(&[
            ("client_id", client_id.to_owned()),
            ("client_secret", client_secret.to_owned()),
            ("redirect_uri", redirect_uri.to_owned()),
            ("code", code.to_owned()),
            ("grant_type", "authorization_code".to_owned())
        ])
        .send()
        .await {
            let text = res.text().await.unwrap_or("error".to_owned());
            text

        } else {
            "request error".to_owned()
        }
}
