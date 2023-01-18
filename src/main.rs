#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::{env, fs};
use std::path::{Path, PathBuf};

use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response::Content;
use rocket::response::NamedFile;

fn list_files(path: &Path, depth: usize) -> String {
    let mut display = String::new();
    display.push_str("<ul>");
    for entry in path.read_dir().unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name == ".git" {
            continue;
        }
        if entry.file_type().unwrap().is_dir() {
            display.push_str(&format!("<li>{}/", file_name));
            display.push_str(&list_files(entry.path().as_path(), depth + 1));
            display.push_str("</li>");
        } else {
            display.push_str(&format!("<li><a href='/files/{}'>{}</a></li>", entry.path().to_string_lossy(), file_name));
        }
    }
    display.push_str("</ul>");
    display
}

#[get("/")]
fn index() -> Content<String> {
    let head = "<!DOCTYPE html><html><head><title>File Server</title></head><body>";
    Content(ContentType::HTML, format!("{}<div class='file-list'>{}</div>", head, list_files(Path::new("."), 0)))
}

#[get("/files/<path..>")]
fn files(path: PathBuf) -> Result<NamedFile, Status> {
    let current_dir = env::current_dir().unwrap();
    let file_path = current_dir.join(path);
    let file = file_path.file_name().unwrap();
    let parent = file_path.parent().unwrap();
    let mut path = parent.to_path_buf();
    path.push(file);

    match fs::metadata(path.clone()) {
        Ok(metadata) => {
            if metadata.is_file() {
                NamedFile::open(path.clone()).or_else(|_| Err(Status::NotFound))
            } else {
                Err(Status::NotFound)
            }
        }
        Err(_) => Err(Status::NotFound),
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, files])
        .launch();
}
