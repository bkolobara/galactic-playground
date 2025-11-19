use anyhow::{anyhow, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

use crate::contracts::kale::Kale;

const SERVER_PORT: u16 = 3737;

#[derive(Debug, Deserialize, Serialize)]
pub struct PubkeyResponse {
    pub pubkey: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlantPrepareRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub amount: String,
}

#[derive(Debug, Serialize)]
pub struct PlantPrepareResponse {
    pub xdr: String,
    pub network: String,
}

#[derive(Debug, Deserialize)]
pub struct PlantSubmitRequest {
    #[serde(rename = "signedXdr")]
    pub signed_xdr: String,
}

#[derive(Debug, Serialize)]
pub struct PlantSubmitResponse {
    pub hash: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct CheckPlantedRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

#[derive(Debug, Serialize)]
pub struct CheckPlantedResponse {
    pub has_planted: bool,
}

#[derive(Debug, Serialize)]
pub struct BlockInfoResponse {
    #[serde(rename = "blockIndex")]
    pub block_index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entropy: Option<String>, // hex-encoded, None if nobody has planted yet
}

#[derive(Debug, Deserialize)]
pub struct WorkPrepareRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub nonce: String, // u64 as string
}

#[derive(Debug, Serialize)]
pub struct WorkPrepareResponse {
    pub xdr: String,
    pub network: String,
}

#[derive(Debug, Deserialize)]
pub struct WorkSubmitRequest {
    #[serde(rename = "signedXdr")]
    pub signed_xdr: String,
}

#[derive(Debug, Serialize)]
pub struct WorkSubmitResponse {
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct PailDataRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "blockIndex")]
    pub block_index: u32,
}

#[derive(Debug, Serialize)]
pub struct PailDataResponse {
    #[serde(rename = "hasPail")]
    pub has_pail: bool,
    #[serde(rename = "hasWorked")]
    pub has_worked: bool,
    #[serde(rename = "leadingZeros")]
    pub leading_zeros: u32,
}

#[derive(Debug, Deserialize)]
pub struct HarvestPrepareRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "blockIndex")]
    pub block_index: u32,
}

#[derive(Debug, Serialize)]
pub struct HarvestPrepareResponse {
    pub xdr: String,
    pub network: String,
}

#[derive(Debug, Deserialize)]
pub struct HarvestSubmitRequest {
    #[serde(rename = "signedXdr")]
    pub signed_xdr: String,
}

