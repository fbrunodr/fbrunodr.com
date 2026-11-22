// Puericultura: acompanhamento das consultas de crianças até dois anos.
//
// Everything about a child is data and lives in a SQLite file inside bucket/, which is not in the repository;
// nothing here names a person. What is code is the rule: the ages at which a consultation is due, how the next
// return follows from the last consultation, and when a child counts as late.

use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use actix_web::cookie::{time::Duration as CookieDuration, Cookie, SameSite};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Result};
use rand::{distributions::Alphanumeric, Rng};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::components::navbar::navbar;

const DB_PATH: &str = "bucket/puericultura.db";
const PASSWORD_PATH: &str = "bucket/puericultura_password.txt";
const COOKIE_NAME: &str = "puericultura_sessao";
const SESSION_SECONDS: i64 = 30 * 24 * 60 * 60;

// The unit works on Brasília time, which has had no daylight saving since 2019: "today" is the date at UTC-3.
const UTC_OFFSET_SECONDS: i64 = -3 * 60 * 60;

// The ages, in months, at which a routine consultation is due, by the Sociedade Brasileira de Pediatria's routine:
// first or second week of life, monthly up to 6 months, every three months up to 2 years. 0 stands for the
// first one, due by the end of the second week. The checklists in the page are keyed by the same numbers.
const MARCOS: [i64; 13] = [0, 1, 2, 3, 4, 5, 6, 9, 12, 15, 18, 21, 24];
const PRIMEIRA_CONSULTA_DIAS: i64 = 14;

// A child is followed here until 2 years and 0 months: once 25 months are complete it leaves the list.
const LIMITE_MESES: i64 = 25;

// Gestational age is counted in days here: weeks and days as written on the birth record, 40 weeks being term.
const TERMO_DIAS: i64 = 40 * 7;
const PREMATURO_ABAIXO_DE_DIAS: i64 = 37 * 7;

const CREATE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS criancas (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        nome TEXT NOT NULL,
        nascimento TEXT NOT NULL,
        ig_semanas INTEGER,
        ig_dias INTEGER,
        acs TEXT NOT NULL,
        ultima_consulta TEXT,
        registro TEXT,
        peso_nascimento INTEGER
    );
    CREATE TABLE IF NOT EXISTS sessoes (
        token TEXT PRIMARY KEY,
        expira_em INTEGER NOT NULL
    );
";


// ----------------------------- Dates -----------------------------
//
// A date is the number of days since 1970-01-01, so subtracting two of them gives days. The two conversions are
// Howard Hinnant's civil-calendar algorithms.

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 29 } else { 28 },
    }
}

// Only a real date written exactly as YYYY-MM-DD passes.
fn parse_date(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    if !b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit()) {
        return None;
    }
    let y: i64 = s[0..4].parse().ok()?;
    let m: i64 = s[5..7].parse().ok()?;
    let d: i64 = s[8..10].parse().ok()?;
    if y < 1900 || !(1..=12).contains(&m) || d < 1 || d > days_in_month(y, m) {
        return None;
    }
    Some(days_from_civil(y, m, d))
}

