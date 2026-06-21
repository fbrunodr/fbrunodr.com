use std::fs;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Result};
use actix_web::cookie::Cookie;
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection};

use crate::components::navbar::navbar;

const DB_PATH: &str = "bucket/lucasodon.db";
const PASSWORD_PATH: &str = "bucket/lucasodon_password.txt";
const DEFAULT_PASSWORD: &str = "lucasodon";
const COOKIE_NAME: &str = "lucasodon_auth";

const CREATE_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS plantoes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        data TEXT NOT NULL,
        local TEXT NOT NULL,
        mfc_pa TEXT NOT NULL,
        duracao_h REAL NOT NULL,
        periodo TEXT NOT NULL,
        valor REAL NOT NULL,
        previsao_pagamento TEXT,
        recebido INTEGER NOT NULL DEFAULT 0,
        mes_ano_pagamento TEXT,
        observacoes TEXT
    )
";


// ----------------------------- Data models -----------------------------

#[derive(Deserialize)]
struct PlantaoInput {
    data: String,
    local: String,
    mfc_pa: String,
    duracao_h: f64,
    periodo: String,
    valor: f64,
    previsao_pagamento: Option<String>,
    recebido: bool,
    mes_ano_pagamento: Option<String>,
    observacoes: Option<String>,
}

#[derive(Deserialize)]
struct PlantaoUpdate {
    id: i64,
    #[serde(flatten)]
    fields: PlantaoInput,
}

#[derive(Deserialize)]
struct IdOnly {
    id: i64,
}

#[derive(Serialize)]
struct Plantao {
    id: i64,
    data: String,
    local: String,
    mfc_pa: String,
    duracao_h: f64,
    periodo: String,
    valor: f64,
    valor_hora: f64,
    previsao_pagamento: Option<String>,
    recebido: bool,
    mes_ano_pagamento: Option<String>,
    observacoes: Option<String>,
}

#[derive(Deserialize)]
struct LoginForm {
    password: String,
}


// ----------------------------- Auth -----------------------------

// The access password is kept in a local file so it never lives in the
// source tree. On first run we create it with a default value.
fn get_password() -> String {
    match fs::read_to_string(PASSWORD_PATH) {
        Ok(p) => p.trim().to_string(),
        Err(_) => {
            let _ = fs::write(PASSWORD_PATH, DEFAULT_PASSWORD);
            DEFAULT_PASSWORD.to_string()
        }
    }
}

fn is_authed(req: &HttpRequest) -> bool {
    match req.cookie(COOKIE_NAME) {
        Some(c) => c.value() == get_password(),
        None => false,
    }
}


// ----------------------------- Database -----------------------------

fn open_db() -> rusqlite::Result<Connection> {
    let conn = Connection::open(DB_PATH)?;
    conn.execute(CREATE_TABLE_SQL, [])?;
    Ok(conn)
}

fn list_plantoes() -> rusqlite::Result<Vec<Plantao>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare(
        "SELECT id, data, local, mfc_pa, duracao_h, periodo, valor,
                previsao_pagamento, recebido, mes_ano_pagamento, observacoes
         FROM plantoes
         ORDER BY data ASC, id ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        let duracao_h: f64 = row.get(4)?;
        let valor: f64 = row.get(6)?;
        Ok(Plantao {
            id: row.get(0)?,
            data: row.get(1)?,
            local: row.get(2)?,
            mfc_pa: row.get(3)?,
            duracao_h,
            periodo: row.get(5)?,
            valor,
            valor_hora: if duracao_h > 0.0 { valor / duracao_h } else { 0.0 },
            previsao_pagamento: row.get(7)?,
            recebido: row.get::<_, i64>(8)? != 0,
            mes_ano_pagamento: row.get(9)?,
            observacoes: row.get(10)?,
        })
    })?;

    rows.collect()
}

fn insert_plantao(p: &PlantaoInput) -> rusqlite::Result<()> {
    let conn = open_db()?;
    conn.execute(
        "INSERT INTO plantoes
            (data, local, mfc_pa, duracao_h, periodo, valor,
             previsao_pagamento, recebido, mes_ano_pagamento, observacoes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            p.data, p.local, p.mfc_pa, p.duracao_h, p.periodo, p.valor,
            p.previsao_pagamento, p.recebido as i64, p.mes_ano_pagamento, p.observacoes,
        ],
    )?;
    Ok(())
}