#[derive(Debug, Serialize)]
pub struct HarvestSubmitResponse {
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountStatusRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

#[derive(Debug, Serialize)]
pub struct AccountStatusResponse {
    pub exists: bool,
    #[serde(rename = "xlmBalance")]
    pub xlm_balance: i64, // in stroops
    #[serde(rename = "hasTrustline")]
    pub has_trustline: bool,
}

#[derive(Debug, Deserialize)]
pub struct FundAccountRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

#[derive(Debug, Serialize)]
pub struct FundAccountResponse {
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct TrustlinePrepareRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

#[derive(Debug, Serialize)]
pub struct TrustlinePrepareResponse {
    pub xdr: String,
    pub network: String,
}

#[derive(Debug, Deserialize)]
pub struct TrustlineSubmitRequest {
    #[serde(rename = "signedXdr")]
    pub signed_xdr: String,
}

#[derive(Debug, Serialize)]
pub struct TrustlineSubmitResponse {
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct AllFarmersRequest {
    #[serde(rename = "blockIndex")]
    pub block_index: u32,
    #[serde(rename = "farmerAddresses")]
    pub farmer_addresses: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FarmerPailInfo {
    #[serde(rename = "farmerAddress")]
    pub farmer_address: String,
    #[serde(rename = "hasPail")]
    pub has_pail: bool,
    #[serde(rename = "hasWorked")]
    pub has_worked: bool,
    #[serde(rename = "leadingZeros")]
    pub leading_zeros: u32,
}

#[derive(Debug, Serialize)]
pub struct AllFarmersResponse {
    pub farmers: Vec<FarmerPailInfo>,
}

/// Represents the state of the Albedo authentication process
#[derive(Clone)]
struct AlbedoState {
    pub_key: Option<String>,
    error: Option<String>,
    completed: bool,
}

/// Shared state for the KALE contract client
struct AppState {
    kale: Kale,
}

/// Initiates Albedo wallet authentication and plant transaction flow
/// Returns the user's public key and transaction hash after successful plant
pub async fn authenticate_and_plant(kale_client: Kale) -> Result<(String, String)> {
    // Create shared state to store the result
    let auth_state = Arc::new(Mutex::new(AlbedoState {
        pub_key: None,
        error: None,
        completed: false,
    }));

    // Build the URL
    let auth_url = format!("http://localhost:{}", SERVER_PORT);

    println!("Please open the following URL in your browser:");
    println!("{}", auth_url);

    // Start the local HTTP server
    let result = start_server(auth_state.clone(), kale_client).await?;

    Ok(result)
}

/// Starts a local HTTP server to serve the frontend and handle responses
async fn start_server(
    auth_state: Arc<Mutex<AlbedoState>>,
    kale_client: Kale,
) -> Result<(String, String)> {
    let auth_state_clone = auth_state.clone();
    let app_state = Arc::new(AppState { kale: kale_client });

    // Create the router
    let app = Router::new()
        .route("/", get(serve_landing))
        .route("/app/kale", get(serve_kale))
        .route("/api/pubkey", post(handle_pubkey))
        .route("/api/plant/prepare", post(handle_plant_prepare))
        .route("/api/plant/submit", post(handle_plant_submit))
        .route("/api/check_planted", post(handle_check_planted))
        .route("/api/block_info", get(handle_block_info))
        .route("/api/work/prepare", post(handle_work_prepare))
        .route("/api/work/submit", post(handle_work_submit))
        .route("/api/pail_data", post(handle_pail_data))
        .route("/api/harvest/prepare", post(handle_harvest_prepare))
        .route("/api/harvest/submit", post(handle_harvest_submit))
        .route("/api/account_status", post(handle_account_status))
        .route("/api/fund_account", post(handle_fund_account))
        .route("/api/trustline/prepare", post(handle_trustline_prepare))
        .route("/api/trustline/submit", post(handle_trustline_submit))
        .route("/api/all_farmers", post(handle_all_farmers))
        .with_state((auth_state_clone, app_state))
        .fallback_service(ServeDir::new("frontend/dist"));

    // Bind to the server port
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", SERVER_PORT)).await?;

    println!("Server listening on http://localhost:{}", SERVER_PORT);

    // Spawn the server in a background task
    let server_handle = tokio::spawn(async move { axum::serve(listener, app).await });

    // Wait for authentication first
    let pub_key = loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let state_guard = auth_state.lock().await;
        if state_guard.completed {
            if let Some(error) = &state_guard.error {
                let error_msg = error.clone();
                drop(state_guard);
                server_handle.abort();
                return Err(anyhow!("Authentication failed: {}", error_msg));
            }
            if let Some(key) = &state_guard.pub_key {
                let result = key.clone();
                drop(state_guard);
                break result;
            }
        }
    };

    println!("\nAuthenticated with public key: {}", pub_key);
    println!("Waiting for plant transaction to complete...");

    // Wait for the transaction to complete (server will keep running)
    // In a real implementation, you'd have another shared state to track the transaction
    // For now, we'll just keep the server running indefinitely
    // The user can manually close when done
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        // Check if we should exit (this is a placeholder)
        // In practice, you might want to add a completion flag
    }
}

/// Serves the landing page
async fn serve_landing() -> impl IntoResponse {
    Html(include_str!("../frontend/public/landing.html"))
}

/// Serves the KALE app HTML page
async fn serve_kale() -> impl IntoResponse {
    Html(include_str!("../frontend/public/index.html"))
}

/// Handles the public key POST request from the frontend
async fn handle_pubkey(
    State((auth_state, _app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<PubkeyResponse>,
) -> impl IntoResponse {
    let mut state_guard = auth_state.lock().await;

    if let Some(pubkey) = payload.pubkey {
        state_guard.pub_key = Some(pubkey);
    }

    if let Some(error) = payload.error {
        state_guard.error = Some(error);
    }

    state_guard.completed = true;

    Json(serde_json::json!({"status": "ok"}))
}

/// Handles the plant transaction preparation request
async fn handle_plant_prepare(
    State((_auth_state, app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<PlantPrepareRequest>,
) -> Result<Json<PlantPrepareResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Parse the amount
    let amount: i128 = payload
        .amount
        .parse()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid amount format".to_string(),
                }),
            )
        })?;

