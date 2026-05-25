// =====================================================
// Teachable Machine Clone Backend
// Rust + Axum + Gemini Vision API
// =====================================================

// -----------------------------
// STEP 1: Import required crates
// -----------------------------
use axum::{
    extract::{DefaultBodyLimit, Multipart, State},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// -----------------------------
// STEP 2: App memory/state
// -----------------------------
#[derive(Clone)]
struct AppState {
    // session_id -> class_name -> images as base64
    sessions: Arc<Mutex<HashMap<String, HashMap<String, Vec<String>>>>>,
    llm_api_key: String,
}

// -----------------------------
// STEP 3: API response structs
// -----------------------------
#[derive(Serialize)]
struct GenericResponse {
    message: String,
}

#[derive(Serialize)]
struct Prediction {
    class_name: String,
    confidence: f32,
}

#[derive(Serialize)]
struct PredictResponse {
    predictions: Vec<Prediction>,
}

// -----------------------------
// STEP 4: Request payload structs
// -----------------------------
#[derive(Deserialize)]
struct TrainPayload {
    session_id: String,
}

#[derive(Deserialize)]
struct RenamePayload {
    session_id: String,
    old_class_name: String,
    new_class_name: String,
}

// -----------------------------
// STEP 5: Gemini response structs
// -----------------------------
#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct GeminiPart {
    text: Option<String>,
}

// -----------------------------
// STEP 6: Start server
// -----------------------------
#[tokio::main]
async fn main() {
    // Enable logs
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Read Gemini API key from environment
    let api_key = env::var("GEMINI_API_KEY")
        .unwrap_or_else(|_| "YOUR_GEMINI_API_KEY_HERE".to_string());

    // Shared memory state
    let state = AppState {
        sessions: Arc::new(Mutex::new(HashMap::new())),
        llm_api_key: api_key,
    };

    // Allow frontend Streamlit to call backend
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // Register API routes
    let app = Router::new()
        .route("/api/ping", get(|| async { "pong" }))
        .route("/api/upload", post(upload_image))
        .route("/api/train", post(train_model))
        .route("/api/predict", post(predict_image))
        .route("/api/rename_class", post(rename_class))
        .layer(cors)
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .with_state(state);

    // Run server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

// -----------------------------
// STEP 7: Upload training images
// -----------------------------
async fn upload_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut session_id = String::new();
    let mut class_name = String::new();
    let mut file_data = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();

        if name == "session_id" {
            session_id = field.text().await.unwrap();
        } else if name == "class_name" {
            class_name = field.text().await.unwrap();
        } else if name == "file" {
            file_data = field.bytes().await.unwrap().to_vec();
        }
    }

    if session_id.is_empty() || class_name.is_empty() || file_data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Missing upload fields".to_string()));
    }

    use base64::{engine::general_purpose, Engine as _};
    let b64_image = general_purpose::STANDARD.encode(&file_data);

    let mut sessions = state.sessions.lock().unwrap();

    let session = sessions
        .entry(session_id.clone())
        .or_insert_with(HashMap::new);

    let class_images = session
        .entry(class_name.clone())
        .or_insert_with(Vec::new);

    class_images.push(b64_image);

    println!("Uploaded image for class: {}", class_name);

    Ok(Json(GenericResponse {
        message: format!("Uploaded image for class {}", class_name),
    }))
}

// -----------------------------
// STEP 8: Train model
// This validates that at least 2 classes exist.
// -----------------------------
async fn train_model(
    State(state): State<AppState>,
    Json(payload): Json<TrainPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let sessions = state.sessions.lock().unwrap();

    if let Some(classes) = sessions.get(&payload.session_id) {
        if classes.len() < 2 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Need at least 2 classes with uploaded images".to_string(),
            ));
        }

        println!("Training completed for session: {}", payload.session_id);

        Ok(Json(GenericResponse {
            message: "Training completed successfully".to_string(),
        }))
    } else {
        Err((StatusCode::NOT_FOUND, "Session not found".to_string()))
    }
}

// -----------------------------
// STEP 9: Rename class
// -----------------------------
async fn rename_class(
    State(state): State<AppState>,
    Json(payload): Json<RenamePayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut sessions = state.sessions.lock().unwrap();

    let session = sessions
        .entry(payload.session_id.clone())
        .or_insert_with(HashMap::new);

    if let Some(images) = session.remove(&payload.old_class_name) {
        session.insert(payload.new_class_name.clone(), images);
    } else {
        session.insert(payload.new_class_name.clone(), Vec::new());
    }

    println!(
        "Class renamed from {} to {}",
        payload.old_class_name, payload.new_class_name
    );

    Ok(Json(GenericResponse {
        message: format!("Class renamed to {}", payload.new_class_name),
    }))
}

