use adw::prelude::*;
use adw::{ActionRow, Application, ApplicationWindow, HeaderBar, NavigationView, NavigationPage};
use gtk::{Box, ListBox, Orientation, SelectionMode, Label};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::io;
use std::string::FromUtf8Error;
use serde_json::Error as SerdeError;
use glib::clone;

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

// NixOS generation retrieval function (parses from json)
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

// program entrypoint
fn main() {
    // App definition
    let application = Application::builder()
        .application_id("com.example.Generator")
        .build();

    // App closure
    application.connect_activate(|app| {
        let list = ListBox::builder()
            .margin_top(32)
            .margin_end(32)
            .margin_bottom(32)
            .margin_start(32)
            .selection_mode(SelectionMode::None)
            // makes the list look nicer
            .css_classes(vec![String::from("boxed-list")])
            .build();

        // Combine the content in a box
        let content = Box::new(Orientation::Vertical, 0);

        // Adwaitas' ApplicationWindow does not include a HeaderBar
        let cleanheader = HeaderBar::builder()
            .show_title(true)
            .css_classes(["flat"])
            .build();
        content.append(&cleanheader);
        content.append(&list);

        // Combine the content in a box
        let homepage = NavigationPage::new(&content, "Dipstick");
        // Combine the content in a box
        let navigation = NavigationView::new();
        navigation.push(&homepage);

        // Iterate through generations for listbox:
        for g in get_nixos_generations().unwrap() {
            let row = ActionRow::builder()
                .activatable(true)
                .title(format!("Generation {:?}", &g.generation))
                .subtitle(format!("{}", &g.date))
                .build();
            row.connect_activated(clone!(@strong navigation => move |_| {
                // Create new header
                let gen_header = HeaderBar::builder()
                    .show_title(true)
                    .css_classes(["flat"])
                    .build();
                // Builders for various parameters
                let nixosVersion = ActionRow::builder()
                    .activatable(false)
                    .title(format!("NixOS Version"))
                    .subtitle(format!("{}", &g.nixosVersion))
                    .build();
                let kernelVersion = ActionRow::builder()
                    .activatable(false)
                    .title(format!("Linux Kernel Version"))
                    .subtitle(format!("{}", &g.kernelVersion))
                    .build();
                let currentVersion = ActionRow::builder()
                    .activatable(false)
                    .title(format!("Current Generation"))
                    .subtitle(format!("{}", &g.current))
                    .build();
                // Combine the content in a box
                let gen_content = Box::new(Orientation::Vertical, 0);
                gen_content.append(&gen_header);
                gen_content.append(&nixosVersion);
                gen_content.append(&kernelVersion);
                gen_content.append(&currentVersion);
                // Generate page
                let gen_page = NavigationPage::new(&gen_content, &format!("Generation {:?}", &g.generation));
                navigation.push(&gen_page);
            }));
            list.append(&row);
        }

        // Define window parameters
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Dipstick")
            .default_width(350)
            // add content to window
            .content(&navigation)
            .build();
        window.present();
    });

    // start app
    application.run();
}