    // Prepare the transaction
    let tx_xdr = app_state
        .kale
        .prepare_plant_transaction(&payload.public_key, amount)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to prepare transaction: {}", e),
                }),
            )
        })?;

    // Return the full network passphrase (Albedo requires the full passphrase)
    let network = app_state.kale.network_passphrase();

    Ok(Json(PlantPrepareResponse {
        xdr: tx_xdr,
        network: network.to_string(),
    }))
}

/// Handles the plant transaction submission request
async fn handle_plant_submit(
    State((_auth_state, app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<PlantSubmitRequest>,
) -> Result<Json<PlantSubmitResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Submit the signed transaction
    let tx_hash = app_state
        .kale
        .submit_plant_transaction(&payload.signed_xdr)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to submit transaction: {}", e),
                }),
            )
        })?;

    println!("\n✓ Transaction submitted successfully!");
    println!("Transaction hash: {}", tx_hash);

    Ok(Json(PlantSubmitResponse { hash: tx_hash }))
}

/// Handles checking if the farmer has planted in the current block
async fn handle_check_planted(
    State((_auth_state, app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<CheckPlantedRequest>,
) -> Result<Json<CheckPlantedResponse>, (StatusCode, Json<ErrorResponse>)> {
    let has_planted = app_state
        .kale
        .has_planted(&payload.public_key)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to check planted status: {}", e),
                }),
            )
        })?;

    Ok(Json(CheckPlantedResponse { has_planted }))
}

/// Handles getting the current block information
async fn handle_block_info(
    State((_auth_state, app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
) -> Result<Json<BlockInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (block_index, entropy) = app_state
        .kale
        .get_block_info()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to get block info: {}", e),
                }),
            )
        })?;

    Ok(Json(BlockInfoResponse {
        block_index,
        entropy: entropy.map(|e| hex::encode(e)),
    }))
}

/// Handles the work transaction preparation request
async fn handle_work_prepare(
    State((_auth_state, app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<WorkPrepareRequest>,
) -> Result<Json<WorkPrepareResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Parse the nonce
    let nonce: u64 = payload.nonce.parse().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid nonce format".to_string(),
            }),
        )
    })?;

    // Prepare the transaction (hash will be calculated in the backend)
    let tx_xdr = app_state
        .kale
        .prepare_work_transaction(&payload.public_key, nonce)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to prepare transaction: {}", e),
                }),
            )
        })?;

    let network = app_state.kale.network_passphrase();

    Ok(Json(WorkPrepareResponse {
        xdr: tx_xdr,
        network: network.to_string(),
    }))
}

/// Handles the work transaction submission request
async fn handle_work_submit(
    State((_auth_state, app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<WorkSubmitRequest>,
) -> Result<Json<WorkSubmitResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Submit the signed transaction
    let tx_hash = app_state
        .kale
        .submit_work_transaction(&payload.signed_xdr)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to submit transaction: {}", e),
                }),
            )
        })?;

    println!("\n✓ Work transaction submitted successfully!");
    println!("Transaction hash: {}", tx_hash);

    Ok(Json(WorkSubmitResponse { hash: tx_hash }))
}

/// Handles getting Pail data for a farmer in a specific block
async fn handle_pail_data(
    State((_auth_state, app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<PailDataRequest>,
) -> Result<Json<PailDataResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (has_pail, has_worked, leading_zeros) = app_state
        .kale
        .get_pail_data(&payload.public_key, payload.block_index)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to get pail data: {}", e),
                }),
            )
        })?;

    Ok(Json(PailDataResponse {
        has_pail,
        has_worked,
        leading_zeros,
    }))
}

/// Handles the harvest transaction preparation request
async fn handle_harvest_prepare(
    State((_auth_state, app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<HarvestPrepareRequest>,
) -> Result<Json<HarvestPrepareResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Prepare the transaction
    let tx_xdr = app_state
        .kale
        .prepare_harvest_transaction(&payload.public_key, payload.block_index)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to prepare transaction: {}", e),
                }),
            )
        })?;

    let network = app_state.kale.network_passphrase();

    Ok(Json(HarvestPrepareResponse {
        xdr: tx_xdr,
        network: network.to_string(),
    }))
}

