use std::{collections::HashMap, net::TcpStream};

use axum::{
    extract::Query,
    response::Html,
    routing::{get, post},
    Form, Router,
};
use clipboard::{ClipboardContext, ClipboardProvider};

use html_tag::HtmlTag;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[derive(serde::Deserialize)]
struct Search {
    q: String,
}

const VALID: [&str; 8] = ["jpg", "png", "jpeg", "webp", "gif", "mp4", "mkv", "webm"];

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home))
        .route("/all", get(all))
        .route("/search", post(search_form))
        .route("/search", get(search))
        .nest_service("/assets", ServeDir::new("./static"));

    let listener = TcpListener::bind("0.0.0.0:2468")
        .await
        .expect("Failed to bind port");

    println!("Server running on http://{}:2468", get_ip_addr());
    copy(&format!("http://{}:2468", get_ip_addr()));
    println!("Copied to clipboard");
    axum::serve(listener, app).await.expect("Failed to serve")
}

fn copy(msg: &str) {
    let mut board: ClipboardContext = ClipboardProvider::new().unwrap();
    board.set_contents(msg.to_owned()).unwrap();
}

fn get_ip_addr() -> String {
    let wlan = ip_extractor::get_wlan(None);
    let mut valid = String::new();

    wlan.iter().for_each(|x| {
        TcpStream::connect(format!("{}:2468", x.inet.as_ref().expect("No IP"))).unwrap();
        valid = x.inet.as_ref().unwrap().to_string();
    });

    valid
}

fn get_htmx_script_content() -> &'static str {
    include_str!("htmx.min.js")
}

fn list_files() -> Vec<String> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir("./static").unwrap() {
        let path = entry.unwrap().path();
        if !VALID.contains(&path.extension().unwrap().to_str().unwrap()) {
            continue;
        }
        files.push(
            path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_ascii_lowercase()
                .to_string(),
        );
    }

    files
}

fn find_files(query: &str) -> Vec<String> {
    let files = list_files()
        .iter()
        .filter(|x| {
            x.replace(['.', '_', '-'].as_ref(), "")
                .to_ascii_lowercase()
                .contains(
                    query
                        .trim()
                        .replace(' ', "")
                        .to_ascii_lowercase()
                        .to_string()
                        .as_str(),
                )
        })
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    match files.len() {
        0 => list_files(),
        _ => files,
    }
}

async fn home() -> Html<String> {
    let basic_html =
        include_str!("index.html").replace("{{htmx_script}}", get_htmx_script_content());

    Html(basic_html)
}

async fn all() -> Html<String> {
    let mut html = HtmlTag::new("div");

    list_files().iter().for_each(|x| {
        let mut link = HtmlTag::new("a");
        link.add_class("link");
        link.set_href(format!("/assets/{}", x).as_str());
        link.set_body(x);

        let mut p = HtmlTag::new("p");
        p.add_child(link);

        html.add_child(p);
    });

    Html(html.to_string())
}

async fn search_form(Form(search): Form<Search>) -> Html<String> {
    let mut html = HtmlTag::new("div");

    find_files(&search.q).iter().for_each(|x| {
        let mut link = HtmlTag::new("a");
        link.add_class("link");
        link.set_href(format!("/assets/{}", x).as_str());
        link.set_body(x);

        let mut p = HtmlTag::new("p");
        p.add_child(link);

        html.add_child(p);
    });

    Html(html.to_string())
}

async fn search(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    let default = "".to_string();
    let query = params.get("q").unwrap_or(&default);

    let mut html = HtmlTag::new("div");

    find_files(query).iter().for_each(|x| {
        let mut link = HtmlTag::new("a");
        link.add_class("link");
        link.set_href(format!("/assets/{}", x).as_str());
        link.set_body(x);

        let mut p = HtmlTag::new("p");
        p.add_child(link);

        html.add_child(p);
    });

    Html(html.to_string())
}
