use adw::prelude::*;
use adw::{ActionRow, Application, ApplicationWindow, HeaderBar};
use gtk::{Box, ListBox, Orientation, SelectionMode, Label};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::io;
use std::string::FromUtf8Error;
use serde_json::Error as SerdeError;

// Generation object
#[derive(Debug, Serialize, Deserialize)]
pub struct Generation {
    pub generation: u64,
    pub date: String,
    pub nixosVersion: String,
    pub kernelVersion: String,
    pub configurationRevision: String,
    pub specialisations: Vec<String>,
    pub current: bool,
}

// Generation parsing error
#[derive(Debug)]
pub enum GenerationError {
    Io(io::Error),
    Utf8(FromUtf8Error),
    Json(SerdeError),
}

// Parse Io
impl From<io::Error> for GenerationError {
    fn from(err: io::Error) -> Self {
        GenerationError::Io(err)
    }
}
// Parse Utf8
impl From<FromUtf8Error> for GenerationError {
    fn from(err: FromUtf8Error) -> Self {
        GenerationError::Utf8(err)
    }
}
// Parse Json
impl From<SerdeError> for GenerationError {
    fn from(err: SerdeError) -> Self {
        GenerationError::Json(err)
    }
}

pub fn get_nixos_generations() -> Result<Vec<Generation>, GenerationError> {
    let output = Command::new("nixos-rebuild")
        .arg("list-generations")
        .arg("--json")
        .output()?;

    if !output.status.success() {
        return Err(GenerationError::Io(io::Error::new(io::ErrorKind::Other, "Command failed")));
    }

    let json_str = String::from_utf8(output.stdout)?;
    let generations: Vec<Generation> = serde_json::from_str(&json_str)?;
    Ok(generations)
}

fn main() {
    let application = Application::builder()
        .application_id("com.example.Generator")
        .build();

    application.connect_activate(|app| {
        // ActionRows are only available in Adwaita
        let row = ActionRow::builder()
            .activatable(true)
            .title("Click me")
            .build();
        row.connect_activated(|_| {
            eprintln!("{:?}", get_nixos_generations().unwrap()[1]);
        });

        let list = ListBox::builder()
            .margin_top(32)
            .margin_end(32)
            .margin_bottom(32)
            .margin_start(32)
            .selection_mode(SelectionMode::None)
            // makes the list look nicer
            .css_classes(vec![String::from("boxed-list")])
            .build();

        // Iterate through generations for listbox:
        for g in get_nixos_generations().unwrap() {
            let row = ActionRow::builder()
                .activatable(true)
                .title(format!("{:?}", g.generation))
                .subtitle(format!("{:?}", g.date))
                .build();
            row.connect_activated(|_| {
                eprintln!("{:?}", get_nixos_generations().unwrap()[1]);
            });
            list.append(&row);
        }

        // Combine the content in a box
        let content = Box::new(Orientation::Vertical, 0);
        // Adwaitas' ApplicationWindow does not include a HeaderBar
        let cleanheader = HeaderBar::builder()
            .show_title(true)
            .css_classes(["flat"])
            .build();
        content.append(&cleanheader);
        content.append(&list);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Dipstick")
            .default_width(350)
            // add content to window
            .content(&content)
            .build();
        window.present();
    });

    application.run();
}
