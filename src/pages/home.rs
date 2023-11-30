use std::fs;
use actix_web::{get, HttpResponse, Result};

use crate::components::navbar::navbar;

fn get_skill_color(skill: &String) -> String {
    match skill.as_str() {
        "Computer Vision" => String::from("\"color: #2525e5;\""),
        "AWS" => String::from("\"color: #FF9900;\""),
        "C++" => String::from("\"color: #659ad1;\""),
        "Rust" => String::from("\"color: #ce422b;\""),
        "Data Structures" => String::from("\"color: #d0d0d0;\""),
        "Portuguese" | "Object Detection" => String::from("\"color: #cfbb0e;\""),
        "IT" => String::from("\"color: #2e6299;\""),
        "Agile development" => String::from("\"color: #37b88b;\""),
        "Python" => String::from("\"color: #386e9d;\""),
        "Design Patterns" => String::from("\"color: #ff9f38;\""),
        "DP" => String::from("\"color: #392fed;\""),
        "Algorithms" | "Object Tracking" => String::from("\"color: #c71212;\""),
        "Quantum Computing" => String::from("\"color: #00B0F0;\""),
        "Deep Neural Network" => String::from("\"color: #2fbdbf;\""),
        "Security" => String::from("\"color: #af46d1;\""),
        "Encryption" => String::from("\"color: #28bb34;\""),
        "Aerospace" => String::from("\"color: #7c33c4;\""),
        "Optimization" => String::from("\"color: red;\""),
        _ => String::from("\"color: red;\""),
    }
}

struct Post{
    id: i32,
    title: String,
    summary: String,
    image_url: String,
    image_description: String,
    skills: Vec<String>,
    url: String,
}

fn get_posts() -> Vec<Post> {
    let contents = fs::read_to_string("bucket/posts_data.txt")
        .expect("Should have been able to read the file");

    let lines: Vec<&str> = contents.lines().collect();

    let mut posts: Vec<Post> = Vec::new();

    for line in lines {
        let fields: Vec<&str> = line.split(';').collect();

        if fields.len() != 7 {
            continue;
        }

        let id = fields[0].parse::<i32>().unwrap();
        let title = fields[1].to_string();
        let summary = fields[2].to_string();
        let image_url = fields[3].to_string();
        let image_description = fields[4].to_string();
        let skills = fields[5].split(',').map(|s| s.to_string()).collect();
        let url = fields[6].to_string();

        let post = Post {
            id,
            title,
            summary,
            image_url,
            image_description,
            skills,
            url,
        };

        posts.push(post);
    }

    posts
}


fn format_skills(skills: &Vec<String>) -> String {
    String::from("skills: ") +
    &skills
        .iter()
        .map(|skill| format!("<div class=\"skill\" style={}> &nbsp;{}</div>", get_skill_color(skill), skill))
        .collect::<Vec<String>>()
        .join("\n")
}


#[get("/")]
pub async fn render() -> Result<HttpResponse> {
    let welcome_content = "
        <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/welcome.css\">
        <script type=\"text/javascript\" src=\"/static/js/welcome.js\"></script>
        <div class=\"welcome\">

            <div class=\"welcome-background\"></div>

            <div class=\"welcome-text-container\">

                <div class=\"welcome-text-block\" id=\"hello\">
                    <h2 class=\"welcome-text big\" id=\"hello-text\"> </h2>
                    <span class=\"input-cursor big\" id=\"hello-input\"> </span>
                </div>

                <div class=\"welcome-text-block\" id=\"name\">
                    <h3 class=\"welcome-text\" id=\"name-text\"> </h3>
                    <span id=\"name-input\"> </span>
                </div>

                <div class=\"welcome-text-block\" id=\"welcome\">
                    <h2 class=\"welcome-text big\" id=\"welcome-text\"> </h2>
                    <span class=\"big\" id=\"welcome-input\"> </span>
                </div>

                <div class=\"welcome-text-block\" id=\"site\">
                    <h3 class=\"welcome-text\" id=\"site-text\"> </h3>
                    <span id=\"site-input\"> </span>
                </div>

            </div>

            <div id=\"gradient\"></div>

        </div>

        <script type=\"text/javascript\">
            typeIntro()
        </script>
    ";

    let mut posts = get_posts();
    posts.reverse();
    let posts_html = posts.iter().map(|post| format!("
        <div class=\"post\" key=\"{}\">
            <div class=\"post-title\">{}</div>
            <div class=\"post-skills\">
                {}
            </div>
            <div class=\"post-image-wrapper\">
                <img src=\"{}\" class=\"post-image\" alt=\"{}\" />
            </div>
            <div class=\"post-summary-wrapper\">
                {}
                <br />
                <a href=\"{}\" class=\"read-more\">Read More</a>
            </div>
        </div>
        ",
        post.id,
        post.title,
        format_skills(&post.skills),
        post.image_url,
        post.image_description,
        post.summary,
        post.url
    )).collect::<Vec<String>>().join("\n");

    let html_content = format!("
        <html lang=\"en\">
            <head>
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/index.css\">
                <link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">
                <link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>
                <link href=\"https://fonts.googleapis.com/css2?family=Open+Sans:wght@300;400&family=Reenie+Beanie&family=Source+Code+Pro&display=swap\" rel=\"stylesheet\">
                <script type=\"text/javascript\" src=\"/static/js/colors.js\"></script>
            <head/>
            <body>
                {}
                {}
                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/posts.css\">
                <div class=\"posts\">
                    {}
                </div>
            </body>
        </html>
    ", navbar(), welcome_content, posts_html);
    
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content))
}
