use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, Orientation, ScrolledWindow, TextView};
use std::process::Command;

const APP_ID: &str = "org.taria.PackageManager";

pub fn run() -> anyhow::Result<()> {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
    Ok(())
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Taria — Package Manager")
        .default_width(1040)
        .default_height(700)
        .build();

    let root = GtkBox::new(Orientation::Horizontal, 0);
    root.add_css_class("taria-root");

    let sidebar = GtkBox::new(Orientation::Vertical, 12);
    sidebar.set_width_request(220);
    sidebar.set_margin_top(24);
    sidebar.set_margin_bottom(24);
    sidebar.set_margin_start(18);
    sidebar.set_margin_end(18);
    sidebar.add_css_class("sidebar");

    let brand = Label::new(Some("TARIA"));
    brand.add_css_class("brand");
    sidebar.append(&brand);
    let subtitle = Label::new(Some("Package Manager"));
    subtitle.add_css_class("muted");
    sidebar.append(&subtitle);

    let discover = Button::with_label("Discover");
    let installed = Button::with_label("Installed packages");
    let updates = Button::with_label("Updates");
    let settings = Button::with_label("Settings");
    for button in [&discover, &installed, &updates, &settings] {
        button.add_css_class("nav-button");
        sidebar.append(button);
    }
    let spacer = GtkBox::new(Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    sidebar.append(&spacer);
    sidebar.append(&Label::new(Some("Open source • Ubuntu")));

    let content = GtkBox::new(Orientation::Vertical, 16);
    content.set_margin_top(28);
    content.set_margin_bottom(24);
    content.set_margin_start(28);
    content.set_margin_end(28);
    content.set_hexpand(true);

    let heading = Label::new(Some("Discover applications"));
    heading.set_xalign(0.0);
    heading.add_css_class("heading");
    content.append(&heading);

    let description = Label::new(Some("Search Ubuntu packages and the community-maintained Taria catalog."));
    description.set_xalign(0.0);
    description.add_css_class("muted");
    content.append(&description);

    let search_row = GtkBox::new(Orientation::Horizontal, 10);
    let search = Entry::builder().placeholder_text("Search software, e.g. firefox").hexpand(true).build();
    let search_button = Button::with_label("Search");
    search_button.add_css_class("suggested-action");
    search_row.append(&search);
    search_row.append(&search_button);
    content.append(&search_row);

    let output = TextView::new();
    output.set_editable(false);
    output.set_cursor_visible(false);
    output.set_wrap_mode(gtk::WrapMode::WordChar);
    let scroll = ScrolledWindow::builder().vexpand(true).hexpand(true).child(&output).build();
    content.append(&scroll);

    let status = Label::new(Some("Ready. Search for a package to see APT results."));
    status.set_xalign(0.0);
    status.add_css_class("muted");
    content.append(&status);

    let install_hint = Label::new(Some("Terminal: sudo taria install <package>  •  Shortcuts: i / s / u / r"));
    install_hint.set_xalign(0.0);
    install_hint.add_css_class("muted");
    content.append(&install_hint);

    let output_buffer = output.buffer();
    let search_entry = search.clone();
    let status_label = status.clone();
    let output_buffer_clone = output_buffer.clone();
    search_button.connect_clicked(move |_| {
        run_search(search_entry.text().as_str(), &output_buffer_clone, &status_label);
    });
    let status_label = status.clone();
    let output_buffer_clone = output_buffer.clone();
    search.connect_activate(move |entry| {
        run_search(entry.text().as_str(), &output_buffer_clone, &status_label);
    });

    let installed_status = status.clone();
    let installed_buffer = output_buffer.clone();
    installed.connect_clicked(move |_| {
        match Command::new("dpkg-query")
            .args(["-W", "-f=${binary:Package}\t${Version}\t${db:Status-Status}\n"])
            .output()
        {
            Ok(result) => {
                let text = String::from_utf8_lossy(&result.stdout).to_string();
                installed_buffer.set_text(&text);
                installed_status.set_text("Installed Debian packages");
            }
            Err(err) => installed_status.set_text(&format!("Could not list packages: {err}")),
        }
    });

    let update_status = status.clone();
    updates.connect_clicked(move |_| {
        update_status.set_text("To refresh package lists, run: sudo taria update");
    });
    let settings_status = status.clone();
    settings.connect_clicked(move |_| {
        settings_status.set_text("Taria uses APT/DPKG on Ubuntu. Catalog source: GitHub.");
    });
    let discover_status = status.clone();
    discover.connect_clicked(move |_| {
        discover_status.set_text("Search the catalog and Ubuntu repositories above.");
    });

    root.append(&sidebar);
    root.append(&content);
    window.set_child(Some(&root));

    let css = gtk::CssProvider::new();
    css.load_from_data("\
        window, .taria-root { background: #111318; color: #f4f4f5; }\
        .sidebar { background: #191c22; border-right: 1px solid #30343c; }\
        .brand { color: #ff8a1f; font-size: 30px; font-weight: 900; letter-spacing: 3px; }\
        .heading { font-size: 26px; font-weight: 700; color: #ffffff; }\
        .muted { color: #a5aab4; }\
        .nav-button { text-align: left; padding: 12px; }\
        entry, textview { background: #20242c; color: #f4f4f5; border: 1px solid #363c46; padding: 10px; }\
        button.suggested-action { background: #f07819; color: #161616; font-weight: 700; }\
    ");
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    window.present();
}

fn run_search(query: &str, buffer: &gtk::TextBuffer, status: &Label) {
    let query = query.trim();
    if query.is_empty() {
        status.set_text("Enter a search term first.");
        return;
    }
    match Command::new("apt-cache").args(["search", "--", query]).output() {
        Ok(result) => {
            let text = String::from_utf8_lossy(&result.stdout).to_string();
            if text.is_empty() {
                buffer.set_text("No APT matches. The online Taria catalog is also available from the terminal using: taria search <name>");
            } else {
                buffer.set_text(&text);
            }
            status.set_text("Search completed. To install a result, use: sudo taria install <package>");
        }
        Err(err) => status.set_text(&format!("Search failed: {err}")),
    }
}