fn format_date(day: i64) -> String {
    let (y, m, d) = civil_from_days(day);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

// The same day of the month n months later, or the month's last day when it has no such day (31 Jan + 1 = 28 Feb).
fn add_months(day: i64, n: i64) -> i64 {
    let (y, m, d) = civil_from_days(day);
    let total = y * 12 + (m - 1) + n;
    let (y2, m2) = (total.div_euclid(12), total.rem_euclid(12) + 1);
    days_from_civil(y2, m2, d.min(days_in_month(y2, m2)))
}

// Whole months and remaining days from one date to a later one: how an age is said.
fn months_and_days(from: i64, to: i64) -> (i64, i64) {
    let (y1, m1, _) = civil_from_days(from);
    let (y2, m2, _) = civil_from_days(to);
    let mut months = (y2 - y1) * 12 + (m2 - m1);
    if add_months(from, months) > to {
        months -= 1;
    }
    (months, to - add_months(from, months))
}

fn now_seconds() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

fn today() -> i64 {
    (now_seconds() + UTC_OFFSET_SECONDS).div_euclid(86400)
}


// ----------------------------- The rule -----------------------------

#[derive(Serialize, Debug, PartialEq)]
struct Idade {
    meses: i64,
    dias: i64,
}

// One consultation to make room for in the schedule.
#[derive(Serialize, Debug, PartialEq)]
struct Previsto {
    marco: i64,
    data: String,
    em_atraso: bool,
}

#[derive(Serialize, Debug, PartialEq)]
struct Calculo {
    idade: Idade,
    prematuro: bool,
    // None for a child born at term, and for a premature one that has not yet reached 40 weeks.
    idade_corrigida: Option<Idade>,
    // The age bracket the child is in: the last consultation age already reached (a key of MARCOS).
    faixa: i64,
    // The consultation the child is waited for next, and its date. None once the 24-month one is done.
    proxima_consulta: Option<i64>,
    proximo_retorno: Option<String>,
    atrasada: bool,
    dias_atraso: i64,
    // Every consultation still ahead of the child, in order: what the monthly schedule is counted from.
    agenda: Vec<Previsto>,
}

fn vencimento(nascimento: i64, marco: i64) -> i64 {
    if marco == 0 { nascimento + PRIMEIRA_CONSULTA_DIAS } else { add_months(nascimento, marco) }
}

// The scheduled consultation that follows one made on the given day: the one after the scheduled date the day
// fell closest to (the earlier one on a tie). None once that is the last.
fn seguinte(nascimento: i64, dia: i64) -> Option<usize> {
    let mut mais_perto = 0;
    for (i, &m) in MARCOS.iter().enumerate() {
        let d = (vencimento(nascimento, m) - dia).abs();
        if d < (vencimento(nascimento, MARCOS[mais_perto]) - dia).abs() {
            mais_perto = i;
        }
    }
    if mais_perto + 1 < MARCOS.len() { Some(mais_perto + 1) } else { None }
}

// The next return follows from the last consultation: that consultation counts as the scheduled one whose date
// it fell closest to (the earlier one on a tie), and the child is then waited for at the following one. A child
// with no consultation on record is waited for at the first. Late means the date of the next return has passed.
fn calcular(nascimento: i64, ig_em_dias: Option<i64>, ultima_consulta: Option<i64>, hoje: i64) -> Calculo {
    let (meses, dias) = months_and_days(nascimento, hoje);

    let prematuro = ig_em_dias.map_or(false, |ig| ig < PREMATURO_ABAIXO_DE_DIAS);
    let idade_corrigida = match ig_em_dias {
        Some(ig) if prematuro => {
            let termo = nascimento + (TERMO_DIAS - ig);
            if hoje >= termo {
                let (m, d) = months_and_days(termo, hoje);
                Some(Idade { meses: m, dias: d })
            } else {
                None
            }
        }
        _ => None,
    };

    let faixa = MARCOS.iter().rev().copied().find(|&m| vencimento(nascimento, m) <= hoje).unwrap_or(0);

    let proximo_indice = match ultima_consulta {
        None => Some(0),
        Some(ultima) => seguinte(nascimento, ultima),
    };
    let proxima_consulta = proximo_indice.map(|i| MARCOS[i]);
    let retorno = proxima_consulta.map(|m| vencimento(nascimento, m));
    let dias_atraso = retorno.map_or(0, |r| (hoje - r).max(0));

    // A late child needs one consultation now, whatever it missed, at the age it has; from then on the
    // schedule goes on as if that consultation were made today. A child on time just follows the schedule.
    let mut agenda = Vec::new();
    let a_partir_de = if dias_atraso > 0 {
        agenda.push(Previsto { marco: faixa, data: format_date(hoje), em_atraso: true });
        seguinte(nascimento, hoje)
    } else {
        proximo_indice
    };
    if let Some(i) = a_partir_de {
        for &m in &MARCOS[i..] {
            agenda.push(Previsto { marco: m, data: format_date(vencimento(nascimento, m)), em_atraso: false });
        }
    }

    Calculo {
        idade: Idade { meses, dias },
        prematuro,
        idade_corrigida,
        faixa,
        proxima_consulta,
        proximo_retorno: retorno.map(format_date),
        atrasada: dias_atraso > 0,
        dias_atraso,
        agenda,
    }
}


// ----------------------------- Supplementation -----------------------------
//
// The unit's protocol. Vitamin D for every child, daily, from birth to 2 years: 400 UI up to 1 year, 600 UI from
// 1 to 2. Iron depends on how the child was born:
//   - at term (37 weeks or more) with 2,500 g or more: a fixed daily dose in two cycles, from 6 to 9 months and
//     from 12 to 15, with a pause between them;
//   - premature and/or under 2,500 g: from 30 days of life with no pause, by birth weight in the first year
//     (2 mg/kg/day from 1,500 g, 3 from 1,000 g, 4 below that) and 1 mg/kg/day in the second.
// The protocol's table stops at 2,500 g; a premature child born heavier is given the 2 mg/kg/day of its top row.

const BAIXO_PESO_ABAIXO_DE: i64 = 2500;

#[derive(Serialize, Debug, PartialEq)]
struct Ferro {
    // "termo" or "risco" (premature and/or low birth weight).
    grupo: &'static str,
    // termo: "antes", "ciclo1", "pausa", "ciclo2", "encerrada". risco: "antes", "primeiro_ano", "segundo_ano".
    fase: &'static str,
    // risco only: the dose now (the one to start with, while "antes"). None while the birth weight is unknown.
    mg_kg: Option<i64>,
    // The day this phase ends and the next begins.
    ate: Option<String>,
}

#[derive(Serialize, Debug, PartialEq)]
struct Suplementacao {
    vitamina_d_ui: i64,
    // None while what is known of the birth does not say which group the child is in.
    ferro: Option<Ferro>,
}

fn suplementacao(nascimento: i64, ig_em_dias: Option<i64>, peso_nascimento: Option<i64>, hoje: i64) -> Suplementacao {
    let meses = months_and_days(nascimento, hoje).0;
    let prematuro = ig_em_dias.map(|ig| ig < PREMATURO_ABAIXO_DE_DIAS);
    let baixo_peso = peso_nascimento.map(|p| p < BAIXO_PESO_ABAIXO_DE);
    let ate = |m: i64| Some(format_date(add_months(nascimento, m)));

    let ferro = if prematuro == Some(true) || baixo_peso == Some(true) {
        let primeiro_ano = peso_nascimento.map(|p| if p < 1000 { 4 } else if p < 1500 { 3 } else { 2 });
        let inicio = nascimento + 30;
        Some(if hoje < inicio {
            Ferro { grupo: "risco", fase: "antes", mg_kg: primeiro_ano, ate: Some(format_date(inicio)) }
        } else if meses < 12 {
            Ferro { grupo: "risco", fase: "primeiro_ano", mg_kg: primeiro_ano, ate: ate(12) }
        } else {
            Ferro { grupo: "risco", fase: "segundo_ano", mg_kg: Some(1), ate: ate(24) }
        })
    } else if prematuro == Some(false) && baixo_peso == Some(false) {
        let (fase, fim) = match meses {
            m if m < 6 => ("antes", ate(6)),
            m if m < 9 => ("ciclo1", ate(9)),
            m if m < 12 => ("pausa", ate(12)),
            m if m < 15 => ("ciclo2", ate(15)),
            _ => ("encerrada", None),
        };
        Some(Ferro { grupo: "termo", fase, mg_kg: None, ate: fim })
    } else {
        None
    };

    Suplementacao { vitamina_d_ui: if meses < 12 { 400 } else { 600 }, ferro }
}


// Weeks and days as one number of days; no weeks, no gestational age.
fn ig_em_dias(semanas: Option<i64>, dias: Option<i64>) -> Option<i64> {
    semanas.map(|s| s * 7 + dias.unwrap_or(0))
}


// ----------------------------- Data models -----------------------------

#[derive(Deserialize)]
struct CriancaInput {
    registro: Option<String>,
    nome: String,
    nascimento: String,
    ig_semanas: Option<i64>,
    ig_dias: Option<i64>,
    peso_nascimento: Option<i64>,
    acs: String,
    ultima_consulta: Option<String>,
}

#[derive(Deserialize)]
struct CriancaUpdate {
    id: i64,
    #[serde(flatten)]
    fields: CriancaInput,
}

#[derive(Deserialize)]
struct IdOnly {
    id: i64,
}

#[derive(Deserialize)]
struct ConsultaInput {
    id: i64,
    data: String,
}

#[derive(Deserialize)]
struct LoginForm {
    password: String,
}

// A child as checked: the dates are day numbers and the texts are trimmed.
#[derive(Debug, PartialEq)]
struct Crianca {
    // The child's registration number at the unit, kept as text: it is a label, not a quantity.
    registro: Option<String>,
    nome: String,
    nascimento: i64,
    ig_semanas: Option<i64>,
    ig_dias: Option<i64>,
    // Birth weight in grams.
    peso_nascimento: Option<i64>,
    acs: String,
    ultima_consulta: Option<i64>,
}

#[derive(Serialize)]
struct CriancaView {
    id: i64,
    registro: Option<String>,
    nome: String,
    nascimento: String,
    ig_semanas: Option<i64>,
    ig_dias: Option<i64>,
    peso_nascimento: Option<i64>,
    acs: String,
    ultima_consulta: Option<String>,
    #[serde(flatten)]
    calculo: Calculo,
    suplementacao: Suplementacao,
}

#[derive(Serialize)]
struct Lista {
    hoje: String,
    criancas: Vec<CriancaView>,
}

fn texto(valor: &str, campo: &str, max: usize) -> std::result::Result<String, String> {
    let v = valor.trim();
    if v.is_empty() {
        return Err(format!("{}: preencha este campo", campo));
    }
    if v.chars().count() > max || v.chars().any(char::is_control) {
        return Err(format!("{}: texto inválido ou longo demais", campo));
    }
    Ok(v.to_string())
}

fn validar_consulta(data: &str, nascimento: i64, hoje: i64) -> std::result::Result<i64, String> {
    let dia = parse_date(data).ok_or("Data da consulta: data inválida")?;
    if dia < nascimento {
        return Err("Data da consulta: não pode ser anterior ao nascimento".to_string());
    }
    if dia > hoje {
        return Err("Data da consulta: não pode ser no futuro".to_string());
    }
    Ok(dia)
}

fn validar(input: &CriancaInput, hoje: i64) -> std::result::Result<Crianca, String> {
    let registro = match input.registro.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(r) if r.len() <= 15 && r.bytes().all(|b| b.is_ascii_digit()) => Some(r.to_string()),
        Some(_) => return Err("Nº de registro: só números, até 15 dígitos".to_string()),
    };
    let nome = texto(&input.nome, "Nome", 120)?;
    // The ACS is what the list is filtered by, so one person must not become two by a difference of case.
    let acs = texto(&input.acs, "ACS", 60)?.to_uppercase();

    let nascimento = parse_date(&input.nascimento).ok_or("Data de nascimento: data inválida")?;
    if nascimento > hoje {
        return Err("Data de nascimento: não pode ser no futuro".to_string());
    }
    if months_and_days(nascimento, hoje).0 >= LIMITE_MESES {
        return Err("Data de nascimento: o acompanhamento é só de crianças até dois anos".to_string());
    }

    // The days only mean something beside the weeks; with the weeks given, no days is 0 days.
    let ig_dias = match (input.ig_semanas, input.ig_dias) {
        (None, None) => None,
        (None, Some(_)) => return Err("Idade gestacional: informe as semanas além dos dias".to_string()),
        (Some(semanas), dias) => {
            if !(20..=44).contains(&semanas) {
                return Err("Idade gestacional: as semanas devem estar entre 20 e 44".to_string());
            }
            let dias = dias.unwrap_or(0);
            if !(0..=6).contains(&dias) {
                return Err("Idade gestacional: os dias devem estar entre 0 e 6".to_string());
            }
            Some(dias)
        }
    };

    if let Some(peso) = input.peso_nascimento {
        if !(300..=7000).contains(&peso) {
            return Err("Peso ao nascer: informe em gramas, entre 300 e 7000".to_string());
        }
    }

    let ultima_consulta = match input.ultima_consulta.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(data) => Some(validar_consulta(data, nascimento, hoje)?),
    };

    Ok(Crianca { registro, nome, nascimento, ig_semanas: input.ig_semanas, ig_dias, peso_nascimento: input.peso_nascimento, acs, ultima_consulta })
}


