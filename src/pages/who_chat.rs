use std::fs::File;
use std::io::{Write, Read, self};
use std::path::Path;
use actix_web::{get, post, web, HttpResponse, Responder, Result};
use serde::Deserialize;
use age::secrecy::Secret;
use rand::{distributions::Alphanumeric, Rng};
use crate::components::navbar::navbar;

const SALT_SIZE: usize = 32;
const CONTENT_SIZE_LIMIT: usize = 40;


#[derive(Deserialize)]
struct ChatAccess {
    name: String,
    password: String,
}


#[derive(Deserialize)]
struct ChatPost {
    name: String,
    password: String,
    content: String,
}


fn generate_salt(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}


fn encrypt(content: &String, password: &String, salt: &String) -> Result<Vec<u8>, age::EncryptError> {
    let key = format!("{}{}",&password,&salt);
    let encryptor = age::Encryptor::with_user_passphrase(Secret::new(key.to_owned()));
    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted)?;
    writer.write_all(content.as_bytes())?;
    writer.finish()?;

    Ok(encrypted)
}


fn decrypt(data: &Vec<u8>, password: &String, salt: &String) -> Result<String, age::DecryptError> {
    let key = format!("{}{}",&password,&salt);
    let decryptor = match age::Decryptor::new(&data[..])? {
        age::Decryptor::Passphrase(d) => d,
        _ => unreachable!(),
    };

    let mut decrypted = vec![];
    let mut reader: age::stream::StreamReader<&[u8]> = decryptor.decrypt(&Secret::new(key.to_owned()), None)?;
    reader.read_to_end(&mut decrypted)?;

    Ok(String::from_utf8(decrypted).unwrap())
}


fn read_data(name: &String) -> io::Result<Vec<u8>> {
    let path = format!("bucket/chats/{}.txt", &name);
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}


struct DataError;


fn get_salt_from_data(data: &Vec<u8>) -> Result<String, DataError> {
    if data.len() < SALT_SIZE {
        return Err(DataError)
    }

    match String::from_utf8(data[0..SALT_SIZE].to_vec()) {
        Ok(val) => Ok(val),
        Err(_e) => Err(DataError)
    }
}


fn get_contents_from_data(data: &Vec<u8>, password: &String) -> Result<String, DataError> {
    if data.len() <= SALT_SIZE {
        return Err(DataError)
    }

    let salt = get_salt_from_data(data)?;

    let encrypted_data = &data[SALT_SIZE..];
    match decrypt(&encrypted_data.to_vec(), &password, &salt) {
        Ok(val) => Ok(val),
        Err(_e) => Err(DataError),
    }
}


fn write_content(chat_name: &String, password: &String, content: &String, salt: &String) -> io::Result<()> {
    let mut content_short = String::new();

    for c in content.chars() {
        content_short.push(c);
        if content_short.len() >= CONTENT_SIZE_LIMIT {
            break;
        }
    }

    let path = format!("bucket/chats/{}.txt", &chat_name);
    let mut file: File = File::create(path).unwrap();
    file.write_all(&salt.as_bytes())?;
    match encrypt(&content_short, &password, salt) {
        Ok(data) => {
            file.write(&data)?;
            Ok(())
        },
        Err(_e) => Err(io::Error::new(io::ErrorKind::InvalidData, "Error")),
    }
}


fn write_first_content(chat_post: &ChatPost) -> io::Result<()> {
    write_content(&chat_post.name, &chat_post.password, &chat_post.content, &generate_salt(SALT_SIZE))
}


fn append_content(chat_post: &ChatPost) -> io::Result<()> {
    let data = match read_data(&chat_post.name) {
        Ok(val) => val,
        Err(e) => return Err(e),
    };

    let salt = match get_salt_from_data(&data) {
        Ok(val) => val,
        Err(_e) => return Err(io::Error::new(io::ErrorKind::Other, "Error")),
    };

    let content = match get_contents_from_data(&data, &chat_post.password) {
        Ok(val) => val,
        Err(_e) => return Err(io::Error::new(io::ErrorKind::InvalidData, "Error")),
    };

    let new_content = format!("{}{}", &chat_post.content, content);
    write_content(&chat_post.name, &chat_post.password, &new_content, &salt)
}


#[post("/who_chat/get")]
async fn get_chat(chat_access: web::Json<ChatAccess>) -> impl Responder {
    let data = match read_data(&chat_access.name) {
        Ok(v) => v,
        Err(_e) => return HttpResponse::NotFound().content_type("text/plain").body("Chat not found!"),
    };
    match get_contents_from_data(&data, &chat_access.password) {
        Ok(contents) => HttpResponse::Ok().content_type("text/plain").body(contents),
        Err(_e) => HttpResponse::InternalServerError().content_type("text/plain").body("Error!"),
    }
}


#[post("/who_chat/post")]
async fn post_chat(chat_post: web::Json<ChatPost>) -> impl Responder {
    let path = format!("bucket/chats/{}.txt", &chat_post.name);

    if Path::new(&path).exists() {
        match append_content(&chat_post) {
            Ok(_val) => HttpResponse::Ok().content_type("text/plain").body("Ok!"),
            Err(_e) => HttpResponse::InternalServerError().content_type("text/plain").body("Error!"),
        }
    }
    else{
        match write_first_content(&chat_post) {
            Ok(_val) => HttpResponse::Ok().content_type("text/plain").body("Ok!"),
            Err(_e) => HttpResponse::InternalServerError().content_type("text/plain").body("Error!"),
        }
    }
}

#[get("/who_chat")]
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
                <canvas id=\"background\">
                </canvas>

               <script type=\"text/javascript\" src=\"/static/js/matrix_background.js\"></script>

                {}

                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/who_chat.css\">

                <div class=\"content\">
                    <form id=\"data-form\" onsubmit=\"event.preventDefault(); return get_chat_data()\">
                        <label for=\"name\">Chat name:</label>
                        <br>
                        <input type=\"text\" id=\"name\" name=\"name\">
                        <br>
                        <label for=\"password\">Password:</label>
                        <br>
                        <input type=\"password\" id=\"password\" name=\"password\">
                        <br>
                        <input class=\"button\" type=\"submit\" value=\"Read Chat\">
                    </form> 
                </div>
            </body>

            <script type=\"text/javascript\" src=\"/static/js/who_chat.js\"></script>

        </html>
    ", navbar());

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content)) 
}