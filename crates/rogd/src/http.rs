use std::collections::BTreeMap;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant, SystemTime};

use axum::extract::{Path, Request, State};
use axum::http::{header, HeaderMap, HeaderName, StatusCode, Uri};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response as HttpResponse};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use rog_fx::{Command, Effect, Response};
use rog_hid::{Profile, Rgb};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};

use crate::engine::Engine;

const BIND: &str = "127.0.0.1:7777";
/// Anything else is a page on another origin (or a DNS-rebinding attempt) trying to drive the LEDs.
const ALLOWED_HOSTS: [&str; 2] = ["127.0.0.1:7777", "localhost:7777"];
const ALLOWED_ORIGINS: [&str; 2] = ["http://127.0.0.1:7777", "http://localhost:7777"];

#[derive(RustEmbed)]
#[folder = "../../web/dist"]
struct Assets;

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<Mutex<Engine>>,
    pub profile: Arc<Profile>,
    pub start: Instant,
}

#[derive(Deserialize)]
struct BrightnessBody {
    level: u8,
}

#[derive(Deserialize)]
struct NameBody {
    name: String,
}

#[derive(Deserialize)]
struct UpdateBody {
    name: Option<String>,
    favourite: Option<bool>,
}

#[derive(Deserialize)]
struct DefaultBody {
    id: Option<String>,
}

#[derive(Serialize)]
struct LedColour<'a> {
    name: &'a str,
    group: &'a str,
    colour: Rgb,
}

#[derive(Serialize)]
struct FrameBody<'a> {
    device: &'a str,
    leds: Vec<LedColour<'a>>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/state", get(get_state))
        .route("/api/v1/effect", put(put_effect))
        .route("/api/v1/brightness", put(put_brightness))
        .route("/api/v1/paint", put(put_paint))
        .route("/api/v1/frame", get(get_frame))
        .route("/api/v1/presets", get(list_presets).post(save_preset))
        .route("/api/v1/presets/{id}", axum::routing::patch(update_preset).delete(delete_preset))
        .route("/api/v1/presets/{id}/apply", post(apply_preset))
        .route("/api/v1/default-preset", put(put_default_preset))
        .fallback(asset)
        .layer(middleware::from_fn(guard))
        .with_state(state)
}

/// Serve the API and UI on a background thread; the LED loop keeps running if the port is taken.
pub fn spawn(state: AppState) {
    thread::spawn(move || {
        let run = || -> io::Result<()> {
            let rt = tokio::runtime::Builder::new_current_thread().enable_io().build()?;
            rt.block_on(async {
                let listener = tokio::net::TcpListener::bind(BIND).await?;
                eprintln!("ui on http://{BIND}");
                axum::serve(listener, router(state)).await
            })
        };
        if let Err(e) = run() {
            eprintln!("http server stopped ({e}); LED control over the socket is unaffected");
        }
    });
}

fn header_is(headers: &HeaderMap, name: HeaderName, allowed: &[&str]) -> Option<bool> {
    headers.get(name).map(|v| v.to_str().is_ok_and(|v| allowed.contains(&v)))
}