// ----------------------------- Database -----------------------------

fn open_db() -> rusqlite::Result<Connection> {
    let conn = Connection::open(DB_PATH)?;
    // Like the password, the records are for the server's user alone.
    let _ = fs::set_permissions(DB_PATH, fs::Permissions::from_mode(0o600));
    conn.execute_batch(CREATE_SQL)?;
    // A database from before a column existed gets it; the children already there keep what they had.
    for (coluna, tipo) in [("ig_dias", "INTEGER"), ("registro", "TEXT"), ("peso_nascimento", "INTEGER")] {
        let existe: i64 = conn.query_row("SELECT COUNT(*) FROM pragma_table_info('criancas') WHERE name = ?1", params![coluna], |r| r.get(0))?;
        if existe == 0 {
            conn.execute_batch(&format!("ALTER TABLE criancas ADD COLUMN {} {}", coluna, tipo))?;
        }
    }
    Ok(conn)
}

// Rows whose dates cannot be read are left out rather than failing the whole list; nothing the page writes is such a row.
fn list_criancas(hoje: i64) -> rusqlite::Result<Vec<CriancaView>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare("SELECT id, nome, nascimento, ig_semanas, ig_dias, acs, ultima_consulta, registro, peso_nascimento FROM criancas ORDER BY nome ASC")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<i64>>(3)?,
            r.get::<_, Option<i64>>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, Option<String>>(6)?,
            r.get::<_, Option<String>>(7)?,
            r.get::<_, Option<i64>>(8)?,
        ))
    })?;

    let mut criancas = Vec::new();
    for row in rows {
        let (id, nome, nascimento_texto, ig_semanas, ig_dias, acs, ultima_texto, registro, peso_nascimento) = row?;
        let nascimento = match parse_date(&nascimento_texto) {
            Some(d) => d,
            None => continue,
        };
        if nascimento > hoje || months_and_days(nascimento, hoje).0 >= LIMITE_MESES {
            continue;
        }
        let ultima = ultima_texto.as_deref().and_then(parse_date);
        criancas.push(CriancaView {
            id,
            registro,
            nome,
            nascimento: nascimento_texto,
            ig_semanas,
            ig_dias: ig_semanas.map(|_| ig_dias.unwrap_or(0)),
            acs,
            ultima_consulta: ultima.map(format_date),
            peso_nascimento,
            calculo: calcular(nascimento, ig_em_dias(ig_semanas, ig_dias), ultima, hoje),
            suplementacao: suplementacao(nascimento, ig_em_dias(ig_semanas, ig_dias), peso_nascimento, hoje),
        });
    }
    Ok(criancas)
}

fn insert_crianca(c: &Crianca) -> rusqlite::Result<()> {
    let conn = open_db()?;
    conn.execute(
        "INSERT INTO criancas (nome, nascimento, ig_semanas, ig_dias, acs, ultima_consulta, registro, peso_nascimento) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![c.nome, format_date(c.nascimento), c.ig_semanas, c.ig_dias, c.acs, c.ultima_consulta.map(format_date), c.registro, c.peso_nascimento],
    )?;
    Ok(())
}

fn update_crianca(id: i64, c: &Crianca) -> rusqlite::Result<usize> {
    let conn = open_db()?;
    conn.execute(
        "UPDATE criancas SET nome = ?1, nascimento = ?2, ig_semanas = ?3, ig_dias = ?4, acs = ?5, ultima_consulta = ?6, registro = ?7, peso_nascimento = ?8 WHERE id = ?9",
        params![c.nome, format_date(c.nascimento), c.ig_semanas, c.ig_dias, c.acs, c.ultima_consulta.map(format_date), c.registro, c.peso_nascimento, id],
    )
}

// A registration number names one child: true when another child (not the one being edited) already has it.
fn registro_em_uso(registro: &Option<String>, exceto: Option<i64>) -> rusqlite::Result<bool> {
    let registro = match registro {
        Some(r) => r,
        None => return Ok(false),
    };
    let conn = open_db()?;
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM criancas WHERE registro = ?1 AND id IS NOT ?2",
        params![registro, exceto],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}

