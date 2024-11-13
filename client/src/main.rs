mod websocket;

use cursive::{
    view::{Nameable, Resizable},
    views::{Dialog, EditView, LinearLayout, ListView, ScrollView, TextView, ViewRef},
    CursiveRunnable, CursiveRunner,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = websocket::connect("ws://localhost:8080").await;
    let mut siv = cursive::default().into_runner();
    let username = LinearLayout::horizontal()
        .child(Dialog::around(TextView::new("username")))
        .child(Dialog::around(
            EditView::new()
                .filler(' ')
                .with_name("username")
                .fixed_width(10),
        ));
    let password = LinearLayout::horizontal()
        .child(Dialog::around(TextView::new("password")))
        .child(Dialog::around(
            EditView::new()
                .secret()
                .filler(" ")
                .with_name("password")
                .fixed_width(10),
        ));
    let layout = LinearLayout::vertical().child(username).child(password);
    let layout_sender = sender.clone();
    siv.add_layer(
        Dialog::around(layout)
            .title("Sing in")
            .button("Quit", |s| s.quit())
            .button("Enter", move |s| {
                let username: ViewRef<EditView> =
                    s.find_name("username").expect("falied find \"username\"");
                let password: ViewRef<EditView> =
                    s.find_name("password").expect("falied find \"password\"");
                let data = json!({
                    "username": username.get_content().to_string(),
                    "password": password.get_content().to_string(),
                }).to_string();
                layout_sender.send(data).expect("failed send");
            }).h_align(cursive::align::HAlign::Center),
    );
    siv.refresh();
    while siv.is_running() {
        siv.step();
        if !receiver.is_empty() {
            let msg = receiver.recv().await.expect("failed read");
            if msg != "@connected" {
                siv.add_layer(Dialog::info(msg).title("Failed authentication"));
            } else { break; }
        }
    }
    messages(siv, sender, receiver).await;
}

async fn messages(
    mut siv: CursiveRunner<CursiveRunnable>,
    sender: websocket::ChannelSender,
    mut receiver: websocket::ChannelReceiver,
) {
    siv.pop_layer();
    let messages = Dialog::around(
        ScrollView::new(ListView::new().with_name("messages").full_screen())
            .scroll_strategy(cursive::view::ScrollStrategy::StickToBottom),
    ).title("Messages");
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
    ).title("Input");
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
