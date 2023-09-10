use std::fs;
use actix_web::{get, HttpResponse, Result};

use crate::components::navbar::navbar;

struct Class{
    id: i32,
    name: String,
    url: String,
}

struct Source{
    id: i32,
    name: String,
    url: String,
}

fn get_classes() -> Vec<Class> {
    let contents = fs::read_to_string("bucket/classes_data.txt")
        .expect("Should have been able to read the file");

    let lines: Vec<&str> = contents.lines().collect();

    let mut classes: Vec<Class> = Vec::new();

    for line in lines {
        let fields: Vec<&str> = line.split(';').collect();

        if fields.len() != 3 {
            continue;
        }

        let id = fields[0].parse::<i32>().unwrap();
        let name = fields[1].to_string();
        let url = fields[2].to_string();

        let class = Class {
            id,
            name,
            url,
        };

        classes.push(class);
    }

    classes
}

fn get_sources() -> Vec<Source> {
    let contents = fs::read_to_string("bucket/classes_sources.txt")
        .expect("Should have been able to read the file");

    let lines: Vec<&str> = contents.lines().collect();

    let mut sources: Vec<Source> = Vec::new();

    for line in lines {
        let fields: Vec<&str> = line.split(';').collect();

        if fields.len() != 3 {
            continue;
        }

        let id = fields[0].parse::<i32>().unwrap();
        let name = fields[1].to_string();
        let url = fields[2].to_string();

        let source = Source {
            id,
            name,
            url,
        };

        sources.push(source);
    }

    sources
}


#[get("/CompetitiveProgrammingClasses")]
pub async fn render() -> Result<HttpResponse> {
    let classes = get_classes();
    let classes_html = classes.iter().map(|class| format!("
        <a class=\"aula\" key={} href={}>
            {} - Aula {}
        </a>
        ",
        class.id,
        class.url,
        class.name,
        class.id,
    )).collect::<Vec<String>>().join("\n");

    let sources = get_sources();
    let sources_html = sources.iter().map(|source| format!("
        <a class=\"aula\" key={} href={}>
            {}
        </a>
        ",
        source.id,
        source.url,
        source.name,
    )).collect::<Vec<String>>().join("\n");

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
                <div class=\"content\">
                    <div class=\"intro\">
                        <h2 class=\"greetings\"> Olá competidores, </h2>
                        <p>
                            Durante o ano de 2022 fui o presidente da ITAbits e ministrei aulas de
                            programação competitiva. Essas aulas tem o intuito de dar base para os
                            alunos poderem praticar questões de alto nível e se prepararem para
                            competições regionais (no Brasil, a Maratona de Programação) e avançarem
                            para ICPC (International Collegiate Programming Contest). Além disso,
                            durante as aulas são passadas algumas dicas para entrevistas técnicas
                            de Big Techs. Essas aulas foram gravadas e podem ser acessadas abaixo:
                        </p>
                    </div>

                    <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/cp.css\">

                    <div class=\"aulas\">
                        {}
                    </div>

                    <div class=\"intro\">
                        <p>
                            Abaixo você também pode acessar o código das aulas comentado:
                        </p>
                    </div>

                    <div class=\"aulas\">
                        {}
                    </div>
                </div>
            </body>
        </html>
    ", navbar(), classes_html, sources_html);
    
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content))
}
