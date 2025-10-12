use crate::player::AudioPlayer;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, FileChooserAction, FileChooserDialog,
    Label, ListBox, Orientation, ResponseType, ScrolledWindow,
};
use std::sync::Arc;

const DEFAULT_MUSIC_DIR: &str = "/home/bulat/music";

pub fn build_ui(app: &Application) {
    let player = Arc::new(AudioPlayer::new());

    let window = create_window(app);
    let main_box = create_main_container();
    let list_box = create_song_list();
    let controls_box = create_controls();

    main_box.append(&create_scrolled_window(&list_box));
    main_box.append(&controls_box);
    window.set_child(Some(&main_box));

    setup_event_handlers(&player, &list_box, &controls_box, &window);

    load_default_songs(&player, &list_box);

    window.present();
}

fn create_window(app: &Application) -> ApplicationWindow {
    ApplicationWindow::builder()
        .application(app)
        .title("Lol Player")
        .default_width(600)
        .default_height(400)
        .build()
}

fn create_main_container() -> GtkBox {
    let main_box = GtkBox::new(Orientation::Vertical, 10);
    main_box.set_margin_top(10);
    main_box.set_margin_bottom(10);
    main_box.set_margin_start(10);
    main_box.set_margin_end(10);
    main_box
}

fn create_song_list() -> ListBox {
    ListBox::new()
}

fn create_scrolled_window(list_box: &ListBox) -> ScrolledWindow {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_height(250)
        .build();
    scrolled.set_child(Some(list_box));
    scrolled
}

fn create_controls() -> GtkBox {
    let controls_box = GtkBox::new(Orientation::Horizontal, 5);
    controls_box.set_halign(gtk::Align::Center);
    controls_box.set_margin_top(10);

    let play_pause_btn = Button::with_label("▶/⏸");
    let load_btn = Button::with_label("Load from dir");

    controls_box.append(&play_pause_btn);
    controls_box.append(&load_btn);

    controls_box
}

fn setup_event_handlers(
    player: &Arc<AudioPlayer>,
    list_box: &ListBox,
    controls_box: &GtkBox,
    window: &ApplicationWindow,
) {
    let play_pause_btn = controls_box.first_child().unwrap();
    let load_btn = play_pause_btn.next_sibling().unwrap();

    setup_song_click_handler(player, list_box);

    setup_play_pause_handler(player, &play_pause_btn);

    setup_load_folder_handler(player, list_box, &load_btn, window);
}

fn setup_song_click_handler(player: &Arc<AudioPlayer>, list_box: &ListBox) {
    let player_clone = player.clone();
    list_box.connect_row_activated(move |_, row| {
        let index = row.index() as usize;
        let songs = player_clone.get_songs();

        if let Some(song) = songs.get(index) {
            player_clone.play_by_file(&song.1);
        }
    });
}

fn setup_play_pause_handler(player: &Arc<AudioPlayer>, button: &gtk::Widget) {
    let player_clone = player.clone();
    button
        .downcast_ref::<Button>()
        .unwrap()
        .connect_clicked(move |_| {
            player_clone.pause();
        });
}

fn setup_load_folder_handler(
    player: &Arc<AudioPlayer>,
    list_box: &ListBox,
    button: &gtk::Widget,
    window: &ApplicationWindow,
) {
    let player_clone = player.clone();
    let list_box_clone = list_box.clone();
    let window_clone = window.clone();

    button
        .downcast_ref::<Button>()
        .unwrap()
        .connect_clicked(move |_| {
            show_folder_chooser_dialog(&player_clone, &list_box_clone, &window_clone);
        });
}

fn show_folder_chooser_dialog(
    player: &Arc<AudioPlayer>,
    list_box: &ListBox,
    window: &ApplicationWindow,
) {
    let dialog = FileChooserDialog::new(
        Some("Where your mus dir?"),
        Some(window),
        FileChooserAction::SelectFolder,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Select", ResponseType::Accept),
        ],
    );

    let player = player.clone();
    let list_box = list_box.clone();

    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(folder) = dialog.file() {
                if let Some(path) = folder.path() {
                    player.load_songs_from_dir(&path.to_string_lossy());
                    update_song_list(&player, &list_box);
                }
            }
        }
        dialog.close();
    });

    dialog.show();
}

fn update_song_list(player: &Arc<AudioPlayer>, list_box: &ListBox) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    let songs = player.get_songs();
    for song in songs.iter() {
        if let Some(name) = song.1.file_name() {
            let label = Label::new(Some(&name.to_string_lossy()));
            label.set_halign(gtk::Align::Start);
            list_box.append(&label);
        }
    }
}

fn load_default_songs(player: &Arc<AudioPlayer>, list_box: &ListBox) {
    player.load_songs_from_dir(DEFAULT_MUSIC_DIR);
    update_song_list(player, list_box);
}
