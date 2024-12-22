use crate::channelmessage::ChannelMessage;
use chrono::{DateTime, Local};
use cursive::{
    view::{Nameable, Resizable},
    views::{Dialog, EditView, LinearLayout, Panel, TextView},
};

pub struct UsernameView;
impl UsernameView {
    pub fn new(name: &str) -> LinearLayout {
        LinearLayout::horizontal()
            .child(Dialog::text("username"))
            .child(Dialog::around(
                EditView::new().filler(" ").with_name(name).fixed_width(8),
            ))
    }
}

pub struct PasswordView;
impl PasswordView {
    pub fn new(name: &str) -> LinearLayout {
        LinearLayout::horizontal()
            .child(Dialog::text("password"))
            .child(Dialog::around(
                EditView::new()
                    .secret()
                    .filler(" ")
                    .with_name(name)
                    .fixed_width(8),
            ))
    }
}

pub struct MessageView;
impl MessageView {
    pub fn new(message: ChannelMessage) -> Panel<LinearLayout> {
        let timestamp = DateTime::from_timestamp_millis(message.timestamp)
            .unwrap()
            .with_timezone(&Local::now().timezone())
            .format("%H:%M")
            .to_string();
        Panel::new(
            LinearLayout::horizontal()
                .child(
                    TextView::new(message.body)
                        .h_align(cursive::align::HAlign::Left)
                        .full_width(),
                )
                .child(
                    TextView::new(timestamp)
                        .h_align(cursive::align::HAlign::Right)
                        .full_width(),
                ),
        )
        .title(message.from)
        .title_position(cursive::align::HAlign::Left)
    }
}
