use super::types::ApiCredentials;



pub fn fetch_user_credentials(user_token: &str) -> Result<ApiCredentials, String>{
    Ok(ApiCredentials{
        openai_token: "placeholder".to_string(),
        onshape_access_key: "placeholder".to_string(),
        onshape_secret_key: "placeholder".to_string()
    })
}