fn delete_crianca(id: i64) -> rusqlite::Result<usize> {
    let conn = open_db()?;
    conn.execute("DELETE FROM criancas WHERE id = ?1", params![id])
}

fn nascimento_de(id: i64) -> rusqlite::Result<Option<i64>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare("SELECT nascimento FROM criancas WHERE id = ?1")?;
    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(r) => Ok(parse_date(&r.get::<_, String>(0)?)),
        None => Ok(None),
    }
}

fn registrar_consulta(id: i64, dia: i64) -> rusqlite::Result<usize> {
    let conn = open_db()?;
    conn.execute("UPDATE criancas SET ultima_consulta = ?1 WHERE id = ?2", params![format_date(dia), id])
}


// ----------------------------- Auth -----------------------------
//
// The records are of real children, so there is no default password: the first run draws one at random and
// leaves it in bucket/, readable by the server's user only. Logging in trades it for a random session token,
// and the cookie carries the token, never the password.

fn random_text(length: usize) -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(length).map(char::from).collect()
}

fn get_password() -> Option<String> {
    if let Ok(p) = fs::read_to_string(PASSWORD_PATH) {
        let p = p.trim().to_string();
        return if p.is_empty() { None } else { Some(p) };
    }
    let password = random_text(20);
    let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(PASSWORD_PATH).ok()?;
    file.write_all(password.as_bytes()).ok()?;
    Some(password)
}

// Compares every byte whatever the first difference is, so the time taken says nothing about the password.
fn same_text(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn create_session() -> rusqlite::Result<String> {
    let conn = open_db()?;
    let agora = now_seconds();
    conn.execute("DELETE FROM sessoes WHERE expira_em <= ?1", params![agora])?;
    let token = random_text(48);
    conn.execute("INSERT INTO sessoes (token, expira_em) VALUES (?1, ?2)", params![token, agora + SESSION_SECONDS])?;
    Ok(token)
}

fn is_authed(req: &HttpRequest) -> bool {
    let token = match req.cookie(COOKIE_NAME) {
        Some(c) => c.value().to_string(),
        None => return false,
    };
    let found = open_db().and_then(|conn| {
        conn.query_row(
            "SELECT COUNT(*) FROM sessoes WHERE token = ?1 AND expira_em > ?2",
            params![token, now_seconds()],
            |r| r.get::<_, i64>(0),
        )
    });
    matches!(found, Ok(n) if n > 0)
}

fn session_cookie(req: &HttpRequest, value: String) -> Cookie<'static> {
    Cookie::build(COOKIE_NAME, value)
        .path("/puericultura")
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(req.connection_info().scheme() == "https")
        .max_age(CookieDuration::seconds(SESSION_SECONDS))
        .finish()
}


// ----------------------------- Handlers -----------------------------

const REGISTRO_REPETIDO: &str = "Nº de registro: já existe outra criança com este número";

fn nao_autorizado() -> HttpResponse {
    HttpResponse::Unauthorized().body("não autorizado")
}

fn invalido(mensagem: String) -> HttpResponse {
    HttpResponse::BadRequest().content_type("text/plain; charset=utf-8").body(mensagem)
}

// What went wrong in the database is for the server's log, not for whoever is asking.
fn erro_interno(e: rusqlite::Error) -> HttpResponse {
    eprintln!("puericultura: {}", e);
    HttpResponse::InternalServerError().body("erro interno")
}

fn redirect(to: &str) -> actix_web::HttpResponseBuilder {
    let mut builder = HttpResponse::SeeOther();
    builder.append_header(("Location", to.to_string()));
    builder
}

#[post("/puericultura/login")]
async fn login(req: HttpRequest, form: web::Form<LoginForm>) -> impl Responder {
    let certo = get_password().map_or(false, |p| same_text(form.password.trim(), &p));
    if !certo {
        // A wrong guess costs a second, which is what makes guessing in bulk slow.
        actix_web::rt::time::sleep(Duration::from_secs(1)).await;
        return redirect("/puericultura?erro=1").finish();
    }
    match create_session() {
        Ok(token) => redirect("/puericultura").cookie(session_cookie(&req, token)).finish(),
        Err(e) => erro_interno(e),
    }
}

#[post("/puericultura/logout")]
async fn logout(req: HttpRequest) -> impl Responder {
    if let Some(c) = req.cookie(COOKIE_NAME) {
        let _ = open_db().and_then(|conn| conn.execute("DELETE FROM sessoes WHERE token = ?1", params![c.value()]));
    }
    let mut cookie = session_cookie(&req, String::new());
    cookie.make_removal();
    redirect("/puericultura").cookie(cookie).finish()
}

#[get("/puericultura/api/criancas")]
async fn list(req: HttpRequest) -> impl Responder {
    if !is_authed(&req) { return nao_autorizado(); }
    let hoje = today();
    match list_criancas(hoje) {
        Ok(criancas) => HttpResponse::Ok().json(Lista { hoje: format_date(hoje), criancas }),
        Err(e) => erro_interno(e),
    }
}

// The form shows the corrected age and the next return while it is being filled: the same rule, nothing saved.
#[post("/puericultura/api/calcular")]
async fn preview(req: HttpRequest, input: web::Json<CriancaInput>) -> impl Responder {
    if !is_authed(&req) { return nao_autorizado(); }
    let hoje = today();
    match validar(&input, hoje) {
        Ok(c) => HttpResponse::Ok().json(calcular(c.nascimento, ig_em_dias(c.ig_semanas, c.ig_dias), c.ultima_consulta, hoje)),
        Err(m) => invalido(m),
    }
}

#[post("/puericultura/api/criancas/create")]
async fn create(req: HttpRequest, input: web::Json<CriancaInput>) -> impl Responder {
    if !is_authed(&req) { return nao_autorizado(); }
    let crianca = match validar(&input, today()) {
        Ok(c) => c,
        Err(m) => return invalido(m),
    };
    match registro_em_uso(&crianca.registro, None) {
        Ok(false) => (),
        Ok(true) => return invalido(REGISTRO_REPETIDO.to_string()),
        Err(e) => return erro_interno(e),
    }
    match insert_crianca(&crianca) {
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => erro_interno(e),
    }
}

#[post("/puericultura/api/criancas/update")]
async fn update(req: HttpRequest, input: web::Json<CriancaUpdate>) -> impl Responder {
    if !is_authed(&req) { return nao_autorizado(); }
    let crianca = match validar(&input.fields, today()) {
        Ok(c) => c,
        Err(m) => return invalido(m),
    };
    match registro_em_uso(&crianca.registro, Some(input.id)) {
        Ok(false) => (),
        Ok(true) => return invalido(REGISTRO_REPETIDO.to_string()),
        Err(e) => return erro_interno(e),
    }
    match update_crianca(input.id, &crianca) {
        Ok(0) => HttpResponse::NotFound().body("criança não encontrada"),
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => erro_interno(e),
    }
}

