use reqwest::ClientBuilder;

pub(crate) async fn add_secret(twitch_token: &str) -> Result<(), String> {
    let cluster_url = std::env::var("K8S_CLUSTER_URL").unwrap_or("".to_owned());
    let namespace = std::env::var("K8S_NAMESPACE").unwrap_or("".to_owned());
    let token = std::env::var("K8S_TOKEN").unwrap_or("".to_owned());
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(
            std::env::var("K8S_CLUSTER_SELF_CERTIFIED")
                .unwrap_or("false".to_owned()) == "true"
        )
        .build()
        .map_err(|_| "reqwest client build failed".to_owned())?
        .post(
            &format!(
                "{}/api/v1/namespaces/{}/secrets",
                cluster_url,
                namespace
            )
        )
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
                    \"name\": \"twitch-access-token\",
                    \"data\":
                    {{ \"token\": \"{}\" }}
                 }}",
                twitch_token
            )
        );

    client
        .send()
        .await
        .map(|i| {
            println!("{:?}", i);
            ()
        })
        .map_err(|e| e.to_string())
}
