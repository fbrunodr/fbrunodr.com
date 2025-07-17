use actix_web::{get, post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use regex::Regex;

use crate::components::navbar::navbar;

#[derive(Deserialize)]
struct WordleRequest {
    green_letters: String,
    grey_letters: String,
    yellow_letters: String,
}

#[derive(Serialize)]
struct WordleResponse {
    success: bool,
    message: String,
    suggestions: Option<Vec<String>>,
}

// Validate inputs
fn validate_wordle_input(green: &str, grey: &str, yellow: &str) -> Result<(), String> {
    // Green letters must be exactly 5 characters
    if green.len() != 5 {
        return Err("Green letters must be exactly 5 characters (use ? for unknown positions)".to_string());
    }

    // Green letters should only contain letters and ?
    let green_regex = Regex::new(r"^[a-z?]{5}$").unwrap();
    if !green_regex.is_match(green) {
        return Err("Green letters can only contain letters and ? characters".to_string());
    }

    // Grey and yellow letters should only contain letters
    let letter_regex = Regex::new(r"^[a-z]*$").unwrap();
    if !letter_regex.is_match(grey) {
        return Err("Grey letters can only contain letters".to_string());
    }
    if !letter_regex.is_match(yellow) {
        return Err("Yellow letters can only contain letters".to_string());
    }

    Ok(())
}

#[get("/wordle-solver")]
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
            </head>
            <body>
                {}

                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/wordle_solver.css\">

                <div class=\"content\">
                    <div class=\"intro\">
                        <h2 class=\"title\">🟩 Wordle <span style=\"color: #5dfa5c;\">Solver</span> 🟨</h2>
                        <p>
                            Enter your Wordle clues below to get word suggestions. Use the green letters for known positions,
                            grey letters for letters not in the word, and yellow letters for letters in the word but wrong position.
                        </p>
                    </div>

                    <div class=\"solver-form\">
                        <div class=\"form-container\">
                            <div class=\"input-group\">
                                <label for=\"green-input\" class=\"form-label\">
                                    🟩 Green Letters (Known positions)
                                </label>
                                <input
                                    type=\"text\"
                                    id=\"green-input\"
                                    class=\"wordle-input green-input\"
                                    placeholder=\"Enter like: ba??n\"
                                    maxlength=\"5\"
                                    pattern=\"[a-zA-Z?]{{5}}\"
                                    title=\"Exactly 5 characters: letters or ? for unknown positions\"
                                >
                                <small class=\"input-help\">Use ? for unknown positions (exactly 5 characters)</small>
                            </div>

                            <div class=\"input-group\">
                                <label for=\"grey-input\" class=\"form-label\">
                                    ⬜ Grey Letters (Not in word)
                                </label>
                                <input
                                    type=\"text\"
                                    id=\"grey-input\"
                                    class=\"wordle-input grey-input\"
                                    placeholder=\"Enter letters not in the word\"
                                    pattern=\"[a-zA-Z]*\"
                                    title=\"Only letters allowed\"
                                >
                                <small class=\"input-help\">Letters that are not in the target word</small>
                            </div>

                            <div class=\"input-group\">
                                <label for=\"yellow-input\" class=\"form-label\">
                                    🟨 Yellow Letters (Wrong position)
                                </label>
                                <input
                                    type=\"text\"
                                    id=\"yellow-input\"
                                    class=\"wordle-input yellow-input\"
                                    placeholder=\"Enter letters in word but wrong position\"
                                    pattern=\"[a-zA-Z]*\"
                                    title=\"Only letters allowed\"
                                >
                                <small class=\"input-help\">Letters in the word but in wrong positions</small>
                            </div>

                            <button id=\"solve-btn\" class=\"solve-button\">
                                <span class=\"button-text\">🔍 Find Words</span>
                                <span class=\"loading-spinner\" style=\"display: none;\">
                                    Solving puzzle...
                                    <span class=\"loading-dots\"></span>
                                </span>
                            </button>
                        </div>
                    </div>

                    <div id=\"result-container\" class=\"result-container\" style=\"display: none;\">
                        <div id=\"result-content\" class=\"result-content\">
                        </div>
                    </div>

                    <div class=\"info-section\">
                        <h3 class=\"info-title\">How to use</h3>
                        <div class=\"instructions\">
                            <div class=\"instruction-item\">
                                <span class=\"instruction-icon\">🟩</span>
                                <div>
                                    <strong>Green Letters:</strong> Enter the exact pattern with known letters in correct positions.
                                    Use ? for unknown positions. Example: \"ba??n\" means B is first, A is second, N is last.
                                </div>
                            </div>
                            <div class=\"instruction-item\">
                                <span class=\"instruction-icon\">⬜</span>
                                <div>
                                    <strong>Grey Letters:</strong> Enter all letters that you know are NOT in the target word.
                                    Example: \"qwerty\" means none of these letters appear in the solution.
                                </div>
                            </div>
                            <div class=\"instruction-item\">
                                <span class=\"instruction-icon\">🟨</span>
                                <div>
                                    <strong>Yellow Letters:</strong> Enter letters that are in the word but in wrong positions.
                                    The solver will find words containing these letters in different positions.
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <script type=\"text/javascript\" src=\"/static/js/wordle_solver.js\"></script>
            </body>
        </html>
    ", navbar());

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content))
}

#[post("/api/wordle-solve")]
pub async fn solve_wordle(data: web::Json<WordleRequest>) -> Result<HttpResponse> {
    let green: String = data.green_letters.trim().to_lowercase();
    let grey = data.grey_letters.trim().to_lowercase();
    let yellow = data.yellow_letters.trim().to_lowercase();

    // Validate inputs
    if let Err(validation_error) = validate_wordle_input(&green, &grey, &yellow) {
        return Ok(HttpResponse::BadRequest().json(WordleResponse {
            success: false,
            message: validation_error,
            suggestions: None,
        }));
    }

    // Transform ? to _ for the C++ binary
    let green_transformed = green.replace('?', "_");

    // Call the C++ binary
    let output = Command::new("./bin/wordle")
        .arg(&green_transformed)
        .arg(&grey)
        .arg(&yellow)
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            // If there's stderr, it's likely an error
            if !stderr.is_empty() {
                return Ok(HttpResponse::Ok().json(WordleResponse {
                    success: false,
                    message: format!("Solver error: {}", stderr),
                    suggestions: None,
                }));
            }

            // Parse the output - assuming each word is on a new line
            let suggestions: Vec<String> = stdout
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_uppercase())
                .collect();

            if suggestions.is_empty() {
                return Ok(HttpResponse::Ok().json(WordleResponse {
                    success: false,
                    message: "No words found matching your criteria. Try adjusting your clues.".to_string(),
                    suggestions: None,
                }));
            }

            Ok(HttpResponse::Ok().json(WordleResponse {
                success: true,
                message: format!("Found {} possible word{}", suggestions.len(), if suggestions.len() == 1 { "" } else { "s" }),
                suggestions: Some(suggestions),
            }))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(WordleResponse {
                success: false,
                message: format!("Failed to execute Wordle solver: {}. Make sure the wordle binary is available in the current directory.", e),
                suggestions: None,
            }))
        }
    }
} 