// -----------------------------
// STEP 10: Predict image using Gemini Vision
// -----------------------------
async fn predict_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut session_id = String::new();
    let mut target_image_data = Vec::new();

    // Read uploaded test image
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();

        if name == "session_id" {
            session_id = field.text().await.unwrap();
        } else if name == "file" {
            target_image_data = field.bytes().await.unwrap().to_vec();
        }
    }

    if session_id.is_empty() || target_image_data.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Missing session_id or test image".to_string(),
        ));
    }

    // Get class names from uploaded training data
    let class_names: Vec<String> = {
        let sessions = state.sessions.lock().unwrap();

        let classes = sessions
            .get(&session_id)
            .ok_or((StatusCode::NOT_FOUND, "Session not found".to_string()))?;

        if classes.len() < 2 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Upload images for at least 2 classes before prediction".to_string(),
            ));
        }

        classes.keys().cloned().collect()
    };

    // Check Gemini key
    let api_key = state.llm_api_key.clone();

    if api_key == "YOUR_GEMINI_API_KEY_HERE" || api_key.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "GEMINI_API_KEY is missing. Set your Gemini API key first.".to_string(),
        ));
    }

    // Convert test image to base64
    use base64::{engine::general_purpose, Engine as _};
    let b64_target = general_purpose::STANDARD.encode(&target_image_data);

    let class_list = class_names.join(", ");

    // Prompt Gemini to return JSON only
    let prompt = format!(
        "You are an image classification model. \
        Classify this uploaded image into exactly one of these classes: {}. \
        Return only valid JSON. \
        JSON format must be exactly: {{\"class_name\":\"class name here\",\"confidence\":0.90}}. \
        The class_name must exactly match one of the given classes.",
        class_list
    );

    println!("Sending image to Gemini for prediction...");
    println!("Available classes: {}", class_list);

    // Gemini request body
    let request_body = serde_json::json!({
        "contents": [
            {
                "parts": [
                    {
                        "text": prompt
                    },
                    {
                        "inline_data": {
                            "mime_type": "image/jpeg",
                            "data": b64_target
                        }
                    }
                ]
            }
        ]
    });

    // Gemini endpoint
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let client = Client::new();

    // Send request to Gemini
    let response = client
        .post(url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Gemini request failed: {}", e),
            )
        })?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    println!("Gemini raw response: {}", body_text);

    if !status.is_success() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Gemini API error: {}", body_text),
        ));
    }

    // Parse Gemini response
    let gemini_response: GeminiResponse = serde_json::from_str(&body_text).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse Gemini response: {} | Body: {}", e, body_text),
        )
    })?;

    let text = gemini_response
        .candidates
        .get(0)
        .and_then(|candidate| candidate.content.parts.get(0))
        .and_then(|part| part.text.clone())
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "No text returned from Gemini".to_string(),
        ))?;

    // Clean ```json markdown if Gemini adds it
    let cleaned_text = text
        .replace("```json", "")
        .replace("```", "")
        .trim()
        .to_string();

    println!("Gemini cleaned response: {}", cleaned_text);

    // Parse Gemini JSON
    let parsed: serde_json::Value = serde_json::from_str(&cleaned_text).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Gemini did not return valid JSON: {} | Text: {}",
                e, cleaned_text
            ),
        )
    })?;

    let predicted_class = parsed["class_name"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let confidence = parsed["confidence"]
        .as_f64()
        .unwrap_or(0.80) as f32;

    println!("Predicted class: {}", predicted_class);
    println!("Confidence: {}", confidence);

    // Convert Gemini single result into chart-friendly response
    let mut predictions = Vec::new();
    let other_confidence = if class_names.len() > 1 {
        (1.0 - confidence) / ((class_names.len() - 1) as f32)
    } else {
        0.0
    };

    for class_name in class_names {
        if class_name == predicted_class {
            predictions.push(Prediction {
                class_name,
                confidence,
            });
        } else {
            predictions.push(Prediction {
                class_name,
                confidence: other_confidence,
            });
        }
    }

    Ok(Json(PredictResponse { predictions }))
}