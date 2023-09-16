use actix_web::{get, HttpResponse, Result};

use crate::components::navbar::navbar;

#[get("/steganography")]
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
                <script type=\"text/javascript\" src=\"/static/js/colors.js\"></script>
            <head/>
            <body>
                {}
                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/steganography.css\">
                <div class=\"content\">
                    Hide a message in your file below :) .
                    <label for=\"input-file\" id=\"drop-area\">
                        <input type=\"file\" accept=\"image/*\" id=\"input-file\" hidden>
                        <div id=\"img-view\">
                            <img src=\"/static/assets/images/upload_icon.png\">
                            <p> Drag and drop or click here<br>to upload image</p>
                            <span>Upload any images from desktop</span>
                        </div>
                    </label>
                </div>
            </body>
        </html>
    ", navbar());
 
   Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content)) 
}
