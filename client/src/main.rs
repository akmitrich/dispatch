mod websocket;

use cursive::{
    theme::Theme,
    view::{Nameable, Resizable},
    views::{Dialog, EditView, LinearLayout, ListView, ScrollView, TextView, ViewRef},
    CursiveRunnable, CursiveRunner,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = websocket::connect("ws://localhost:8080").await;
    let mut siv = cursive::default().into_runner();
    let mut theme = Theme::terminal_default();
    theme.shadow = false;
    siv.set_theme(theme);
    let username = LinearLayout::horizontal()
        .child(Dialog::text("username").h_align(cursive::align::HAlign::Center))
        .child(Dialog::around(
            EditView::new()
                .filler(" ")
                .with_name("username")
                .fixed_width(8),
        ));
    let password = LinearLayout::horizontal()
        .child(Dialog::text("password"))
        .child(Dialog::around(
            EditView::new()
                .secret()
                .filler(" ")
                .with_name("password")
                .fixed_width(8),
        ));
    let layout = LinearLayout::vertical().child(username).child(password);
    let sender_copy = sender.clone();
    siv.add_layer(
        Dialog::around(layout)
            .title("Dispatch")
            .button("Quit", |s| s.quit())
            .button("Enter", move |s| {
                let username: ViewRef<EditView> =
                    s.find_name("username").expect("falied find \"username\"");
                let password: ViewRef<EditView> =
                    s.find_name("password").expect("falied find \"password\"");
                let userdata = json!({
                    "username": username.get_content().to_string(),
                    "password": password.get_content().to_string(),
                });
                sender_copy.send(userdata.to_string()).expect("failed send");
            })
            .h_align(cursive::align::HAlign::Center),
    );
    siv.refresh();
    while siv.is_running() {
        siv.step();
        if !receiver.is_empty() {
            let msg = receiver.recv().await.expect("failed read");
            match msg.as_str() {
                "@connected" => {
                    message_exchange(&mut siv, sender.clone(), &mut receiver).await;
                }
                _ => {}
            }
        }
    }
}

async fn message_exchange(
    siv: &mut CursiveRunner<CursiveRunnable>,
    sender: websocket::ChannelSender,
    receiver: &mut websocket::ChannelReceiver,
) {
    siv.pop_layer();
    let messages = Dialog::around(
        ScrollView::new(ListView::new().with_name("messages").full_screen())
            .scroll_strategy(cursive::view::ScrollStrategy::StickToBottom),
    )
    .title("Messages");
    let input = Dialog::around(
        EditView::new()
            .filler(" ")
            .on_submit(move |s, text| {
                if !text.is_empty() {
                    let msg = text.to_string();
                    sender.send(msg).expect("failed send");
                    let mut input: ViewRef<EditView> =
                        s.find_name("input").expect("falied find \"input\"");
                    input.set_content("");
                }
            })
            .with_name("input")
            .full_width(),
    );
    let layout = LinearLayout::vertical().child(messages).child(input);
    siv.add_layer(layout);
    siv.refresh();
    while siv.is_running() {
        siv.step();
        if !receiver.is_empty() {
            let msg = receiver.recv().await.expect("failed read");
            siv.call_on_name("messages", |view: &mut ListView| {
                view.add_child("", TextView::new(msg));
            });
            siv.refresh();
        }
    }
}
