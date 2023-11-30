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
                <canvas id=\"background\">
                </canvas>

               <script type=\"text/javascript\" src=\"/static/js/matrix_background.js\"></script>

                {}

                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/steganography.css\">

                <div class=\"content\">

                    <div class=\"intro\">
                        <h2 class=\"title\"> What is <a href=\"https://en.wikipedia.org/wiki/Steganography\"><span style=\"color: #5dfa5c;\">Steganography</span></a>? </h2>
                        <p>
                            Steganography is the practice of concealing one piece of information within another to hide
                            its existence. It typically involves embedding secret data, such as text or images, within
                            a seemingly innocuous carrier medium, like an image or audio file, in a way that is
                            difficult to detect without specific knowledge or tools. Steganography is often used for
                            covert communication or data protection, where the goal is to hide the presence of the
                            hidden information rather than encrypting it.
                        </p>
                        <h3 class =\"smallTitle\"> How this page work? </h3>
                        <p>
                            Because we really really care about security here, this web page does both steganography
                            and encryption. Given a image, a message and a password we encrypt the message according
                            to the password and hide the encrypted message inside the image. To read the hidden
                            message inside the image, just upload the image that contains the hidden message and
                            provide the password used to encrypt it.
                        </p>
                        <br/>
                        <p>
                            The encryption algorithm we use is the <a href=\"https://en.wikipedia.org/wiki/Advanced_Encryption_Standard\">
                            <span style=\"color:#5ce0ff;\">Advanced Encryption Standard (AES)</span></a>. After
                            encryption, we then break the encrypted message into its bits (yes, bits not bytes). Then
                            we hide each bit in the least significant bit of the color channels of the image. For
                            example, if the first bit is 0, we turn off the last bit of the top-left red pixel. If
                            its value is 127, it becomes 126. If it is 52, it remains as 52 (least significant bit is
                            already off). This way the image remains unchanged to the naked eye. <a href=\"https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/putImageData#data_loss_due_to_browser_optimization\">
                            <span style=\"color: #c34412;\"> Due to the lossy nature of converting to and from premultiplied
                            alpha color values </span> </a> we only use pixels with alpha value equal to 255, as
                            <a href=\"/static/js/minimumAlpha.js\"> <span style=\"color: #c06bff\"> all other values may lead to loss of data</span></a>.
                            Of course we could send your image, message and password to the site's server, do all the
                            processing outside of the web browser (where we wouldn't have to deal with this bs) and
                            then return your desired image back. We don't do that because of the following:
                        </p>
                        <ol>
                            <br/>
                            <li> By doing everything client-side you don't have to trust me that I am not saving your
                            message, password (which you probably use elsewhere, dumbass) and image in my server. </li>
                            <br/>
                            <li> I don't want to waste my server storage, processing time and bandwith with your data,
                            sorry. </li>
                        </ol>
                        <br/>
                        <p>
                            If you don't care about encryption and just wanna hide the message inside the image file
                            just leave the password field empty. Enjoy this code I provided you:
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

                    <div id=\"encrypt-decrypt-switch\">
                        <div id=\"encrypt-switch\" class=\"switch\" onClick=\"set_encrypt_mode();\">Encrypt mode</div>
                        <div id=\"decrypt-switch\" class=\"switch\" onClick=\"set_decrypt_mode();\">Decrypt mode</div>
                    </div>

                    <div id=\"encrypt-decrypt\">
                        Something
                    </div>

                    <script type=\"text/javascript\" src=\"/static/js/steganography.js\"></script>
                    <script type=\"text/javascript\">
                        set_encrypt_mode()
                    </script>

                    <div id=\"response\">
                    </div>

                </div>
            </body>
        </html>
    ", navbar());

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content)) 
}