fn update_plantao(u: &PlantaoUpdate) -> rusqlite::Result<usize> {
    let conn = open_db()?;
    let p = &u.fields;
    conn.execute(
        "UPDATE plantoes SET
            data = ?1, local = ?2, mfc_pa = ?3, duracao_h = ?4, periodo = ?5,
            valor = ?6, previsao_pagamento = ?7, recebido = ?8,
            mes_ano_pagamento = ?9, observacoes = ?10
         WHERE id = ?11",
        params![
            p.data, p.local, p.mfc_pa, p.duracao_h, p.periodo, p.valor,
            p.previsao_pagamento, p.recebido as i64, p.mes_ano_pagamento,
            p.observacoes, u.id,
        ],
    )
}

fn delete_plantao(id: i64) -> rusqlite::Result<usize> {
    let conn = open_db()?;
    conn.execute("DELETE FROM plantoes WHERE id = ?1", params![id])
}


// ----------------------------- Routes -----------------------------

#[post("/lucasodon/login")]
async fn login(form: web::Form<LoginForm>) -> impl Responder {
    if form.password.trim() == get_password() {
        let cookie = Cookie::build(COOKIE_NAME, get_password())
            .path("/lucasodon")
            .http_only(true)
            .finish();
        HttpResponse::SeeOther()
            .append_header(("Location", "/lucasodon"))
            .cookie(cookie)
            .finish()
    } else {
        HttpResponse::SeeOther()
            .append_header(("Location", "/lucasodon?erro=1"))
            .finish()
    }
}

#[get("/lucasodon/logout")]
async fn logout() -> impl Responder {
    let mut cookie = Cookie::build(COOKIE_NAME, "")
        .path("/lucasodon")
        .http_only(true)
        .finish();
    cookie.make_removal();
    HttpResponse::SeeOther()
        .append_header(("Location", "/lucasodon"))
        .cookie(cookie)
        .finish()
}