#[post("/puericultura/api/criancas/delete")]
async fn delete(req: HttpRequest, input: web::Json<IdOnly>) -> impl Responder {
    if !is_authed(&req) { return nao_autorizado(); }
    match delete_crianca(input.id) {
        Ok(0) => HttpResponse::NotFound().body("criança não encontrada"),
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => erro_interno(e),
    }
}

#[post("/puericultura/api/criancas/consulta")]
async fn consulta(req: HttpRequest, input: web::Json<ConsultaInput>) -> impl Responder {
    if !is_authed(&req) { return nao_autorizado(); }
    let nascimento = match nascimento_de(input.id) {
        Ok(Some(n)) => n,
        Ok(None) => return HttpResponse::NotFound().body("criança não encontrada"),
        Err(e) => return erro_interno(e),
    };
    let dia = match validar_consulta(input.data.trim(), nascimento, today()) {
        Ok(d) => d,
        Err(m) => return invalido(m),
    };
    match registrar_consulta(input.id, dia) {
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => erro_interno(e),
    }
}

#[get("/puericultura")]
pub async fn render(req: HttpRequest) -> Result<HttpResponse> {
    let body = if is_authed(&req) { app_page() } else { login_page(req.query_string() == "erro=1") };
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        // The page is the door to patient records: no cache keeps a copy of it.
        .insert_header(("Cache-Control", "no-store"))
        .body(body))
}


// ----------------------------- Views -----------------------------

const HEAD: &str = r#"
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="robots" content="noindex, nofollow" />
    <title>Puericultura</title>
    <link type="text/css" rel="stylesheet" href="/static/css/index.css">
    <link href="https://fonts.googleapis.com/css2?family=Open+Sans:wght@300;400;600;700&family=Reenie+Beanie&display=swap" rel="stylesheet">
    <link type="text/css" rel="stylesheet" href="/static/css/puericultura.css">
"#;

