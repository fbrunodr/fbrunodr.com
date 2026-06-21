use std::fs;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Result};
use actix_web::cookie::Cookie;
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, OptionalExtension};

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

// Despesas module: categories, one-off/materialized expenses, and recurring
// rules. The partial unique index makes recurrence generation idempotent
// (one row per rule per competência), which also lets a deleted occurrence
// stay as a 'cancelado' tombstone that blocks regeneration.
const CREATE_DESPESAS_SQL: &str = "
    CREATE TABLE IF NOT EXISTS categorias (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        nome TEXT NOT NULL UNIQUE,
        padrao INTEGER NOT NULL DEFAULT 0
    );
    CREATE TABLE IF NOT EXISTS despesas_recorrentes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        descricao TEXT NOT NULL,
        valor REAL NOT NULL,
        categoria_id INTEGER,
        tipo TEXT NOT NULL,
        periodicidade TEXT NOT NULL,
        dia_vencimento INTEGER NOT NULL,
        mes_vencimento INTEGER,
        data_inicio TEXT NOT NULL,
        data_fim TEXT,
        ativo INTEGER NOT NULL DEFAULT 1
    );
    CREATE TABLE IF NOT EXISTS despesas (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        valor REAL NOT NULL,
        data TEXT NOT NULL,
        descricao TEXT,
        categoria_id INTEGER,
        tipo TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'pago',
        recorrente_id INTEGER,
        competencia TEXT,
        observacoes TEXT
    );
    CREATE UNIQUE INDEX IF NOT EXISTS idx_despesa_recorrencia
        ON despesas(recorrente_id, competencia) WHERE recorrente_id IS NOT NULL;
";

const CATEGORIAS_PADRAO: [&str; 8] = [
    "Moradia", "Alimentação", "Transporte", "Impostos",
    "Educação", "Lazer", "Saúde", "Outros",
];


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
    conn.execute_batch(CREATE_DESPESAS_SQL)?;
    seed_categorias(&conn)?;
    Ok(conn)
}

fn seed_categorias(conn: &Connection) -> rusqlite::Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM categorias", [], |r| r.get(0))?;
    if count == 0 {
        for nome in CATEGORIAS_PADRAO {
            conn.execute("INSERT INTO categorias (nome, padrao) VALUES (?1, 1)", params![nome])?;
        }
    }
    Ok(())
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


// ----------------------- Despesas: models -----------------------

#[derive(Serialize)]
struct Categoria { id: i64, nome: String, padrao: bool }

#[derive(Deserialize)]
struct CategoriaInput { nome: String }

#[derive(Deserialize)]
struct CategoriaUpdate { id: i64, nome: String }

#[derive(Serialize)]
struct Despesa {
    id: i64,
    valor: f64,
    data: String,
    descricao: Option<String>,
    categoria_id: Option<i64>,
    tipo: String,
    status: String,
    recorrente_id: Option<i64>,
    competencia: Option<String>,
    observacoes: Option<String>,
}

#[derive(Deserialize)]
struct DespesaInput {
    valor: f64,
    data: String,
    descricao: Option<String>,
    categoria_id: Option<i64>,
    tipo: String,
    status: Option<String>,
    observacoes: Option<String>,
}

