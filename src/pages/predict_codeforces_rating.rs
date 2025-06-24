use actix_web::{get, post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use regex::Regex;

use crate::components::navbar::navbar;

#[derive(Deserialize)]
struct HandleRequest {
    handle: String,
}

#[derive(Serialize)]
struct PredictionResponse {
    success: bool,
    message: String,
    current_rating: Option<i32>,
    predicted_rating: Option<i32>,
    rating_change: Option<i32>,
    motivational_message: Option<String>,
}

// Validate Codeforces handle format
fn validate_handle(handle: &str) -> Result<(), String> {
    // Codeforces handles: 3-24 characters, alphanumeric, underscore, hyphen, dot
    let handle_regex = Regex::new(r"^[a-zA-Z0-9_\-\.]{3,24}$").unwrap();

    if handle.is_empty() {
        return Err("Handle cannot be empty".to_string());
    }

    if handle.len() > 24 {
        return Err("Handle too long (maximum 24 characters)".to_string());
    }

    if !handle_regex.is_match(handle) {
        return Err("Invalid handle format. Use only letters, numbers, underscore, hyphen, and dot (3-24 characters)".to_string());
    }

    // Additional checks for potentially dangerous patterns
    let dangerous_patterns = [
        ";", "&&", "||", "|", ">", "<", "`", "$", "(", ")", "{", "}", "[", "]",
        "rm", "cat", "nc", "wget", "curl", "bash", "sh", "python", "perl"
    ];

    let handle_lower = handle.to_lowercase();
    for pattern in &dangerous_patterns {
        if handle_lower.contains(pattern) {
            return Err("Handle contains invalid characters".to_string());
        }
    }

    Ok(())
}

#[get("/predict-codeforces-rating")]
pub async fn render() -> Result<HttpResponse> {
    let html_content = format!("
        <html lang=\"en\">
            <head>
                <meta charset=\"utf-8\" />
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/index.css\">
                <link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">
                <link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>
                <link href=\"https://fonts.googleapis.com/css2?family=Open+Sans:wght@300;400&family=Reenie+Beanie&family=Source+Code+Pro&display=swap\" rel=\"stylesheet\">
            <head/>
            <body>
                {}

                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/predict_rating.css\">

                <div class=\"content\">
                    <div class=\"intro\">
                        <h2 class=\"title\">Code<span style=\"color: #ff3333;\">forces</span> Rating <span style=\"color: #5dfa5c;\">Predictor</span></h2>
                        <p>
                            Enter your Codeforces' handle below to get a prediction of your rating in 6 months.
                            This prediction is based on your current rating and historical data patterns.
                        </p>
                    </div>

                    <div class=\"prediction-form\">
                        <div class=\"form-container\">
                            <label for=\"handle-input\" class=\"form-label\">Codeforces' Handle:</label>
                            <input
                                type=\"text\"
                                id=\"handle-input\"
                                class=\"handle-input\"
                                placeholder=\"Enter your handle (e.g., tourist)\"
                                maxlength=\"24\"
                                pattern=\"[a-zA-Z0-9_\\-\\.]{{3,24}}\"
                                title=\"3-24 characters: letters, numbers, underscore, hyphen, dot\"
                            >
                            <button id=\"predict-btn\" class=\"predict-button\">
                                <span class=\"button-text\">Predict Rating</span>
                                <span class=\"loading-spinner\" style=\"display: none;\">Fetching data from Codeforces... ⏳</span>
                            </button>
                        </div>
                    </div>

                    <div id=\"result-container\" class=\"result-container\" style=\"display: none;\">
                        <div id=\"result-content\" class=\"result-content\">
                        </div>
                    </div>

                    <div class=\"info-section\">
                        <h3 class=\"info-title\">How it works</h3>
                        <p>
                            This predictor analyzes your Codeforces' submissions history, problem-solving patterns,
                            and rating progression to forecast your future performance. The prediction takes into account:
                            </p>
                        <ul>
                            <li>Your current rating and recent performance</li>
                            <li>Contest participation frequency</li>
                            <li>Solved problems and their difficulty</li>
                            <li>Rating volatility and growth patterns</li>
                        </ul>
                        <p class=\"disclaimer\">
                            <strong>Note:</strong> This is a prediction based on statistical analysis and should not be
                            considered as a guarantee. Your actual performance depends on your continued practice and improvement.
                        </p>
                    </div>

                    <div class=\"github-link\">
                        <p>
                            Code used to get the data, generate labels and train the machine learning model can be found at:
                            <a href=\"https://github.com/fbrunodr/CFRatingPredictor\" target=\"_blank\" rel=\"noopener noreferrer\">
                                https://github.com/fbrunodr/CFRatingPredictor
                            </a>
                        </p>
                    </div>
                </div>

                <script type=\"text/javascript\" src=\"/static/js/predict_rating.js\"></script>
            </body>
        </html>
    ", navbar());

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content))
}