fn login_page(erro: bool) -> String {
    let aviso = if erro { r#"<p class="login-erro" role="alert">Senha incorreta.</p>"# } else { "" };
    format!(r#"<!DOCTYPE html>
        <html lang="pt-br">
            <head>{}</head>
            <body>
                {}
                <div class="content pk-login">
                    <h1 class="pk-title">Puericultura</h1>
                    <form class="login-card" method="POST" action="/puericultura/login">
                        <label for="password">Senha</label>
                        <input type="password" id="password" name="password" autocomplete="current-password" autofocus required>
                        {}
                        <button type="submit" class="btn primary">Entrar</button>
                    </form>
                </div>
            </body>
        </html>
    "#, HEAD, navbar(), aviso)
}

fn app_page() -> String {
    format!(r#"<!DOCTYPE html>
        <html lang="pt-br">
            <head>{}</head>
            <body>
                {}
                <div class="content pk-app">
                    <div class="pk-header">
                        <div>
                            <h1 class="pk-title">Puericultura</h1>
                            <p class="pk-sub">Consultas de crian&ccedil;as at&eacute; dois anos</p>
                        </div>
                        <form method="POST" action="/puericultura/logout"><button type="submit" class="btn link">Sair</button></form>
                    </div>

                    <div id="resumo" class="resumo"></div>

                    <div class="abas" role="tablist">
                        <button id="aba-lista" type="button" class="aba ativa" role="tab" aria-selected="true" aria-controls="sec-lista">Crian&ccedil;as</button>
                        <button id="aba-agenda" type="button" class="aba" role="tab" aria-selected="false" aria-controls="sec-agenda">Agenda de consultas</button>
                        <button id="aba-prematuros" type="button" class="aba" role="tab" aria-selected="false" aria-controls="sec-prematuros">Prematuros</button>
                    </div>

                    <div id="msg" class="msg" role="status"></div>

                    <div id="sec-lista" role="tabpanel" aria-labelledby="aba-lista">
                    <div class="barra">
                        <button id="novo-btn" type="button" class="btn primary">+ Cadastrar crian&ccedil;a</button>
                        <div class="filtros">
                            <input type="search" id="busca" placeholder="Buscar por nome ou registro" aria-label="Buscar por nome ou n&uacute;mero de registro">
                            <select id="filtro-acs" aria-label="Filtrar por ACS"></select>
                            <button id="filtro-atrasadas" type="button" class="btn toggle" aria-pressed="false">S&oacute; atrasadas</button>
                        </div>
                    </div>

                    <ul id="lista" class="lista"></ul>
                    <p id="vazio" class="vazio" hidden></p>
                    </div>

                    <div id="sec-agenda" role="tabpanel" aria-labelledby="aba-agenda" hidden>
                        <p class="nota">Quantas vagas de puericultura abrir em cada m&ecirc;s: as consultas previstas pela idade de cada crian&ccedil;a, mais uma consulta agora para cada crian&ccedil;a atrasada. Toque no m&ecirc;s para ver quem.</p>
                        <div id="agenda"></div>
                    </div>

                    <div id="sec-prematuros" role="tabpanel" aria-labelledby="aba-prematuros" hidden>
                        <p class="nota">Crian&ccedil;as nascidas com menos de 37 semanas. A idade corrigida desconta o tempo que faltou para completar 40 semanas e vale para avaliar crescimento e desenvolvimento; as vacinas seguem a idade cronol&oacute;gica.</p>
                        <ul id="prematuros" class="lista"></ul>
                    </div>
                </div>

                <div id="form-modal" class="modal-overlay" hidden>
                    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="form-titulo">
                        <div class="modal-head">
                            <h2 id="form-titulo">Cadastrar crian&ccedil;a</h2>
                            <button type="button" class="modal-close" data-fechar aria-label="Fechar">&times;</button>
                        </div>
                        <form id="form" class="form-card" novalidate>
                            <div class="field"><label for="f-registro">N&ordm; de registro (ID)</label><input type="text" id="f-registro" maxlength="15" inputmode="numeric" pattern="[0-9]*" autocomplete="off"></div>
                            <div class="field wide"><label for="f-nome">Nome da crian&ccedil;a</label><input type="text" id="f-nome" maxlength="120" autocomplete="off" required></div>
                            <div class="field"><label for="f-nascimento">Data de nascimento</label><input type="date" id="f-nascimento" required></div>
                            <fieldset class="field ig">
                                <legend>Idade gestacional ao nascer</legend>
                                <label><input type="number" id="f-ig" min="20" max="44" step="1" inputmode="numeric" placeholder="39" aria-label="Semanas"><span>semanas</span></label>
                                <label><input type="number" id="f-ig-dias" min="0" max="6" step="1" inputmode="numeric" placeholder="0" aria-label="Dias"><span>dias</span></label>
                            </fieldset>
                            <div class="field"><label for="f-peso">Peso ao nascer (gramas)</label><input type="number" id="f-peso" min="300" max="7000" step="1" inputmode="numeric" placeholder="ex.: 3200"></div>
                            <div class="field"><label for="f-acs">ACS respons&aacute;vel</label><input type="text" id="f-acs" maxlength="60" list="acs-list" autocomplete="off" required></div>
                            <div class="field"><label for="f-ultima">Data da &uacute;ltima consulta</label><input type="date" id="f-ultima"></div>
                            <datalist id="acs-list"></datalist>
                            <div id="previa" class="previa wide" aria-live="polite"></div>
                            <div id="form-msg" class="msg err wide" role="alert"></div>
                            <div class="modal-actions wide">
                                <button type="button" class="btn" data-fechar>Cancelar</button>
                                <button type="submit" class="btn primary">Salvar</button>
                            </div>
                        </form>
                    </div>
                </div>

                <div id="ficha-modal" class="modal-overlay" hidden>
                    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="ficha-nome">
                        <div class="modal-head">
                            <h2 id="ficha-nome"></h2>
                            <button type="button" class="modal-close" data-fechar aria-label="Fechar">&times;</button>
                        </div>
                        <div id="ficha-corpo"></div>
                    </div>
                </div>

                <script type="text/javascript" src="/static/js/puericultura.js"></script>
            </body>
        </html>
    "#, HEAD, navbar())
}


#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> i64 {
        parse_date(s).unwrap()
    }

    fn input(nome: &str, nascimento: &str, ig: Option<i64>, acs: &str, ultima: Option<&str>) -> CriancaInput {
        CriancaInput {
            registro: None,
            nome: nome.to_string(),
            nascimento: nascimento.to_string(),
            ig_semanas: ig,
            ig_dias: None,
            peso_nascimento: None,
            acs: acs.to_string(),
            ultima_consulta: ultima.map(str::to_string),
        }
    }

    #[test]
    fn a_date_survives_the_round_trip() {
        assert_eq!(d("1970-01-01"), 0);
        assert_eq!(d("2000-03-01") - d("2000-02-28"), 2);
        assert_eq!(d("2100-03-01") - d("2100-02-28"), 1);
        for s in ["1970-01-01", "1999-12-31", "2024-02-29", "2026-11-22", "2100-03-01"] {
            assert_eq!(format_date(d(s)), s);
        }
    }

    #[test]
    fn only_a_real_date_passes() {
        for bad in ["", "2026-02-30", "2025-02-29", "2026-13-01", "2026-00-10", "2026-01-00", "26-01-01", "2026/01/01",
            "2026-1-1", "2026-01-01 ", " 2026-01-01", "2026-01-0a", "+026-01-01", "2026-01-01T00", "01/02/2026", "２０２６-01-01"] {
            assert_eq!(parse_date(bad), None, "{:?} must not pass", bad);
        }
    }

    #[test]
    fn a_month_later_stays_inside_the_month() {
        assert_eq!(add_months(d("2026-01-31"), 1), d("2026-02-28"));
        assert_eq!(add_months(d("2024-01-31"), 1), d("2024-02-29"));
        assert_eq!(add_months(d("2025-08-31"), 6), d("2026-02-28"));
        assert_eq!(add_months(d("2025-11-15"), 2), d("2026-01-15"));
        assert_eq!(add_months(d("2024-02-29"), 24), d("2026-02-28"));
    }

    #[test]
    fn an_age_is_whole_months_and_the_days_left() {
        assert_eq!(months_and_days(d("2026-01-15"), d("2026-01-15")), (0, 0));
        assert_eq!(months_and_days(d("2026-01-15"), d("2026-02-14")), (0, 30));
        assert_eq!(months_and_days(d("2026-01-15"), d("2026-02-15")), (1, 0));
        assert_eq!(months_and_days(d("2024-11-08"), d("2026-11-22")), (24, 14));
        assert_eq!(months_and_days(d("2026-01-31"), d("2026-02-28")), (1, 0));
        assert_eq!(months_and_days(d("2026-01-31"), d("2026-03-01")), (1, 1));
    }

    #[test]
    fn with_no_consultation_the_first_week_is_what_is_due() {
        let nasc = d("2026-11-01");
        let c = calcular(nasc, None, None, d("2026-11-05"));
        assert_eq!((c.proxima_consulta, c.proximo_retorno.as_deref(), c.atrasada), (Some(0), Some("2026-11-15"), false));
        // On the day itself the child is still on time; the day after, late.
        assert!(!calcular(nasc, None, None, d("2026-11-15")).atrasada);
        let c = calcular(nasc, None, None, d("2026-11-22"));
        assert_eq!((c.atrasada, c.dias_atraso), (true, 7));
    }

    #[test]
    fn the_next_return_follows_the_consultation_it_was_closest_to() {
        let nasc = d("2026-01-10");
        let proxima = |ultima: &str| {
            let c = calcular(nasc, None, Some(d(ultima)), d("2026-01-10"));
            (c.proxima_consulta, c.proximo_retorno)
        };
        // Seen in the first week: that was the first consultation, and the next is at one month.
        assert_eq!(proxima("2026-01-16"), (Some(1), Some("2026-02-10".to_string())));
        assert_eq!(proxima("2026-02-10"), (Some(2), Some("2026-03-10".to_string())));
        // Monthly up to six months, then every three.
        assert_eq!(proxima("2026-06-10"), (Some(6), Some("2026-07-10".to_string())));
        assert_eq!(proxima("2026-07-10"), (Some(9), Some("2026-10-10".to_string())));
        // Seen closer to four months than to three: the next is at five.
        assert_eq!(proxima("2026-04-28"), (Some(5), Some("2026-06-10".to_string())));
        // Exactly between three and four months counts as the earlier one.
        assert_eq!(proxima("2026-04-25"), (Some(4), Some("2026-05-10".to_string())));
        assert_eq!(proxima("2027-07-10"), (Some(21), Some("2027-10-10".to_string())));
        // After the 24-month consultation nothing more is due here.
        assert_eq!(proxima("2028-01-10"), (None, None));
    }

    #[test]
    fn the_next_return_is_never_before_the_last_consultation() {
        let nasc = d("2025-03-31");
        for ultima in nasc..add_months(nasc, 26) {
            let c = calcular(nasc, None, Some(ultima), ultima);
            if let Some(retorno) = c.proximo_retorno {
                assert!(d(&retorno) > ultima, "seen on {} and waited for on {}", format_date(ultima), retorno);
                assert!(!c.atrasada);
            }
        }
    }

    #[test]
    fn late_is_when_the_return_date_has_passed() {
        let nasc = d("2026-01-10");
        let ultima = Some(d("2026-03-10"));
        assert!(!calcular(nasc, None, ultima, d("2026-04-10")).atrasada);
        let c = calcular(nasc, None, ultima, d("2026-04-20"));
        assert_eq!((c.atrasada, c.dias_atraso, c.proxima_consulta), (true, 10, Some(3)));
        // With the 24-month consultation done the child is never late.
        assert!(!calcular(nasc, None, Some(d("2028-01-10")), d("2028-02-05")).atrasada);
    }

    #[test]
    fn the_schedule_is_every_consultation_still_ahead() {
        let nasc = d("2026-01-10");
        let agenda = |ultima: Option<&str>, hoje: &str| -> Vec<(i64, String, bool)> {
            calcular(nasc, None, ultima.map(d), d(hoje)).agenda.into_iter().map(|p| (p.marco, p.data, p.em_atraso)).collect()
        };
        let s = |m: i64, data: &str, atraso: bool| (m, data.to_string(), atraso);

        let resto = vec![s(15, "2027-04-10", false), s(18, "2027-07-10", false), s(21, "2027-10-10", false), s(24, "2028-01-10", false)];
        let com = |antes: Vec<(i64, String, bool)>| [antes, resto.clone()].concat();

        // On time after the 9-month consultation: one every three months is ahead, up to 24.
        assert_eq!(agenda(Some("2026-10-10"), "2026-11-22"), com(vec![s(12, "2027-01-10", false)]));
        // Never seen and ten months old: one consultation now, at the age it has, and not the eight it missed.
        assert_eq!(agenda(None, "2026-11-22"), com(vec![s(9, "2026-11-22", true), s(12, "2027-01-10", false)]));
        // Late with the 12-month date three days away: seen now, that is the 12-month consultation, not two.
        assert_eq!(agenda(Some("2026-07-10"), "2027-01-07"), com(vec![s(9, "2027-01-07", true)]));
        // A newborn: all thirteen consultations, six of them monthly.
        let todas = agenda(None, "2026-01-10");
        assert_eq!(todas.len(), 13);
        assert_eq!(todas[0], s(0, "2026-01-24", false));
        assert_eq!(todas[6], s(6, "2026-07-10", false));
        // Late for the last one: only the consultation now.
        assert_eq!(agenda(Some("2027-10-10"), "2028-01-20"), vec![s(24, "2028-01-20", true)]);
        assert_eq!(agenda(Some("2028-01-10"), "2028-01-20"), vec![]);
    }

    #[test]
    fn the_bracket_is_the_last_consultation_age_reached() {
        let nasc = d("2026-01-10");
        let faixa = |hoje: &str| calcular(nasc, None, None, d(hoje)).faixa;
        assert_eq!(faixa("2026-01-10"), 0);
        assert_eq!(faixa("2026-02-09"), 0);
        assert_eq!(faixa("2026-02-10"), 1);
        assert_eq!(faixa("2026-06-30"), 5);
        assert_eq!(faixa("2026-07-10"), 6);
        assert_eq!(faixa("2026-10-09"), 6);
        assert_eq!(faixa("2027-07-09"), 15);
        assert_eq!(faixa("2028-01-20"), 24);
    }

    #[test]
    fn only_a_premature_child_has_a_corrected_age() {
        let nasc = d("2026-01-10");
        let hoje = d("2026-07-10");
        let ig = |semanas: i64, dias: i64| ig_em_dias(Some(semanas), Some(dias));
        assert_eq!(ig_em_dias(None, Some(3)), None);
        assert_eq!(ig_em_dias(Some(32), None), Some(224));
        for ig in [None, ig(37, 0), ig(40, 0), ig(42, 0)] {
            let c = calcular(nasc, ig, None, hoje);
            assert_eq!((c.prematuro, c.idade_corrigida), (false, None), "{:?}", ig);
        }
        // 36 weeks and 6 days is still premature; the corrected age is 22 days less.
        let c = calcular(nasc, ig(36, 6), None, hoje);
        assert_eq!((c.prematuro, c.idade_corrigida), (true, Some(Idade { meses: 5, dias: 9 })));
        assert_eq!(months_and_days(nasc + 22, hoje), (5, 9));
        // The days count: 32 weeks and 4 days is 4 days less to discount than 32 weeks.
        assert_eq!(calcular(nasc, ig(32, 4), None, hoje).idade_corrigida, Some(Idade { meses: 4, dias: 7 }));
        assert_eq!(months_and_days(nasc + 52, hoje), (4, 7));
        // Born at 32 weeks, 8 weeks early: at six months the corrected age is 56 days less.
        let c = calcular(nasc, ig(32, 0), None, hoje);
        assert_eq!(c.idade, Idade { meses: 6, dias: 0 });
        assert_eq!(c.idade_corrigida, Some(Idade { meses: 4, dias: 3 }));
        assert_eq!(months_and_days(nasc + 56, hoje), (4, 3));
        // Before reaching 40 weeks there is no corrected age yet.
        let c = calcular(nasc, ig(32, 0), None, d("2026-02-10"));
        assert_eq!((c.prematuro, c.idade_corrigida), (true, None));
    }

    #[test]
    fn a_child_is_checked_before_it_is_saved() {
        let hoje = d("2026-11-22");
        let c = validar(&input("  Fulano de Tal ", "2026-01-10", Some(38), " maria ", Some("")), hoje).unwrap();
        assert_eq!(c, Crianca { registro: None, nome: "Fulano de Tal".to_string(), nascimento: d("2026-01-10"), ig_semanas: Some(38), ig_dias: Some(0), peso_nascimento: None, acs: "MARIA".to_string(), ultima_consulta: None });
        assert!(validar(&input("Fulano", "2026-01-10", None, "A", Some("2026-03-01")), hoje).is_ok());

        let longo = "a".repeat(121);
        for bad in [
            input("", "2026-01-10", None, "A", None),
            input("   ", "2026-01-10", None, "A", None),
            input(&longo, "2026-01-10", None, "A", None),
            input("Fulano\n", "2026-01-10", None, "A\u{7}B", None),
            input("Fulano", "2026-01-10", None, "", None),
            input("Fulano", "10/01/2026", None, "A", None),
            input("Fulano", "2026-11-23", None, "A", None),
            input("Fulano", "2026-01-10", Some(19), "A", None),
            input("Fulano", "2026-01-10", Some(45), "A", None),
            input("Fulano", "2026-01-10", None, "A", Some("2026-01-09")),
            input("Fulano", "2026-01-10", None, "A", Some("2026-11-23")),
            input("Fulano", "2026-01-10", None, "A", Some("ontem")),
        ] {
            assert!(validar(&bad, hoje).is_err(), "{:?} / {:?} / {:?} must not pass", bad.nome, bad.nascimento, bad.ultima_consulta);
        }
    }

    #[test]
    fn birth_weight_is_grams_within_reason() {
        let hoje = d("2026-11-22");
        let com = |peso: Option<i64>| {
            let mut i = input("Fulano", "2026-01-10", None, "A", None);
            i.peso_nascimento = peso;
            validar(&i, hoje).map(|c| c.peso_nascimento)
        };
        assert_eq!(com(None), Ok(None));
        assert_eq!(com(Some(3200)), Ok(Some(3200)));
        assert_eq!(com(Some(300)), Ok(Some(300)));
        // 3.2 typed as kilograms, or a slip of the finger, is not a weight in grams.
        for bad in [0, 3, 299, 7001, 32000, -3200] {
            assert!(com(Some(bad)).is_err(), "{} g must not pass", bad);
        }
    }

    #[test]
    fn vitamin_d_goes_up_at_one_year() {
        let nasc = d("2026-01-10");
        let ui = |hoje: &str| suplementacao(nasc, None, None, d(hoje)).vitamina_d_ui;
        assert_eq!(ui("2026-01-10"), 400);
        assert_eq!(ui("2027-01-09"), 400);
        assert_eq!(ui("2027-01-10"), 600);
        assert_eq!(ui("2028-01-20"), 600);
    }

    #[test]
    fn iron_at_term_with_adequate_weight_is_two_cycles() {
        let nasc = d("2026-01-10");
        let ferro = |hoje: &str| {
            let f = suplementacao(nasc, Some(39 * 7), Some(3200), d(hoje)).ferro.unwrap();
            assert_eq!((f.grupo, f.mg_kg), ("termo", None));
            (f.fase, f.ate)
        };
        let ate = |s: &str| Some(s.to_string());
        assert_eq!(ferro("2026-07-09"), ("antes", ate("2026-07-10")));
        assert_eq!(ferro("2026-07-10"), ("ciclo1", ate("2026-10-10")));
        assert_eq!(ferro("2026-10-09"), ("ciclo1", ate("2026-10-10")));
        assert_eq!(ferro("2026-10-10"), ("pausa", ate("2027-01-10")));
        assert_eq!(ferro("2027-01-10"), ("ciclo2", ate("2027-04-10")));
        assert_eq!(ferro("2027-04-09"), ("ciclo2", ate("2027-04-10")));
        assert_eq!(ferro("2027-04-10"), ("encerrada", None));
        // 37 weeks and 2,500 g are already term and adequate weight.
        assert_eq!(suplementacao(nasc, Some(37 * 7), Some(2500), d("2026-08-01")).ferro.unwrap().grupo, "termo");
    }

    #[test]
    fn iron_for_premature_or_low_weight_is_continuous_and_by_weight() {
        let nasc = d("2026-01-10");
        let ferro = |ig: Option<i64>, peso: Option<i64>, hoje: &str| {
            let f = suplementacao(nasc, ig.map(|s| s * 7), peso, d(hoje)).ferro.unwrap();
            assert_eq!(f.grupo, "risco");
            (f.fase, f.mg_kg, f.ate.unwrap())
        };
        let s = |fase: &'static str, mg: Option<i64>, ate: &str| (fase, mg, ate.to_string());
        // It starts at 30 days of life.
        assert_eq!(ferro(Some(34), Some(2000), "2026-02-08"), s("antes", Some(2), "2026-02-09"));
        assert_eq!(ferro(Some(34), Some(2000), "2026-02-09"), s("primeiro_ano", Some(2), "2027-01-10"));
        // The first year's dose goes by birth weight.
        for (peso, mg) in [(2499, 2), (1500, 2), (1499, 3), (1000, 3), (999, 4), (600, 4)] {
            assert_eq!(ferro(Some(30), Some(peso), "2026-06-01"), s("primeiro_ano", Some(mg), "2027-01-10"), "{} g", peso);
        }
        // In the second year it is 1 mg/kg/day whatever the birth weight, to 24 months with no pause.
        assert_eq!(ferro(Some(30), Some(900), "2027-01-10"), s("segundo_ano", Some(1), "2028-01-10"));
        assert_eq!(ferro(Some(30), Some(900), "2028-01-05"), s("segundo_ano", Some(1), "2028-01-10"));
        // Either condition is enough: born at term but light, or premature but not light.
        assert_eq!(ferro(Some(39), Some(2300), "2026-06-01"), s("primeiro_ano", Some(2), "2027-01-10"));
        assert_eq!(ferro(Some(36), Some(2700), "2026-06-01"), s("primeiro_ano", Some(2), "2027-01-10"));
        // And one known condition is enough when the other was not informed; the dose waits for the weight.
        assert_eq!(ferro(None, Some(2300), "2026-06-01"), s("primeiro_ano", Some(2), "2027-01-10"));
        assert_eq!(ferro(Some(34), None, "2026-06-01"), s("primeiro_ano", None, "2027-01-10"));
    }

    #[test]
    fn iron_is_not_guessed_when_the_birth_is_not_known() {
        let nasc = d("2026-01-10");
        for (ig, peso) in [(None, None), (Some(39 * 7), None), (None, Some(3200))] {
            assert_eq!(suplementacao(nasc, ig, peso, d("2026-08-01")).ferro, None, "{:?} / {:?}", ig, peso);
        }
    }

    #[test]
    fn a_registration_number_is_digits_only() {
        let hoje = d("2026-11-22");
        let com = |registro: Option<&str>| {
            let mut i = input("Fulano", "2026-01-10", None, "A", None);
            i.registro = registro.map(str::to_string);
            validar(&i, hoje).map(|c| c.registro)
        };
        assert_eq!(com(None), Ok(None));
        assert_eq!(com(Some("  ")), Ok(None));
        assert_eq!(com(Some(" 123456 ")), Ok(Some("123456".to_string())));
        assert_eq!(com(Some("007")), Ok(Some("007".to_string())));
        for bad in ["12a", "12.5", "-12", "12 34", "1234567890123456", "１２"] {
            assert!(com(Some(bad)).is_err(), "{:?} must not pass", bad);
        }
    }

    #[test]
    fn gestational_age_is_weeks_and_days() {
        let hoje = d("2026-11-22");
        let com = |semanas: Option<i64>, dias: Option<i64>| {
            let mut i = input("Fulano", "2026-01-10", semanas, "A", None);
            i.ig_dias = dias;
            validar(&i, hoje).map(|c| (c.ig_semanas, c.ig_dias))
        };
        assert_eq!(com(None, None), Ok((None, None)));
        assert_eq!(com(Some(34), None), Ok((Some(34), Some(0))));
        assert_eq!(com(Some(34), Some(6)), Ok((Some(34), Some(6))));
        for (semanas, dias) in [(None, Some(3)), (Some(34), Some(7)), (Some(34), Some(-1)), (Some(19), Some(6)), (Some(45), Some(0))] {
            assert!(com(semanas, dias).is_err(), "{:?} semanas e {:?} dias must not pass", semanas, dias);
        }
    }

    #[test]
    fn only_children_up_to_two_years_are_followed() {
        let hoje = d("2026-11-22");
        // Two years and thirty days: still "2 anos e 0 meses".
        assert!(validar(&input("Fulano", "2024-10-23", None, "A", None), hoje).is_ok());
        // Two years and one month.
        assert!(validar(&input("Fulano", "2024-10-22", None, "A", None), hoje).is_err());
    }

    #[test]
    fn the_password_is_compared_whole() {
        assert!(same_text("abc123", "abc123"));
        for (a, b) in [("abc123", "abc124"), ("abc123", "abc12"), ("", "a"), ("abc", "ABC")] {
            assert!(!same_text(a, b), "{:?} is not {:?}", a, b);
        }
    }
}
