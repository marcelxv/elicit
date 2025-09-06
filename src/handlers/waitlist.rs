use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use tracing::{error, info};

#[derive(Deserialize)]
pub struct WaitlistRequest {
    pub email: String,
}

#[derive(Serialize)]
pub struct WaitlistResponse {
    pub success: bool,
    pub message: String,
}

/// Handler for waitlist email collection
pub async fn waitlist_handler(
    Json(payload): Json<WaitlistRequest>,
) -> Result<ResponseJson<WaitlistResponse>, StatusCode> {
    // Basic email validation
    if !is_valid_email(&payload.email) {
        return Ok(ResponseJson(WaitlistResponse {
            success: false,
            message: "Invalid email address".to_string(),
        }));
    }

    // Store email in a simple file (for production, use a proper database)
    match store_email(&payload.email).await {
        Ok(_) => {
            info!("New waitlist signup: {}", payload.email);
            Ok(ResponseJson(WaitlistResponse {
                success: true,
                message: "Successfully added to waitlist".to_string(),
            }))
        }
        Err(e) => {
            error!("Failed to store email {}: {}", payload.email, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Basic email validation
fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

/// Store email in a simple text file
async fn store_email(email: &str) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("waitlist.txt")?;
    
    writeln!(file, "{}", email)?;
    Ok(())
}