#[get("/lucasodon/api/list")]
async fn list(req: HttpRequest) -> impl Responder {
    if !is_authed(&req) {
        return HttpResponse::Unauthorized().body("nao autorizado");
    }
    match list_plantoes() {
        Ok(plantoes) => HttpResponse::Ok().json(plantoes),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

#[post("/lucasodon/api/create")]
async fn create(req: HttpRequest, input: web::Json<PlantaoInput>) -> impl Responder {
    if !is_authed(&req) {
        return HttpResponse::Unauthorized().body("nao autorizado");
    }
    match insert_plantao(&input) {
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

#[post("/lucasodon/api/update")]
async fn update(req: HttpRequest, input: web::Json<PlantaoUpdate>) -> impl Responder {
    if !is_authed(&req) {
        return HttpResponse::Unauthorized().body("nao autorizado");
    }
    match update_plantao(&input) {
        Ok(0) => HttpResponse::NotFound().body("plantao nao encontrado"),
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

#[post("/lucasodon/api/delete")]
async fn delete(req: HttpRequest, input: web::Json<IdOnly>) -> impl Responder {
    if !is_authed(&req) {
        return HttpResponse::Unauthorized().body("nao autorizado");
    }
    match delete_plantao(input.id) {
        Ok(0) => HttpResponse::NotFound().body("plantao nao encontrado"),
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

#[get("/lucasodon")]
pub async fn render(req: HttpRequest) -> Result<HttpResponse> {
    if !is_authed(&req) {
        return Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(login_page()));
    }

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(app_page()))
}


// ----------------------------- Views -----------------------------

fn login_page() -> String {
    format!("
        <html lang=\"pt-br\">
            <head>
                <meta charset=\"utf-8\" />
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/index.css\">
                <link href=\"https://fonts.googleapis.com/css2?family=Open+Sans:wght@300;400;600&family=Reenie+Beanie&display=swap\" rel=\"stylesheet\">
                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/lucasodon.css\">
            </head>
            <body>
                {}
                <div class=\"content lucasodon-login\">
                    <h1 class=\"lucasodon-title\">Controle de Plant&otilde;es</h1>
                    <form class=\"login-card\" method=\"POST\" action=\"/lucasodon/login\">
                        <label for=\"password\">Senha</label>
                        <input type=\"password\" id=\"password\" name=\"password\" autofocus required>
                        <button type=\"submit\">Entrar</button>
                    </form>
                </div>
            </body>
        </html>
    ", navbar())
}

fn app_page() -> String {
    format!("
        <html lang=\"pt-br\">
            <head>
                <meta charset=\"utf-8\" />
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/index.css\">
                <link href=\"https://fonts.googleapis.com/css2?family=Open+Sans:wght@300;400;600&family=Reenie+Beanie&display=swap\" rel=\"stylesheet\">
                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/lucasodon.css\">
            </head>
            <body>
                {}
                <div class=\"content lucasodon-app\">
                    <div class=\"lucasodon-header\">
                        <h1 class=\"lucasodon-title\">Controle de Plant&otilde;es</h1>
                        <a class=\"logout-link\" href=\"/lucasodon/logout\">Sair</a>
                    </div>

                    <div id=\"totais\" class=\"totais\"></div>

                    <button id=\"open-form-btn\" class=\"open-form-btn\">+ Lan&ccedil;ar novo plant&atilde;o</button>

                    <div id=\"msg\" class=\"msg\"></div>

                    <div id=\"form-modal\" class=\"modal-overlay\">
                        <div class=\"modal\">
                            <div class=\"modal-head\">
                                <h2>Lan&ccedil;ar novo plant&atilde;o</h2>
                                <button id=\"close-form-btn\" class=\"modal-close\" aria-label=\"Fechar\">&times;</button>
                            </div>
                            <div class=\"form-card\">
                                <div class=\"field\"><label>Data</label><input type=\"date\" id=\"f-data\"></div>
                                <div class=\"field\"><label>Local</label><input type=\"text\" id=\"f-local\" list=\"locais-list\" placeholder=\"FAM Barrinha\"></div>
                                <div class=\"field\"><label>MFC/PA</label>
                                    <select id=\"f-mfc_pa\"><option>PA</option><option>MFC</option></select>
                                </div>
                                <div class=\"field\"><label>Dura&ccedil;&atilde;o (h)</label><input type=\"number\" step=\"0.5\" id=\"f-duracao_h\" placeholder=\"12\"></div>
                                <div class=\"field\"><label>Per&iacute;odo</label>
                                    <select id=\"f-periodo\">
                                        <option>Diurno</option><option>Noturno</option>
                                        <option>Cinderela</option><option>24hrs</option>
                                        <option>24hrs invertido</option>
                                    </select>
                                </div>
                                <div class=\"field\"><label>Valor (R$)</label><input type=\"number\" step=\"0.01\" id=\"f-valor\" placeholder=\"1200\"></div>
                                <div class=\"field\"><label>Previs&atilde;o pagamento</label><input type=\"date\" id=\"f-previsao_pagamento\"></div>
                                <div class=\"field\"><label>Dia de pagamento</label><input type=\"date\" id=\"f-mes_ano_pagamento\"></div>
                                <div class=\"field checkbox\"><label>Recebido?</label><input type=\"checkbox\" id=\"f-recebido\"></div>
                                <div class=\"field wide\"><label>Observa&ccedil;&otilde;es</label><input type=\"text\" id=\"f-observacoes\"></div>
                            </div>
                            <div class=\"modal-actions\">
                                <button id=\"cancel-form-btn\" class=\"edit\">Cancelar</button>
                                <button id=\"add-btn\">Adicionar</button>
                            </div>
                        </div>
                    </div>

                    <datalist id=\"locais-list\"></datalist>

                    <div class=\"view-toggle\">
                        <button id=\"tab-table\" class=\"view-btn active\" type=\"button\">Tabela</button>
                        <button id=\"tab-dash\" class=\"view-btn\" type=\"button\">Dashboard</button>
                    </div>

                    <div id=\"view-table\" class=\"table-wrap\">
                        <table id=\"plantoes-table\">
                            <thead>
                                <tr>
                                    <th>#</th><th>Data</th><th>Local</th><th>MFC/PA</th>
                                    <th>Dura&ccedil;&atilde;o</th><th>Per&iacute;odo</th><th>Valor</th>
                                    <th>R$/h</th><th>Previs&atilde;o</th><th>Recebido</th>
                                    <th>Dia de pagamento</th><th>Observa&ccedil;&otilde;es</th><th></th>
                                </tr>
                            </thead>
                            <tbody id=\"plantoes-body\"></tbody>
                        </table>
                    </div>

                    <div id=\"view-dashboard\" class=\"dashboard\" style=\"display: none;\"></div>
                </div>

                <script type=\"text/javascript\" src=\"/static/js/lucasodon.js\"></script>
            </body>
        </html>
    ", navbar())
}
