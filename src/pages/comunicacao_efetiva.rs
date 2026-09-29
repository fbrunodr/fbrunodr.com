use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use actix_web::{get, HttpRequest, HttpResponse, Result};

const LOG_PATH: &str = "bucket/rick_rolls.txt";
const RICK_ROLL_URL: &str = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";

#[get("/comunicacao_efetiva")]
pub async fn render(req: HttpRequest) -> Result<HttpResponse> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .replace(['\t', '\n', '\r'], " ");

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(LOG_PATH) {
        let _ = writeln!(file, "{}\t{}\t{}", timestamp, ip, user_agent);
    }

    // Serve an HTML page instead of a 302 so link previews (WhatsApp, Telegram, ...)
    // show a harmless title rather than the YouTube video.
    let html_content = format!("
        <html lang=\"pt-BR\">
            <head>
                <meta charset=\"utf-8\" />
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
                <meta property=\"og:title\" content=\"Comunicação Efetiva\" />
                <meta property=\"og:description\" content=\"Guia prático de comunicação efetiva.\" />
                <meta http-equiv=\"refresh\" content=\"0; url={url}\" />
                <title>Comunicação Efetiva</title>
                <script>window.location.replace(\"{url}\");</script>
            </head>
            <body></body>
        </html>
    ", url = RICK_ROLL_URL);

    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html_content))
}
