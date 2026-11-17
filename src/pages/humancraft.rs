// Human craft (https://github.com/fbrunodr/humancraft), served by version.
//
//   /humancraft                  -> the latest release (the highest tag vX.Y)
//   /humancraft?v=v0.3           -> that release
//   /humancraft?rules=0          -> the latest release of that version of the rules (v0.*):
//                                   what a saved game asks for (its "version" field)
//   /humancraft/v0.3/...         -> that release's files
//   /humancraft/main/...         -> the main branch as it stands (a preview; while there is no
//                                   release yet, the latest is this)
//
// A version is built once, the first time it is asked for: the tag is checked out in a clone
// of the game's repository (bucket/humancraft/repo), built with its own tools, and the
// result kept (bucket/humancraft/builds/<version>). A release never changes, so its build
// is kept for good; the main branch is built again when it has moved on. While a build
// runs (some seconds; a minute the first time, when the game's tools are installed), the
// visitor sees a page that waits and reloads itself.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use actix_files::NamedFile;
use actix_web::http::header;
use actix_web::{get, web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;

const REPO_URL: &str = "https://github.com/fbrunodr/humancraft.git";
const ROOT: &str = "bucket/humancraft";
const MAIN: &str = "main";
// How long what the repository holds (its tags, where main stands) is taken on trust.
const REFS_FOR: Duration = Duration::from_secs(60);

// ---------- versions ----------

// "v0.3" -> (0, 3); anything else is not a release's name.
fn release(name: &str) -> Option<(u32, u32)> {
    let (x, y) = name.strip_prefix('v')?.split_once('.')?;
    let ok = |s: &str| !s.is_empty() && s.len() <= 6 && s.bytes().all(|b| b.is_ascii_digit());
    if !ok(x) || !ok(y) { return None; }
    Some((x.parse().ok()?, y.parse().ok()?))
}

struct Refs { at: Instant, releases: Vec<(u32, u32)>, main: String }

fn run(cmd: &mut Command) -> Result<String, String> {
    let out = cmd.output().map_err(|e| format!("{:?}: {}", cmd.get_program(), e))?;
    if !out.status.success() {
        return Err(format!("{:?} failed: {}", cmd.get_program(), String::from_utf8_lossy(&out.stderr).chars().take(2000).collect::<String>()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

// The releases there are and the commit main stands at, asked of the repository (and kept a minute).
fn refs() -> Result<(Vec<(u32, u32)>, String), String> {
    static CACHE: OnceLock<Mutex<Option<Refs>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(None));
    if let Some(r) = cache.lock().unwrap().as_ref() {
        if r.at.elapsed() < REFS_FOR { return Ok((r.releases.clone(), r.main.clone())); }
    }
    let listed = run(Command::new("git").args(["ls-remote", "--tags", "--heads", REPO_URL]))?;
    let mut releases = Vec::new();
    let mut main = String::new();
    for line in listed.lines() {
        let Some((sha, name)) = line.split_once('\t') else { continue };
        if name == "refs/heads/main" { main = sha.to_string(); }
        if let Some(tag) = name.strip_prefix("refs/tags/") {
            if let Some(v) = release(tag.trim_end_matches("^{}")) { if !releases.contains(&v) { releases.push(v); } }
        }
    }
    releases.sort();
    *cache.lock().unwrap() = Some(Refs { at: Instant::now(), releases: releases.clone(), main: main.clone() });
    Ok((releases, main))
}

// ---------- builds ----------

enum Build { Running, Failed(String, Instant) }

fn builds() -> &'static Mutex<HashMap<String, Build>> {
    static BUILDS: OnceLock<Mutex<HashMap<String, Build>>> = OnceLock::new();
    BUILDS.get_or_init(|| Mutex::new(HashMap::new()))
}

// Where a version's build is kept: a release under its own name; main under the commit it stood at.
fn build_dir(version: &str, main_sha: &str) -> PathBuf {
    let name = if version == MAIN { format!("main-{}", &main_sha[..main_sha.len().min(12)]) } else { version.to_string() };
    Path::new(ROOT).join("builds").join(name)
}

// Checks the version out and builds it into `out` (through a folder beside it, so a build is whole or not there).
// One build at a time: they share the clone.
fn build(version: &str, out: &Path) -> Result<(), String> {
    static ONE: Mutex<()> = Mutex::new(());
    let _one = ONE.lock().unwrap();
    if out.join("index.html").exists() { return Ok(()); }
    fs::create_dir_all(Path::new(ROOT).join("builds")).map_err(|e| e.to_string())?;
    let root = fs::canonicalize(ROOT).map_err(|e| e.to_string())?;
    let repo = root.join("repo");
    if !repo.join(".git").exists() {
        run(Command::new("git").args(["clone", "--quiet", REPO_URL]).arg(&repo))?;
    }
    run(Command::new("git").arg("-C").arg(&repo).args(["fetch", "--quiet", "--tags", "--force", "origin", "main"]))?;
    let target = if version == MAIN { "FETCH_HEAD".to_string() } else { format!("refs/tags/{}", version) };
    run(Command::new("git").arg("-C").arg(&repo).args(["checkout", "--quiet", "--force", "--detach", &target]))?;
    // (Its tools, installed again only when what it asks for has changed.)
    let lock = fs::read_to_string(repo.join("package-lock.json")).unwrap_or_default();
    let had = fs::read_to_string(root.join("installed.lock")).unwrap_or_default();
    if lock != had || !repo.join("node_modules").exists() {
        // (--ignore-scripts: no package's own install script is run on this server.)
        run(Command::new("npm").args(["ci", "--ignore-scripts", "--no-audit", "--no-fund"]).current_dir(&repo))?;
        fs::write(root.join("installed.lock"), &lock).map_err(|e| e.to_string())?;
    }
    let tmp = root.join("builds").join(format!(".building-{}", version));
    let _ = fs::remove_dir_all(&tmp);
    run(Command::new("npx").args(["vite", "build", "--emptyOutDir", "--logLevel", "error"])
        .arg(format!("--base=/humancraft/{}/", version)).arg("--outDir").arg(&tmp).current_dir(&repo))?;
    let out = root.join("builds").join(out.file_name().unwrap());
    let _ = fs::remove_dir_all(&out);
    fs::rename(&tmp, &out).map_err(|e| e.to_string())?;
    // (Main's older builds are of no more use.)
    if version == MAIN {
        if let Ok(list) = fs::read_dir(root.join("builds")) {
            for e in list.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with("main-") && e.path() != out { let _ = fs::remove_dir_all(e.path()); }
            }
        }
    }
    Ok(())
}

fn page(title: &str, body: &str, reload: bool) -> HttpResponse {
    let html = format!("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />{}<title>{}</title>
        <style>body{{margin:0;min-height:100vh;display:grid;place-items:center;background:#0b0f17;color:#d8dee9;font:16px/1.5 system-ui,sans-serif;text-align:center}}h1{{font-weight:600;letter-spacing:.12em}}p{{color:#8f9bb3;max-width:34em;margin:8px 16px}}pre{{text-align:left;white-space:pre-wrap;color:#e0a0a0;max-width:60em;margin:16px}}a{{color:#9ecbff}}</style>
        </head><body><div><h1>Human craft</h1>{}</div></body></html>",
        if reload { "<meta http-equiv=\"refresh\" content=\"4\" />" } else { "" }, title, body);
    HttpResponse::Ok().content_type("text/html; charset=utf-8").insert_header((header::CACHE_CONTROL, "no-store")).body(html)
}

fn escape(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;") }

// ---------- the routes ----------

#[derive(Deserialize)]
pub struct Which { v: Option<String>, rules: Option<u32> }

// /humancraft: on to the version asked for (or the latest).
#[get("/humancraft")]
pub async fn render(q: web::Query<Which>) -> Result<HttpResponse> {
    let found = web::block(refs).await?;
    let (releases, _) = match found {
        Ok(r) => r,
        Err(e) => return Ok(page("Human craft", &format!("<p>The game's repository could not be reached.</p><pre>{}</pre>", escape(&e)), false)),
    };
    let name = |v: &(u32, u32)| format!("v{}.{}", v.0, v.1);
    let version = if let Some(v) = &q.v {
        if v == MAIN { Some(MAIN.to_string()) } else { release(v).filter(|r| releases.contains(r)).map(|r| name(&r)) }
    } else if let Some(x) = q.rules {
        releases.iter().filter(|r| r.0 == x).last().map(name)
    } else {
        // (The latest release; before there is one, the main branch.)
        Some(releases.last().map(name).unwrap_or_else(|| MAIN.to_string()))
    };
    match version {
        Some(v) => Ok(HttpResponse::Found().insert_header((header::LOCATION, format!("/humancraft/{}/", v))).insert_header((header::CACHE_CONTROL, "no-store")).finish()),
        None => {
            let list = releases.iter().rev().map(|r| format!("<a href=\"/humancraft/{0}/\">{0}</a>", name(r))).collect::<Vec<_>>().join(" · ");
            Ok(page("Human craft", &format!("<p>There is no such version of the game.</p><p>{}</p>", if list.is_empty() { "<a href=\"/humancraft\">The latest</a>".to_string() } else { list }), false))
        }
    }
}

#[get("/humancraft/")]
pub async fn render_slash(q: web::Query<Which>) -> Result<HttpResponse> {
    // (Only a version's own name is passed on: nothing a visitor typed goes into the header as it came.)
    let query = match (&q.v, q.rules) {
        (Some(v), _) if v == MAIN || release(v).is_some() => format!("?v={}", v),
        (None, Some(x)) => format!("?rules={}", x),
        _ => String::new(),
    };
    Ok(HttpResponse::Found().insert_header((header::LOCATION, format!("/humancraft{}", query))).finish())
}

// /humancraft/<version>/<file>: the file of that version's build (built now, if it never was).
#[get("/humancraft/{version}/{file:.*}")]
pub async fn serve(req: HttpRequest, path: web::Path<(String, String)>) -> Result<HttpResponse> {
    let (version, file) = path.into_inner();
    if version != MAIN && release(&version).is_none() { return Ok(HttpResponse::NotFound().body("No such version.")); }
    let found = web::block(refs).await?;
    let (releases, main_sha) = match found {
        Ok(r) => r,
        Err(e) => return Ok(page("Human craft", &format!("<p>The game's repository could not be reached.</p><pre>{}</pre>", escape(&e)), false)),
    };
    if version != MAIN && !releases.contains(&release(&version).unwrap()) { return Ok(HttpResponse::NotFound().body("No such version.")); }
    if version == MAIN && main_sha.is_empty() { return Ok(HttpResponse::NotFound().body("No such version.")); }
    let dir = build_dir(&version, &main_sha);

    if !dir.join("index.html").exists() {
        // (Only the page itself starts a build or waits for one: a stray request for a file does not.)
        if !file.is_empty() && file != "index.html" { return Ok(HttpResponse::NotFound().body("Not built yet.")); }
        let key = dir.to_string_lossy().to_string();
        let mut all = builds().lock().unwrap();
        match all.get(&key) {
            Some(Build::Running) => {}
            // (A build that failed is tried again after a minute, not on every reload.)
            Some(Build::Failed(why, when)) if when.elapsed() < Duration::from_secs(60) => {
                return Ok(page("Human craft", &format!("<p>{} could not be built.</p><pre>{}</pre>", escape(&version), escape(why)), false));
            }
            _ => {
                all.insert(key.clone(), Build::Running);
                let (v, d) = (version.clone(), dir.clone());
                std::thread::spawn(move || {
                    let result = build(&v, &d);
                    let mut all = builds().lock().unwrap();
                    match result { Ok(()) => { all.remove(&key); } Err(e) => { all.insert(key, Build::Failed(e, Instant::now())); } }
                });
            }
        }
        return Ok(page("Human craft", &format!("<p>Getting {} ready: the game is being built from its source. This takes a few seconds (up to a minute the first time), and the page will open by itself.</p>", escape(&version)), true));
    }

    // (Never out of the build's own folder.)
    let wanted = if file.is_empty() { "index.html".to_string() } else { file };
    if wanted.split('/').any(|part| part == ".." || part.starts_with('.')) { return Ok(HttpResponse::NotFound().finish()); }
    let full = dir.join(&wanted);
    let (Ok(full), Ok(base)) = (fs::canonicalize(&full), fs::canonicalize(&dir)) else { return Ok(HttpResponse::NotFound().finish()); };
    if !full.starts_with(&base) || !full.is_file() { return Ok(HttpResponse::NotFound().finish()); }
    let named = NamedFile::open(full)?;
    let mut res = named.into_response(&req);
    // (A release never changes, and its assets are named by their content: kept long. Pages, and all of main, checked each time.)
    let lasting = version != MAIN && wanted.starts_with("assets/");
    res.headers_mut().insert(header::CACHE_CONTROL, header::HeaderValue::from_static(if lasting { "public, max-age=31536000, immutable" } else { "no-cache" }));
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::release;

    // A version's name is all that ever reaches a command or a path: "v", digits, a dot, digits, and nothing else.
    #[test]
    fn only_a_release_name_passes() {
        assert_eq!(release("v0.0"), Some((0, 0)));
        assert_eq!(release("v12.345"), Some((12, 345)));
        for bad in ["", "v", "v1", "v1.", "v.1", "1.0", "V1.0", "v1.0.0", "v1.0 ", " v1.0", "v1.0\n", "v-1.0", "v+1.0", "v1.0;ls", "v1.0/..", "../v1.0",
            "v1.0$(id)", "v1.0`id`", "v1.0|id", "v1.0&&id", "v1.0 --upload-pack=x", "--help", "v1.0\0", "v１.０", "v1e3.0", "v0x1.0", "v1234567.0", "main", "refs/tags/v1.0"] {
            assert_eq!(release(bad), None, "{:?} must not pass", bad);
        }
    }
}
