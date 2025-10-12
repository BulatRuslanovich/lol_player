mod player;
mod ui;

use gtk::prelude::*;
use gtk::Application;

fn main() {
    let app = Application::builder()
        .application_id("com.bipbop.lol")
        .build();

    app.connect_activate(ui::build_ui);
    app.run();
}