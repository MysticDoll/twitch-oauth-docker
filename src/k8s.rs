use reqwest::{
    ClientBuilder,
};

pub(crate) async fn add_secret(twitch_token: &str) -> Result<(), String> {
    let cluster_url = std::env::var("K8S_CLUSTER_URL").unwrap_or("".to_owned());
    let namespace = std::env::var("K8S_NAMESPACE").unwrap_or("".to_owned());
    let token = std::env::var("K8S_TOKEN").unwrap_or("".to_owned());

    let secret_exists = ClientBuilder::new()
        .danger_accept_invalid_certs(
            std::env::var("K8S_CLUSTER_SELF_CERTIFIED")
                .unwrap_or("false".to_owned()) == "true"
        )
        .build()
        .map_err(|_| "reqwest client build failed".to_owned())?
        .get(
            &format!(
                "{}/api/v1/namespaces/{}/secrets/twitch-access-token",
                cluster_url,
                namespace
            )
        )
        .header(
            "Authorization",
            &format!(
                "bearer {}",
                token
            )
        )
        .send()
        .await
        .map_err(|e| e.to_string())?
        .status() != 404;

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(
            std::env::var("K8S_CLUSTER_SELF_CERTIFIED")
                .unwrap_or("false".to_owned()) == "true"
        )
        .build()
        .map_err(|_| "reqwest client build failed".to_owned())?;

    let mut request = if secret_exists {
        client.put(
            &format!(
                "{}/api/v1/namespaces/{}/secrets/twitch-access-token",
                cluster_url,
                namespace
            )
        )
    } else {
        client.post(
            &format!(
                "{}/api/v1/namespaces/{}/secrets/",
                cluster_url,
                namespace
            )
        )
    };

    request = request
        .header("Content-Type", "application/json")
        .header(
            "Authorization",
            &format!(
                "bearer {}",
                token
            )
        )
        .body(
            format!(
                "{{
                    \"apiVersion\": \"v1\",
                    \"kind\": \"Secret\",
                    \"metadata\": {{
                      \"name\": \"twitch-access-token\"
                    }},
                    \"data\":
                    {{ \"token\": \"{}\" }}
                 }}",
                twitch_token
            )
        );


    request
        .send()
        .await
        .map_err(|e| e.to_string())
        .and_then(|r| {
            if !r.status().is_success() {
                Err("request failed".to_owned())
            } else {
                Ok(r)
            }
        })
        .map(|_| ())
}
