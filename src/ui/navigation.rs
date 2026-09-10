pub const DESTINATIONS: [(&str, &str); 9] = [
    ("home", "Home"),
    ("learn", "Learn"),
    ("review", "Review"),
    ("reader", "Reader"),
    ("vocabulary", "Vocabulary"),
    ("kanji", "Kanji"),
    ("grammar", "Grammar"),
    ("progress", "Progress"),
    ("settings", "Settings"),
];

pub fn build_sidebar() -> gtk::ListBox {
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Browse)
        .css_classes(["navigation-sidebar"])
        .build();

    for (name, title) in DESTINATIONS {
        let row = gtk::ListBoxRow::builder()
            .name(name)
            .child(
                &gtk::Label::builder()
                    .label(title)
                    .xalign(0.0)
                    .margin_start(12)
                    .margin_end(12)
                    .margin_top(8)
                    .margin_bottom(8)
                    .build(),
            )
            .build();
        list.append(&row);
    }

    list
}