async fn guard(req: Request, next: Next) -> HttpResponse {
    let host_ok = header_is(req.headers(), header::HOST, &ALLOWED_HOSTS).unwrap_or(false);
    let origin_ok = header_is(req.headers(), header::ORIGIN, &ALLOWED_ORIGINS).unwrap_or(true);
    if host_ok && origin_ok {
        next.run(req).await
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}

fn run(state: &AppState, cmd: Command) -> (StatusCode, Json<Response>) {
    let reply = state.engine.lock().unwrap_or_else(|e| e.into_inner()).apply(cmd, Instant::now(), SystemTime::now());
    let status = if matches!(reply, Response::Error { .. }) { StatusCode::BAD_REQUEST } else { StatusCode::OK };
    (status, Json(reply))
}

async fn get_state(State(s): State<AppState>) -> (StatusCode, Json<Response>) {
    run(&s, Command::State)
}

async fn put_effect(State(s): State<AppState>, Json(effect): Json<Effect>) -> (StatusCode, Json<Response>) {
    run(&s, Command::SetEffect { effect })
}

async fn put_brightness(State(s): State<AppState>, Json(b): Json<BrightnessBody>) -> (StatusCode, Json<Response>) {
    run(&s, Command::SetBrightness { level: b.level })
}

async fn put_paint(State(s): State<AppState>, Json(paint): Json<BTreeMap<String, Rgb>>) -> (StatusCode, Json<Response>) {
    run(&s, Command::SetPaint { paint })
}

async fn list_presets(State(s): State<AppState>) -> (StatusCode, Json<Response>) {
    run(&s, Command::ListPresets)
}

async fn save_preset(State(s): State<AppState>, Json(b): Json<NameBody>) -> (StatusCode, Json<Response>) {
    run(&s, Command::SavePreset { name: b.name })
}

async fn apply_preset(State(s): State<AppState>, Path(id): Path<String>) -> (StatusCode, Json<Response>) {
    run(&s, Command::ApplyPreset { id })
}

async fn delete_preset(State(s): State<AppState>, Path(id): Path<String>) -> (StatusCode, Json<Response>) {
    run(&s, Command::DeletePreset { id })
}

async fn update_preset(State(s): State<AppState>, Path(id): Path<String>, Json(b): Json<UpdateBody>) -> (StatusCode, Json<Response>) {
    run(&s, Command::UpdatePreset { id, name: b.name, favourite: b.favourite })
}

async fn put_default_preset(State(s): State<AppState>, Json(b): Json<DefaultBody>) -> (StatusCode, Json<Response>) {
    run(&s, Command::SetDefaultPreset { id: b.id })
}

/// Current target colour of every LED, for the UI preview.
async fn get_frame(State(s): State<AppState>) -> impl IntoResponse {
    let t = s.start.elapsed();
    let targets = s.engine.lock().unwrap_or_else(|e| e.into_inner()).targets(&s.profile, t, Instant::now());
    let leds = s
        .profile
        .leds
        .iter()
        .zip(targets)
        .map(|(led, colour)| LedColour { name: &led.name, group: &led.group, colour })
        .collect();
    Json(FrameBody { device: &s.profile.name, leds }).into_response()
}

async fn asset(uri: Uri) -> HttpResponse {
    let path = uri.path().trim_start_matches('/');
    let found = Assets::get(path).map(|f| (path, f)).or_else(|| Assets::get("index.html").map(|f| ("index.html", f)));
    match found {
        Some((name, file)) => {
            let mime = mime_for(name);
            ([(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, "no-cache")], file.data.into_owned()).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn mime_for(name: &str) -> &'static str {
    match name.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript",
        Some("css") => "text/css",
        Some("svg") => "image/svg+xml",
        Some("woff2") => "font/woff2",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    fn app() -> Router {
        let engine = Engine::new(Effect::Static { colour: Rgb::new(255, 0, 0) });
        router(AppState { engine: Arc::new(Mutex::new(engine)), profile: Arc::new(Profile::g614ju()), start: Instant::now() })
    }

    fn request(method: &str, uri: &str, origin: Option<&str>, body: &str) -> Request {
        let mut b = Request::builder().method(method).uri(uri).header("host", "127.0.0.1:7777");
        if let Some(o) = origin {
            b = b.header("origin", o);
        }
        b.header("content-type", "application/json").body(Body::from(body.to_owned())).unwrap()
    }

    async fn send(req: Request) -> (StatusCode, String) {
        let res = app().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8_lossy(&bytes).into_owned())
    }

    #[tokio::test]
    async fn state_reports_the_effect_and_brightness() {
        let (status, body) = send(request("GET", "/api/v1/state", None, "")).await;

        assert_eq!(status, StatusCode::OK);
        assert!(body.contains(r##""colour":"#ff0000""##) && body.contains(r#""brightness":100"#), "{body}");
    }

    #[tokio::test]
    async fn put_effect_from_the_same_origin_is_accepted() {
        let body = r##"{"effect":"breathe","colour":"#00ffff","period_ms":2000}"##;

        let (status, _) = send(request("PUT", "/api/v1/effect", Some("http://127.0.0.1:7777"), body)).await;

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn a_foreign_origin_cannot_drive_the_leds() {
        let (status, _) = send(request("PUT", "/api/v1/brightness", Some("https://evil.example"), r#"{"level":0}"#)).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn a_foreign_host_header_is_rejected_against_dns_rebinding() {
        let req = Request::builder().uri("/api/v1/state").header("host", "evil.example:7777").body(Body::empty()).unwrap();

        let (status, _) = send(req).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn brightness_above_100_is_a_bad_request() {
        let (status, _) = send(request("PUT", "/api/v1/brightness", None, r#"{"level":101}"#)).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn unknown_effects_are_rejected() {
        let (status, _) = send(request("PUT", "/api/v1/effect", None, r#"{"effect":"disco"}"#)).await;

        assert!(status.is_client_error());
    }

    #[tokio::test]
    async fn frame_lists_every_led_with_its_colour() {
        let (_, body) = send(request("GET", "/api/v1/frame", None, "")).await;

        assert!(body.contains("ROG Strix G614JU") && body.contains(r#""name":"bar6""#), "{body}");
        assert_eq!(body.matches(r##""colour":"#ff0000""##).count(), 10);
    }

    #[test]
    fn mime_types_cover_the_build_output() {
        assert_eq!(mime_for("assets/app.js"), "text/javascript");
        assert_eq!(mime_for("index.html"), "text/html; charset=utf-8");
    }

    #[tokio::test]
    async fn presets_can_be_saved_listed_applied_favourited_and_deleted_over_http() {
        let state = AppState {
            engine: Arc::new(Mutex::new(Engine::new(Effect::Static { colour: Rgb::new(255, 0, 0) }))),
            profile: Arc::new(Profile::g614ju()),
            start: Instant::now(),
        };
        let call = |method: &'static str, uri: String, body: &'static str| {
            let app = router(state.clone());
            async move {
                let res = app.oneshot(request(method, &uri, None, body)).await.unwrap();
                let status = res.status();
                let bytes = res.into_body().collect().await.unwrap().to_bytes();
                (status, String::from_utf8_lossy(&bytes).into_owned())
            }
        };

        let (status, saved) = call("POST", "/api/v1/presets".into(), r#"{"name":"Night"}"#).await;
        assert_eq!(status, StatusCode::OK, "{saved}");
        let id = saved.split(r#""id":""#).nth(1).and_then(|r| r.split('"').next()).unwrap().to_owned();

        let (_, patched) = call("PATCH", format!("/api/v1/presets/{id}"), r#"{"favourite":true}"#).await;
        let (_, listed) = call("GET", "/api/v1/presets".into(), "").await;
        let (applied, _) = call("POST", format!("/api/v1/presets/{id}/apply"), "").await;
        let (deleted, _) = call("DELETE", format!("/api/v1/presets/{id}"), "").await;
        let (missing, _) = call("DELETE", format!("/api/v1/presets/{id}"), "").await;

        assert!(patched.contains(r#""status":"ok""#));
        assert!(listed.contains(r#""favourite":true"#) && listed.contains("Night"), "{listed}");
        assert_eq!((applied, deleted, missing), (StatusCode::OK, StatusCode::OK, StatusCode::BAD_REQUEST));
    }

    #[tokio::test]
    async fn put_paint_sets_zone_colours() {
        let (status, _) = send(request("PUT", "/api/v1/paint", None, r##"{"key1":"#ff0000","bar6":"#0000ff"}"##)).await;

        assert_eq!(status, StatusCode::OK);
    }
}