/// Handles the harvest transaction submission request
async fn handle_harvest_submit(
    State((_auth_state, app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<HarvestSubmitRequest>,
) -> Result<Json<HarvestSubmitResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Submit the signed transaction
    let tx_hash = app_state
        .kale
        .submit_harvest_transaction(&payload.signed_xdr)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to submit transaction: {}", e),
                }),
            )
        })?;

    println!("\n✓ Harvest transaction submitted successfully!");
    println!("Transaction hash: {}", tx_hash);

    Ok(Json(HarvestSubmitResponse { hash: tx_hash }))
}

/// Handles checking account status (balance and trustline)
async fn handle_account_status(
    State((_auth_state, app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<AccountStatusRequest>,
) -> Result<Json<AccountStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check XLM balance
    let xlm_balance = app_state
        .kale
        .get_xlm_balance(&payload.public_key)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to check balance: {}", e),
                }),
            )
        })?;

    let (exists, balance) = match xlm_balance {
        Some(bal) => (true, bal),
        None => (false, 0),
    };

    // Check KALE trustline
    let (has_trustline, _) = app_state
        .kale
        .check_kale_trustline(&payload.public_key)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to check trustline: {}", e),
                }),
            )
        })?;

    Ok(Json(AccountStatusResponse {
        exists,
        xlm_balance: balance,
        has_trustline,
    }))
}

/// Handles funding an account via friendbot
async fn handle_fund_account(
    State((_auth_state, _app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<FundAccountRequest>,
) -> Result<Json<FundAccountResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Call friendbot
    let friendbot_url = format!(
        "https://friendbot.stellar.org?addr={}",
        payload.public_key
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&friendbot_url)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to call friendbot: {}", e),
                }),
            )
        })?;

    if response.status().is_success() {
        println!("\n✓ Account funded successfully via friendbot!");
        Ok(Json(FundAccountResponse { success: true }))
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Friendbot request failed: {}", error_text),
            }),
        ))
    }
}

/// Handles preparing a trustline transaction
async fn handle_trustline_prepare(
    State((_auth_state, app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<TrustlinePrepareRequest>,
) -> Result<Json<TrustlinePrepareResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Prepare the trustline transaction
    let tx_xdr = app_state
        .kale
        .prepare_add_kale_trustline_transaction(&payload.public_key)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to prepare trustline transaction: {}", e),
                }),
            )
        })?;

    let network = app_state.kale.network_passphrase();

    Ok(Json(TrustlinePrepareResponse {
        xdr: tx_xdr,
        network: network.to_string(),
    }))
}

/// Handles submitting a trustline transaction
async fn handle_trustline_submit(
    State((_auth_state, app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<TrustlineSubmitRequest>,
) -> Result<Json<TrustlineSubmitResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Submit the signed transaction
    let tx_hash = app_state
        .kale
        .submit_trustline_transaction(&payload.signed_xdr)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to submit trustline transaction: {}", e),
                }),
            )
        })?;

    println!("\n✓ Trustline transaction submitted successfully!");
    println!("Transaction hash: {}", tx_hash);

    Ok(Json(TrustlineSubmitResponse { hash: tx_hash }))
}

/// Handles getting pail data for a list of farmers in a specific block
async fn handle_all_farmers(
    State((_auth_state, app_state)): State<(Arc<Mutex<AlbedoState>>, Arc<AppState>)>,
    Json(payload): Json<AllFarmersRequest>,
) -> Result<Json<AllFarmersResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Fetch pail data for each farmer address provided
    let mut farmers_info = Vec::new();

    for farmer_address in payload.farmer_addresses {
        match app_state
            .kale
            .get_pail_data(&farmer_address, payload.block_index)
            .await
        {
            Ok((has_pail, has_worked, leading_zeros)) => {
                // Only include farmers who actually planted
                if has_pail {
                    farmers_info.push(FarmerPailInfo {
                        farmer_address,
                        has_pail,
                        has_worked,
                        leading_zeros,
                    });
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to get pail data for farmer {}: {}",
                    farmer_address, e
                );
                // Continue with other farmers even if one fails
            }
        }
    }

    Ok(Json(AllFarmersResponse {
        farmers: farmers_info,
    }))
}