#[derive(Deserialize)]
struct DespesaUpdate { id: i64, #[serde(flatten)] fields: DespesaInput }

#[derive(Serialize)]
struct Recorrente {
    id: i64,
    descricao: String,
    valor: f64,
    categoria_id: Option<i64>,
    tipo: String,
    periodicidade: String,
    dia_vencimento: i64,
    mes_vencimento: Option<i64>,
    data_inicio: String,
    data_fim: Option<String>,
    ativo: bool,
}

#[derive(Deserialize)]
struct RecorrenteInput {
    descricao: String,
    valor: f64,
    categoria_id: Option<i64>,
    tipo: String,
    periodicidade: String,
    dia_vencimento: i64,
    mes_vencimento: Option<i64>,
    data_inicio: String,
    data_fim: Option<String>,
    ativo: Option<bool>,
}

#[derive(Deserialize)]
struct RecorrenteUpdate { id: i64, #[serde(flatten)] fields: RecorrenteInput }

#[derive(Deserialize)]
struct DespesaFilter {
    inicio: Option<String>,
    fim: Option<String>,
    categoria_id: Option<i64>,
    tipo: Option<String>,
}


// ----------------------- Despesas: date helpers -----------------------

fn is_leap(y: i32) -> bool { (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 }

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap(y) { 29 } else { 28 },
        _ => 30,
    }
}

// Parse the (year, month) out of a "YYYY-MM" or "YYYY-MM-DD" string.
fn parse_ym(s: &str) -> (i32, u32) {
    let y = s.get(0..4).and_then(|v| v.parse().ok()).unwrap_or(0);
    let m = s.get(5..7).and_then(|v| v.parse().ok()).unwrap_or(1);
    (y, m)
}


// ----------------------- Despesas: categorias -----------------------

fn list_categorias() -> rusqlite::Result<Vec<Categoria>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare("SELECT id, nome, padrao FROM categorias ORDER BY nome ASC")?;
    let rows = stmt.query_map([], |r| Ok(Categoria {
        id: r.get(0)?, nome: r.get(1)?, padrao: r.get::<_, i64>(2)? != 0,
    }))?;
    rows.collect()
}

fn insert_categoria(c: &CategoriaInput) -> rusqlite::Result<()> {
    let conn = open_db()?;
    conn.execute("INSERT INTO categorias (nome, padrao) VALUES (?1, 0)", params![c.nome.trim()])?;
    Ok(())
}

fn update_categoria(u: &CategoriaUpdate) -> rusqlite::Result<usize> {
    let conn = open_db()?;
    conn.execute("UPDATE categorias SET nome = ?1 WHERE id = ?2", params![u.nome.trim(), u.id])
}

// Deleting a category reassigns its expenses/rules to "Outros" (or NULL if
// "Outros" itself is being deleted), so nothing is left dangling.
fn delete_categoria(id: i64) -> rusqlite::Result<usize> {
    let conn = open_db()?;
    let outros: Option<i64> = conn
        .query_row("SELECT id FROM categorias WHERE nome = 'Outros'", [], |r| r.get(0))
        .optional()?;
    let target = if outros == Some(id) { None } else { outros };
    conn.execute("UPDATE despesas SET categoria_id = ?1 WHERE categoria_id = ?2", params![target, id])?;
    conn.execute("UPDATE despesas_recorrentes SET categoria_id = ?1 WHERE categoria_id = ?2", params![target, id])?;
    conn.execute("DELETE FROM categorias WHERE id = ?1", params![id])
}


// ----------------------- Despesas: recurring rules -----------------------

fn list_recorrentes() -> rusqlite::Result<Vec<Recorrente>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare(
        "SELECT id, descricao, valor, categoria_id, tipo, periodicidade,
                dia_vencimento, mes_vencimento, data_inicio, data_fim, ativo
         FROM despesas_recorrentes ORDER BY descricao ASC")?;
    let rows = stmt.query_map([], |r| Ok(Recorrente {
        id: r.get(0)?, descricao: r.get(1)?, valor: r.get(2)?, categoria_id: r.get(3)?,
        tipo: r.get(4)?, periodicidade: r.get(5)?, dia_vencimento: r.get(6)?,
        mes_vencimento: r.get(7)?, data_inicio: r.get(8)?, data_fim: r.get(9)?,
        ativo: r.get::<_, i64>(10)? != 0,
    }))?;
    rows.collect()
}

fn insert_recorrente(p: &RecorrenteInput) -> rusqlite::Result<()> {
    let conn = open_db()?;
    conn.execute(
        "INSERT INTO despesas_recorrentes
            (descricao, valor, categoria_id, tipo, periodicidade,
             dia_vencimento, mes_vencimento, data_inicio, data_fim, ativo)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            p.descricao, p.valor, p.categoria_id, p.tipo, p.periodicidade,
            p.dia_vencimento, p.mes_vencimento, p.data_inicio, p.data_fim,
            p.ativo.unwrap_or(true) as i64,
        ],
    )?;
    Ok(())
}

