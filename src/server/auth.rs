use std::error::Error;

use crate::server::types::{ONSHAPE_API, OPENAI_API};
use crate::util::PolybrainError;

use super::types::{ApiCredentials, UserDocument, UserInfo};

use aes::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::BlockMode;
use block_modes::Cbc;
use hex::decode;
use log::{info, warn};
use mongodb::{
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client, Collection,
};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct MongoUtil {
    mongo_client: Client,
    user_collection: Collection<UserDocument>,
}

type Aes128Cbc = Cbc<Aes128, Pkcs7>;

/// AES128 Decryption
pub fn decrypt(encrypted_data: &str) -> String {
    let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY must be set");

    // Hash the secret key using SHA-256
    let mut hasher = Sha256::new();
    hasher.update(secret_key);
    let hash_result = hasher.finalize();

    // Take the first 16 bytes of the hash as the key
    let key = {
        let mut key = [0u8; 16];
        key.copy_from_slice(&hash_result[..16]);
        key
    };

    let encrypted_data = decode(encrypted_data).unwrap();
    let (iv, encrypted_data) = encrypted_data.split_at(16);
    let cipher = Aes128Cbc::new_from_slices(&key, iv).expect("Failed to process cipher");
    let decrypted_data = cipher
        .decrypt_vec(encrypted_data)
        .expect("Failed to decrypt key: decrypt_vec failed");

    String::from_utf8(decrypted_data).expect("Failed to decrypt key: could not assemble UTF-8")
}

impl MongoUtil {
    pub async fn new() -> Result<Self, mongodb::error::Error> {
        let mut client_options =
            ClientOptions::parse(std::env::var("MONGODB_URL").expect("MONGODB_URL must be set"))
                .await?;

        // Set the server_api field of the client_options object to set the version of the Stable API on the client
        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);

        // Get a handle to the cluster
        let mongo_client = Client::with_options(client_options)?;
        let user_collection = mongo_client
            .database(
                std::env::var("MONGODB_DATABASE")
                    .expect("MONGODB_DATABASE must be set")
                    .as_str(),
            )
            .collection("users");

        let new_instance = MongoUtil {
            mongo_client,
            user_collection,
        };

        new_instance.ping().await.expect("MongoDB ping failed");
        Ok(new_instance)
    }
    pub async fn ping(&self) -> Result<(), mongodb::error::Error> {
        _ = &self
            .mongo_client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await?;
        Ok(())
    }

    pub async fn get_user(&self, user_id: &str) -> Option<UserDocument> {
        let filter = doc! { "user_id": user_id };
        let user = self
            .user_collection
            .find_one(filter.clone(), None)
            .await
            .expect("Fatal MongoDB error on user query");
        if user.is_some() {
            info!("successfully fetched user with id {user_id}");
        } else {
            warn!("user with id {user_id} does not exist in MongoDB");
        }
        user
    }
}

pub async fn fetch_user_id(user_token: &str) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    println!("fetching user id...");
    let res = client
        .get("https://polybrain.xyz/auth0/user-data")
        .header("Cookie", format!("polybrain-session={user_token}"))
        .send()
        .await
        .map_err(Box::new)?;

    if res.status().is_success() {
        let user_info: UserInfo = serde_json::from_str(&res.text().await.unwrap())
            .expect("Unable to deserialize good polybrain server response");
        Ok(user_info.user_id)
    } else {
        println!(
            "polybrain-server request failed with body:\n{}",
            res.text().await.unwrap()
        );
        Err(Box::from("Failed to authenticate using user token"))
    }
}

async fn validate_credentials(credentials: &ApiCredentials) -> Result<(), PolybrainError> {
    println!("validating credentials...");

    println!("pinging onshape...");
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/documents?limit=1", ONSHAPE_API))
        .basic_auth(
            &credentials.onshape_access_key,
            Some(&credentials.onshape_secret_key),
        )
        .send()
        .await;

    match response {
        Ok(r) => {
            if r.status().is_success() {
                println!("OnShape ping successful");
            } else {
                let response_message = r.text().await.unwrap();
                println!("OnShape responded with error: \n{}", response_message);
                return Err(PolybrainError::BadRequest("Bad OnShape Credentials".to_string()));
            }
        }
        Err(err) => {
            println!("Call to onshape had internal error: {}", err);
            return Err(PolybrainError::InternalError("Call to OnShape failed".to_owned()));
        }
    }

    println!("pinging open ai...");
    let response = client
        .get(format!("{}/models", OPENAI_API))
        .bearer_auth(&credentials.openai_token)
        .send()
        .await;

    match response {
        Ok(r) => {
            if r.status().is_success() {
                println!("OpenAI ping successful");
            } else {
                let response_message = r.text().await.unwrap();
                println!("OpenAI responded with error: \n{}", response_message);
                return Err(PolybrainError::BadRequest("Bad OpenAI Credentials".to_string()));
            }
        }
        Err(err) => {
            println!("Call to OpenAI had internal error: {}", err);
            return Err(PolybrainError::InternalError("Call to OpenAI failed".to_owned()));
        }
    }

    println!("All credentials are valid");
    Ok(())
}

pub async fn fetch_user_credentials(user_token: &str) -> Result<ApiCredentials, PolybrainError> {
    // TODO: make a global, mutex-protected instance to avoid having to reconnect for each connection
    let mongo_instance = MongoUtil::new()
        .await
        .expect("Failed to connect to mongodb");

    let user_id = match fetch_user_id(user_token).await {
        Ok(id) => id,
        Err(err) => {
            println!("failed to get user token: {}", err);
            return Err(PolybrainError::BadRequest("Invalid user token".to_owned()));
        }
    };

    println!("resolved user id as: {}", user_id);

    let user = match mongo_instance.get_user(&user_id).await {
        Some(u) => u,
        None => {
            println!("No corresponding user exists in mongodb");
            return Err(PolybrainError::BadRequest("Missing user credentials".to_owned()));
        }
    };

    if [
        &user.credentials.open_ai_api,
        &user.credentials.onshape_access,
        &user.credentials.onshape_secret,
    ]
    .contains(&&None)
    {
        return Err(PolybrainError::BadRequest(("User is missing some credentials".to_string())));
    }

    let openai_cyphertext = user.credentials.open_ai_api.unwrap();
    let onshape_access_cyphertext = user.credentials.onshape_access.unwrap();
    let onshape_secret_cyphertext = user.credentials.onshape_secret.unwrap();

    let credentials = ApiCredentials {
        openai_token: decrypt(&openai_cyphertext),
        onshape_access_key: decrypt(&onshape_access_cyphertext),
        onshape_secret_key: decrypt(&onshape_secret_cyphertext),
    };

    println!("Got credentials: {:?}", credentials);
    validate_credentials(&credentials).await?;

    Ok(credentials)
}
