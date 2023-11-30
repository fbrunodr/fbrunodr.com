use std::fs::{File, self};
use std::io::{Write, Read, self};
use std::path::Path;
use actix_web::{get, post, web, HttpResponse, Responder, Result};
use serde::Deserialize;
use age::secrecy::Secret;
use rand::{distributions::Alphanumeric, Rng};
use crate::components::navbar::navbar;

const SALT_SIZE: usize = 32;
const CONTENT_SIZE_LIMIT: usize = 100_000;


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


struct DeleteError;


fn delete_data(chat_access: &ChatAccess) -> Result<(), DeleteError> {
    let data = match read_data(&chat_access.name) {
        Ok(v) => v,
        Err(_e) => return Err(DeleteError),
    };

    match get_contents_from_data(&data, &chat_access.password) {
        Ok(_contents) => {
            match fs::remove_file(format!("bucket/chats/{}.txt", chat_access.name)) {
                Ok(_v) => (),
                Err(_e) => return Err(DeleteError),
            };
            Ok(())
        },
        Err(_e) => Err(DeleteError),
    }
}


fn truncate_start_string(content: &String) -> String {
    let mut start_idx = content.chars().count();
    let mut curr_len: usize= 0;
    for c in content.chars().rev() {
        curr_len += c.len_utf8();
        start_idx -= 1;

        if curr_len > CONTENT_SIZE_LIMIT {
            start_idx += 1;
            break;
        }
    }

    return content.chars().skip(start_idx).collect()
}


fn write_content(chat_name: &String, password: &String, content: &String, salt: &String) -> io::Result<()> {
    let content_truncated = truncate_start_string(&content);

    let path = format!("bucket/chats/{}.txt", &chat_name);
    let mut file: File = File::create(path).unwrap();
    file.write_all(&salt.as_bytes())?;
    match encrypt(&content_truncated, &password, salt) {
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

    let new_content = format!("{}{}", &content, &chat_post.content);
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
            Ok(_val) => HttpResponse::Ok().content_type("text/plain").body("Posted!"),
            Err(_e) => HttpResponse::InternalServerError().content_type("text/plain").body("Error!"),
        }
    }
    else{
        match write_first_content(&chat_post) {
            Ok(_val) => HttpResponse::Ok().content_type("text/plain").body("Posted!"),
            Err(_e) => HttpResponse::InternalServerError().content_type("text/plain").body("Error!"),
        }
    }
}


#[post("/who_chat/delete")]
async fn delete_chat(chat_access: web::Json<ChatAccess>) -> impl Responder {
    match delete_data(&chat_access) {
        Ok(_val) => HttpResponse::Ok().content_type("text/plain").body("Deleted!"),
        Err(_e) => HttpResponse::InternalServerError().content_type("text/plain").body("Error!"),
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

                    <h1 class=\"title who-chat\">Who Chat</h1>

                    <div class=\"intro\">
                        <p>
                            Post data to a chat anonimously using <span class=\"who-chat\">Who chat</span>.
                            This service employs robust encryption, ensuring that only individuals with
                            the correct password can access and post to the conversation. Even I, Francisco
                            Bruno, can not read or write to your chats, although I can delete them. Some
                            things you should mind:
                        </p>

                        <ul>
                            <li> Each chat has a maximum data capacity of 100KB. When this limit is reached,
                            the system automatically removes the oldest data, ensuring that only the most
                            recent 100KB of text is displayed.</li>

                            <li> Although I can not read the contents inside a chat, I, Francisco Bruno, can
                            read the chat's name. Don't put sensitive information on the chat's name.</li>

                            <li> If you post to a chat that does not exist, the chat is created then the
                            information is written in the chat using the provided password and a ramdonly
                            generated salt. If the chat already exists, the information is posted only if
                            the password matches the one used during creation.
                            </li>
                        </ul>
                    </div>

                    <div id=\"mode-switch\">
                        <div id=\"get-switch\" class=\"switch\" onClick=\"set_get_chat();\">Get</div>
                        <div id=\"post-switch\" class=\"switch\" onClick=\"set_post_chat();\">Post</div>
                        <div id=\"delete-switch\" class=\"switch\" onClick=\"set_delete_chat();\">Delete</div>
                    </div>

                    <div id=\"forms-wrapper\">
                    </div>

                    <div id=\"response\">
                    </div>

                </div>
            </body>

            <script type=\"text/javascript\" src=\"/static/js/who_chat.js\"></script>

        </html>
    ", navbar());

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content)) 
}