fn update_recorrente(u: &RecorrenteUpdate) -> rusqlite::Result<usize> {
    let conn = open_db()?;
    let p = &u.fields;
    conn.execute(
        "UPDATE despesas_recorrentes SET
            descricao = ?1, valor = ?2, categoria_id = ?3, tipo = ?4, periodicidade = ?5,
            dia_vencimento = ?6, mes_vencimento = ?7, data_inicio = ?8, data_fim = ?9, ativo = ?10
         WHERE id = ?11",
        params![
            p.descricao, p.valor, p.categoria_id, p.tipo, p.periodicidade,
            p.dia_vencimento, p.mes_vencimento, p.data_inicio, p.data_fim,
            p.ativo.unwrap_or(true) as i64, u.id,
        ],
    )
}

// Deleting a rule drops its still-pending ('previsto') occurrences but keeps
// already-paid ones as historical record.
fn delete_recorrente(id: i64) -> rusqlite::Result<usize> {
    let conn = open_db()?;
    conn.execute("DELETE FROM despesas WHERE recorrente_id = ?1 AND status = 'previsto'", params![id])?;
    conn.execute("DELETE FROM despesas_recorrentes WHERE id = ?1", params![id])
}


// ----------------------- Despesas: lançamentos -----------------------

// Lazily materialize occurrences of every active rule across the [inicio, fim]
// window. Idempotent via INSERT OR IGNORE on (recorrente_id, competencia).
fn materialize_recorrentes(conn: &Connection, inicio: &str, fim: &str) -> rusqlite::Result<()> {
    let (y0, m0) = parse_ym(inicio);
    let (y1, m1) = parse_ym(fim);

    let mut stmt = conn.prepare(
        "SELECT id, valor, categoria_id, tipo, periodicidade,
                dia_vencimento, mes_vencimento, data_inicio, data_fim
         FROM despesas_recorrentes WHERE ativo = 1")?;
    let rules: Vec<(i64, f64, Option<i64>, String, String, i64, Option<i64>, String, Option<String>)> =
        stmt.query_map([], |r| Ok((
            r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?,
            r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?,
        )))?.collect::<rusqlite::Result<Vec<_>>>()?;

    for (id, valor, cat, tipo, periodicidade, dia_venc, mes_venc, data_inicio, data_fim) in rules {
        let (sy, sm) = parse_ym(&data_inicio);
        let (mut y, mut m) = (y0, m0);
        let mut guard = 0;
        while (y, m) <= (y1, m1) {
            guard += 1;
            if guard > 600 { break; }

            let within_start = (y, m) >= (sy, sm);
            let within_end = match &data_fim {
                Some(df) => { let (ey, em) = parse_ym(df); (y, m) <= (ey, em) }
                None => true,
            };
            let applies = if periodicidade == "anual" {
                mes_venc.map_or(false, |mv| mv as u32 == m)
            } else {
                true
            };

            if within_start && within_end && applies {
                let dom = (dia_venc as u32).clamp(1, days_in_month(y, m));
                let data = format!("{:04}-{:02}-{:02}", y, m, dom);
                let comp = format!("{:04}-{:02}", y, m);
                conn.execute(
                    "INSERT OR IGNORE INTO despesas
                        (valor, data, descricao, categoria_id, tipo, status, recorrente_id, competencia, observacoes)
                     SELECT ?1, ?2, descricao, ?3, ?4, 'previsto', ?5, ?6, NULL
                     FROM despesas_recorrentes WHERE id = ?5",
                    params![valor, data, cat, tipo, id, comp],
                )?;
            }

            if m == 12 { y += 1; m = 1; } else { m += 1; }
        }
    }
    Ok(())
}

fn list_despesas(
    inicio: Option<&str>, fim: Option<&str>,
    categoria_id: Option<i64>, tipo: Option<&str>,
) -> rusqlite::Result<Vec<Despesa>> {
    let conn = open_db()?;
    if let (Some(i), Some(f)) = (inicio, fim) {
        materialize_recorrentes(&conn, i, f)?;
    }
    let lo = inicio.unwrap_or("0001-01-01");
    let hi = fim.unwrap_or("9999-12-31");

    let mut stmt = conn.prepare(
        "SELECT id, valor, data, descricao, categoria_id, tipo, status,
                recorrente_id, competencia, observacoes
         FROM despesas
         WHERE status != 'cancelado' AND data >= ?1 AND data <= ?2
         ORDER BY data ASC, id ASC")?;
    let rows = stmt.query_map(params![lo, hi], |r| Ok(Despesa {
        id: r.get(0)?, valor: r.get(1)?, data: r.get(2)?, descricao: r.get(3)?,
        categoria_id: r.get(4)?, tipo: r.get(5)?, status: r.get(6)?,
        recorrente_id: r.get(7)?, competencia: r.get(8)?, observacoes: r.get(9)?,
    }))?;
    let mut v: Vec<Despesa> = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    if let Some(c) = categoria_id { v.retain(|d| d.categoria_id == Some(c)); }
    if let Some(t) = tipo { v.retain(|d| d.tipo == t); }
    Ok(v)
}

fn insert_despesa(d: &DespesaInput) -> rusqlite::Result<()> {
    let conn = open_db()?;
    let status = d.status.clone().unwrap_or_else(|| "pago".to_string());
    conn.execute(
        "INSERT INTO despesas
            (valor, data, descricao, categoria_id, tipo, status, recorrente_id, competencia, observacoes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, ?7)",
        params![d.valor, d.data, d.descricao, d.categoria_id, d.tipo, status, d.observacoes],
    )?;
    Ok(())
}

fn update_despesa(u: &DespesaUpdate) -> rusqlite::Result<usize> {
    let conn = open_db()?;
    let d = &u.fields;
    let status = d.status.clone().unwrap_or_else(|| "pago".to_string());
    conn.execute(
        "UPDATE despesas SET
            valor = ?1, data = ?2, descricao = ?3, categoria_id = ?4,
            tipo = ?5, status = ?6, observacoes = ?7
         WHERE id = ?8",
        params![d.valor, d.data, d.descricao, d.categoria_id, d.tipo, status, d.observacoes, u.id],
    )
}

// An occurrence of a recurring rule is tombstoned ('cancelado') so it won't be
// regenerated; a one-off expense is hard-deleted.
fn delete_despesa(id: i64) -> rusqlite::Result<usize> {
    let conn = open_db()?;
    let recorrente_id: Option<i64> = conn
        .query_row("SELECT recorrente_id FROM despesas WHERE id = ?1", params![id], |r| r.get(0))
        .optional()?
        .flatten();
    if recorrente_id.is_some() {
        conn.execute("UPDATE despesas SET status = 'cancelado' WHERE id = ?1", params![id])
    } else {
        conn.execute("DELETE FROM despesas WHERE id = ?1", params![id])
    }
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

// ---- Categorias ----

#[get("/lucasodon/api/categorias")]
async fn categorias_list(req: HttpRequest) -> impl Responder {
    if !is_authed(&req) { return HttpResponse::Unauthorized().body("nao autorizado"); }
    match list_categorias() {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

#[post("/lucasodon/api/categorias/create")]
async fn categoria_create(req: HttpRequest, input: web::Json<CategoriaInput>) -> impl Responder {
    if !is_authed(&req) { return HttpResponse::Unauthorized().body("nao autorizado"); }
    match insert_categoria(&input) {
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

#[post("/lucasodon/api/categorias/update")]
async fn categoria_update(req: HttpRequest, input: web::Json<CategoriaUpdate>) -> impl Responder {
    if !is_authed(&req) { return HttpResponse::Unauthorized().body("nao autorizado"); }
    match update_categoria(&input) {
        Ok(0) => HttpResponse::NotFound().body("categoria nao encontrada"),
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

#[post("/lucasodon/api/categorias/delete")]
async fn categoria_delete(req: HttpRequest, input: web::Json<IdOnly>) -> impl Responder {
    if !is_authed(&req) { return HttpResponse::Unauthorized().body("nao autorizado"); }
    match delete_categoria(input.id) {
        Ok(0) => HttpResponse::NotFound().body("categoria nao encontrada"),
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

// ---- Recorrentes ----

#[get("/lucasodon/api/recorrentes")]
async fn recorrentes_list(req: HttpRequest) -> impl Responder {
    if !is_authed(&req) { return HttpResponse::Unauthorized().body("nao autorizado"); }
    match list_recorrentes() {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

#[post("/lucasodon/api/recorrentes/create")]
async fn recorrente_create(req: HttpRequest, input: web::Json<RecorrenteInput>) -> impl Responder {
    if !is_authed(&req) { return HttpResponse::Unauthorized().body("nao autorizado"); }
    match insert_recorrente(&input) {
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

#[post("/lucasodon/api/recorrentes/update")]
async fn recorrente_update(req: HttpRequest, input: web::Json<RecorrenteUpdate>) -> impl Responder {
    if !is_authed(&req) { return HttpResponse::Unauthorized().body("nao autorizado"); }
    match update_recorrente(&input) {
        Ok(0) => HttpResponse::NotFound().body("recorrente nao encontrada"),
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

#[post("/lucasodon/api/recorrentes/delete")]
async fn recorrente_delete(req: HttpRequest, input: web::Json<IdOnly>) -> impl Responder {
    if !is_authed(&req) { return HttpResponse::Unauthorized().body("nao autorizado"); }
    match delete_recorrente(input.id) {
        Ok(0) => HttpResponse::NotFound().body("recorrente nao encontrada"),
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

// ---- Despesas ----

#[get("/lucasodon/api/despesas")]
async fn despesas_list(req: HttpRequest, q: web::Query<DespesaFilter>) -> impl Responder {
    if !is_authed(&req) { return HttpResponse::Unauthorized().body("nao autorizado"); }
    let tipo = q.tipo.clone().filter(|t| !t.is_empty() && t != "todos");
    match list_despesas(q.inicio.as_deref(), q.fim.as_deref(), q.categoria_id, tipo.as_deref()) {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

#[post("/lucasodon/api/despesas/create")]
async fn despesa_create(req: HttpRequest, input: web::Json<DespesaInput>) -> impl Responder {
    if !is_authed(&req) { return HttpResponse::Unauthorized().body("nao autorizado"); }
    match insert_despesa(&input) {
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

#[post("/lucasodon/api/despesas/update")]
async fn despesa_update(req: HttpRequest, input: web::Json<DespesaUpdate>) -> impl Responder {
    if !is_authed(&req) { return HttpResponse::Unauthorized().body("nao autorizado"); }
    match update_despesa(&input) {
        Ok(0) => HttpResponse::NotFound().body("despesa nao encontrada"),
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro: {}", e)),
    }
}

#[post("/lucasodon/api/despesas/delete")]
async fn despesa_delete(req: HttpRequest, input: web::Json<IdOnly>) -> impl Responder {
    if !is_authed(&req) { return HttpResponse::Unauthorized().body("nao autorizado"); }
    match delete_despesa(input.id) {
        Ok(0) => HttpResponse::NotFound().body("despesa nao encontrada"),
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
                        <h1 class=\"lucasodon-title\">Controle Financeiro</h1>
                        <a class=\"logout-link\" href=\"/lucasodon/logout\">Sair</a>
                    </div>

                    <div class=\"main-tabs\">
                        <button id=\"maintab-plantoes\" class=\"main-tab active\" type=\"button\">Plant&otilde;es</button>
                        <button id=\"maintab-despesas\" class=\"main-tab\" type=\"button\">Despesas</button>
                    </div>

                    <div id=\"section-plantoes\">
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

                    <div id=\"section-despesas\" style=\"display: none;\">
                        <div id=\"despesas-totais\" class=\"totais\"></div>

                        <div class=\"despesas-toolbar\">
                            <div class=\"filtros\">
                                <div class=\"field\"><label>Per&iacute;odo</label>
                                    <select id=\"d-periodo-tipo\"><option value=\"mes\">M&ecirc;s</option><option value=\"ano\">Ano</option></select>
                                </div>
                                <div class=\"field\"><label>M&ecirc;s/Ano</label><input type=\"month\" id=\"d-mes\"></div>
                                <div class=\"field\"><label>Categoria</label><select id=\"d-filtro-categoria\"></select></div>
                                <div class=\"field\"><label>Tipo</label>
                                    <select id=\"d-filtro-tipo\"><option value=\"todos\">Todos</option><option value=\"pessoal\">Pessoal</option><option value=\"profissional\">Profissional</option></select>
                                </div>
                            </div>
                            <div class=\"toolbar-actions\">
                                <button id=\"open-despesa-btn\" class=\"open-form-btn\">+ Nova despesa</button>
                                <button id=\"open-categorias-btn\" class=\"edit\">Categorias</button>
                            </div>
                        </div>

                        <div id=\"despesas-msg\" class=\"msg\"></div>

                        <div class=\"table-wrap\">
                            <table id=\"despesas-table\">
                                <thead><tr>
                                    <th>Data</th><th>Descri&ccedil;&atilde;o</th><th>Categoria</th><th>Tipo</th>
                                    <th>Valor</th><th>Status</th><th></th>
                                </tr></thead>
                                <tbody id=\"despesas-body\"></tbody>
                            </table>
                        </div>

                        <div class=\"recorrentes-head\">
                            <h2 class=\"section-title\">Despesas fixas (recorrentes)</h2>
                            <button id=\"open-recorrente-btn\" class=\"open-form-btn\">+ Nova recorrente</button>
                        </div>

                        <div class=\"table-wrap\">
                            <table id=\"recorrentes-table\">
                                <thead><tr>
                                    <th>Descri&ccedil;&atilde;o</th><th>Categoria</th><th>Tipo</th><th>Valor</th>
                                    <th>Periodicidade</th><th>Vencimento</th><th>Vig&ecirc;ncia</th><th>Ativo</th><th></th>
                                </tr></thead>
                                <tbody id=\"recorrentes-body\"></tbody>
                            </table>
                        </div>
                    </div>

                    <div id=\"despesa-modal\" class=\"modal-overlay\">
                        <div class=\"modal\">
                            <div class=\"modal-head\">
                                <h2 id=\"despesa-modal-title\">Nova despesa</h2>
                                <button id=\"close-despesa-btn\" class=\"modal-close\" aria-label=\"Fechar\">&times;</button>
                            </div>
                            <div class=\"form-card\">
                                <div class=\"field\"><label>Data</label><input type=\"date\" id=\"dx-data\"></div>
                                <div class=\"field\"><label>Valor (R$)</label><input type=\"number\" step=\"0.01\" id=\"dx-valor\"></div>
                                <div class=\"field wide\"><label>Descri&ccedil;&atilde;o</label><input type=\"text\" id=\"dx-descricao\" list=\"descricoes-list\"></div>
                                <div class=\"field\"><label>Categoria</label><select id=\"dx-categoria\"></select></div>
                                <div class=\"field\"><label>Tipo</label><select id=\"dx-tipo\"><option value=\"pessoal\">Pessoal</option><option value=\"profissional\">Profissional</option></select></div>
                                <div class=\"field\"><label>Status</label><select id=\"dx-status\"><option value=\"pago\">Pago</option><option value=\"previsto\">Previsto</option></select></div>
                                <div class=\"field wide\"><label>Observa&ccedil;&otilde;es</label><input type=\"text\" id=\"dx-observacoes\"></div>
                            </div>
                            <div class=\"modal-actions\">
                                <button id=\"cancel-despesa-btn\" class=\"edit\">Cancelar</button>
                                <button id=\"save-despesa-btn\">Salvar</button>
                            </div>
                        </div>
                    </div>
                    <datalist id=\"descricoes-list\"></datalist>

                    <div id=\"recorrente-modal\" class=\"modal-overlay\">
                        <div class=\"modal\">
                            <div class=\"modal-head\">
                                <h2 id=\"recorrente-modal-title\">Nova recorrente</h2>
                                <button id=\"close-recorrente-btn\" class=\"modal-close\" aria-label=\"Fechar\">&times;</button>
                            </div>
                            <div class=\"form-card\">
                                <div class=\"field wide\"><label>Descri&ccedil;&atilde;o</label><input type=\"text\" id=\"rx-descricao\" placeholder=\"Aluguel\"></div>
                                <div class=\"field\"><label>Valor (R$)</label><input type=\"number\" step=\"0.01\" id=\"rx-valor\"></div>
                                <div class=\"field\"><label>Categoria</label><select id=\"rx-categoria\"></select></div>
                                <div class=\"field\"><label>Tipo</label><select id=\"rx-tipo\"><option value=\"pessoal\">Pessoal</option><option value=\"profissional\">Profissional</option></select></div>
                                <div class=\"field\"><label>Periodicidade</label><select id=\"rx-periodicidade\"><option value=\"mensal\">Mensal</option><option value=\"anual\">Anual</option></select></div>
                                <div class=\"field\"><label>Dia de vencimento</label><input type=\"number\" min=\"1\" max=\"31\" id=\"rx-dia\" placeholder=\"5\"></div>
                                <div class=\"field\" id=\"rx-mes-field\" style=\"display: none;\"><label>M&ecirc;s (anual)</label>
                                    <select id=\"rx-mes\">
                                        <option value=\"1\">Janeiro</option><option value=\"2\">Fevereiro</option>
                                        <option value=\"3\">Mar&ccedil;o</option><option value=\"4\">Abril</option>
                                        <option value=\"5\">Maio</option><option value=\"6\">Junho</option>
                                        <option value=\"7\">Julho</option><option value=\"8\">Agosto</option>
                                        <option value=\"9\">Setembro</option><option value=\"10\">Outubro</option>
                                        <option value=\"11\">Novembro</option><option value=\"12\">Dezembro</option>
                                    </select>
                                </div>
                                <div class=\"field\"><label>In&iacute;cio</label><input type=\"date\" id=\"rx-inicio\"></div>
                                <div class=\"field\"><label>Fim (opcional)</label><input type=\"date\" id=\"rx-fim\"></div>
                                <div class=\"field checkbox\"><label>Ativo</label><input type=\"checkbox\" id=\"rx-ativo\" checked></div>
                            </div>
                            <div class=\"modal-actions\">
                                <button id=\"cancel-recorrente-btn\" class=\"edit\">Cancelar</button>
                                <button id=\"save-recorrente-btn\">Salvar</button>
                            </div>
                        </div>
                    </div>

                    <div id=\"categorias-modal\" class=\"modal-overlay\">
                        <div class=\"modal\">
                            <div class=\"modal-head\">
                                <h2>Categorias</h2>
                                <button id=\"close-categorias-btn\" class=\"modal-close\" aria-label=\"Fechar\">&times;</button>
                            </div>
                            <div class=\"cat-manage\">
                                <div class=\"cat-add\">
                                    <input type=\"text\" id=\"cat-novo\" placeholder=\"Nova categoria\">
                                    <button id=\"cat-add-btn\">Adicionar</button>
                                </div>
                                <ul id=\"cat-list\" class=\"cat-list\"></ul>
                            </div>
                        </div>
                    </div>
                </div>

                <script type=\"text/javascript\" src=\"/static/js/lucasodon.js\"></script>
            </body>
        </html>
    ", navbar())
}
