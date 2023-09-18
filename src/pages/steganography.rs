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
                <script src=\"https://cdnjs.cloudflare.com/ajax/libs/crypto-js/4.1.1/crypto-js.min.js\"></script> 
            <head/>
            <body>
                {}
                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/steganography.css\">

                <div class=\"content\">

                    <div class=\"intro\">
                        <h2 class=\"title\"> What is <span>Steganography</span>? </h2>
                        <p>
                            Steganography is the practice of concealing one piece of information within another to hide
                            its existence. It typically involves embedding secret data, such as text or images, within
                            a seemingly innocuous carrier medium, like an image or audio file, in a way that is
                            difficult to detect without specific knowledge or tools. Steganography is often used for
                            covert communication or data protection, where the goal is to hide the presence of the
                            hidden information rather than encrypting it.
                        </p>
                        <br>
                        <p>
                            Because we really really care about security here, this web page does both. Given a image,
                            a message and a password we encrypt the message according to the password and hide the
                            encrypted message inside the image. To read the hidden message inside the image, just
                            upload the image that contains the hidden message and provide the password.
                        </p>
                    </div>

                    <label for=\"input-file\" id=\"drop-area\">
                        <input type=\"file\" accept=\"image/*\" id=\"input-file\" hidden>
                        <div id=\"img-view\">
                            <img src=\"/static/assets/images/upload_icon.png\">
                            <p> Drag and drop or click here<br>to upload image</p>
                            <span>Upload any images from desktop</span>
                        </div>
                    </label>

                    <div class=\"encrypt-decrypt\">
                        <button id=\"encrypt-decrypt-button\">Encrypt</button>
                    </div>

                <script type=\"text/javascript\" src=\"/static/js/steganography.js\"></script>
                </div>
            </body>
        </html>
    ", navbar());
 
   Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content)) 
}