#[post("/api/predict-codeforces-rating")]
pub async fn predict_rating(data: web::Json<HandleRequest>) -> Result<HttpResponse> {
    let handle = &data.handle.trim();

    // Validate handle before processing
    if let Err(validation_error) = validate_handle(handle) {
        return Ok(HttpResponse::BadRequest().json(PredictionResponse {
            success: false,
            message: validation_error.to_string(),
            current_rating: None,
            predicted_rating: None,
            rating_change: None,
            motivational_message: None,
        }));
    }

    // Call the Python script using the virtual environment
    let output = Command::new("../CFRatingPredictor/.venv/bin/python")
        .arg("../CFRatingPredictor/getUserRating.py")
        .arg(handle)
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            // If there's stderr, it's likely an error
            if !stderr.is_empty() {
                return Ok(HttpResponse::Ok().json(PredictionResponse {
                    success: false,
                    message: format!("Error: {}", stderr),
                    current_rating: None,
                    predicted_rating: None,
                    rating_change: None,
                    motivational_message: None,
                }));
            }

            let parts: Vec<&str> = stdout.split_whitespace().collect();

            if parts.len() == 2 {
                if let (Ok(current), Ok(predicted)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                    let change = predicted - current;
                    let (message, motivational) = generate_messages(current, predicted, change);

                    return Ok(HttpResponse::Ok().json(PredictionResponse {
                        success: true,
                        message,
                        current_rating: Some(current),
                        predicted_rating: Some(predicted),
                        rating_change: Some(change),
                        motivational_message: Some(motivational),
                    }));
                }
            }

            // Fallback for any error: wrong number of parts or parse failure
            Ok(HttpResponse::Ok().json(PredictionResponse {
                success: false,
                message: stdout,
                current_rating: None,
                predicted_rating: None,
                rating_change: None,
                motivational_message: None,
            }))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(PredictionResponse {
                success: false,
                message: format!("Failed to execute prediction script: {}", e),
                current_rating: None,
                predicted_rating: None,
                rating_change: None,
                motivational_message: None,
            }))
        }
    }
}

fn generate_messages(current: i32, predicted: i32, change: i32) -> (String, String) {
    let message = format!("Current rating: {} → Predicted in 6 months: {}", current, predicted);

    let motivational = if change > 0 {
        match change {
            1..=50 => "🎉 Great progress! Keep up the consistent practice!",
            51..=100 => "🚀 Excellent improvement! You're on the right track!",
            101..=200 => "🔥 Amazing growth! Your dedication is paying off!",
            201..=300 => "💎 Outstanding progress! You're becoming a force to reckon with!",
            _ => "🌟 Legendary improvement! You're rewriting the rules!",
        }
    } else if change < 0 {
        match change.abs() {
            1..=50 => "💪 Don't worry! Small setbacks are part of the journey. Keep practicing!",
            51..=100 => "🔥 Every challenge makes you stronger! Focus on learning from mistakes.",
            101..=200 => "⚡ This is just a temporary dip! Your potential is limitless.",
            _ => "🌅 Every master was once a beginner. This is your comeback story!",
        }
    } else {
        "🎯 Steady as a rock! Consistency is key to long-term success."
    };

    (message, motivational.to_string())
}
