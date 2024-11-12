use cursive::{
    theme::Theme,
    view::{Nameable, Resizable},
    views::{Dialog, EditView, LinearLayout, ListView, ScrollView, TextView},
    Cursive,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub type ChannelSender = mpsc::UnboundedSender<String>;
pub type ChannelReceiver = mpsc::UnboundedReceiver<String>;

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = connect("ws://localhost:8080").await;
    let mut siv = cursive::default().into_runner();
    let theme = Theme::terminal_default();
    siv.set_theme(theme);
    let username = LinearLayout::horizontal()
        .child(Dialog::around(TextView::new("username")))
        .child(Dialog::around(
            EditView::new()
                .on_submit(move |s, text| {
                    sender.send(text.to_string()).expect("failed send");
                    handler(s, sender.clone());
                })
                .filler(" ")
                .fixed_width(10)
                .with_name("username"),
        ));
    siv.add_layer(Dialog::around(username).title("Sing in"));
    siv.refresh();
    while siv.is_running() {
        siv.step();
        if !receiver.is_empty() {
            let msg = receiver.recv().await.unwrap();
            siv.call_on_name("messages", |view: &mut ListView| {
                view.add_child("", TextView::new(msg));
            });
            siv.refresh();
        }
    }
}

async fn connect(url: &str) -> (ChannelSender, ChannelReceiver) {
    let (ws_stream, _) = connect_async(url).await.expect("failed connect");
    let (mut ws_tx, mut ws_rx) = ws_stream.split();
    let (server_tx, mut server_rx) = mpsc::unbounded_channel::<String>();
    tokio::spawn(async move {
        while let Some(msg) = server_rx.recv().await {
            ws_tx.send(Message::from(msg)).await.expect("failed send");
        }
    });
    let (client_tx, client_rx) = mpsc::unbounded_channel::<String>();
    tokio::spawn(async move {
        while let Some(Ok(Message::Text(msg))) = ws_rx.next().await {
            client_tx.send(msg).expect("failed send");
        }
    });
    (server_tx, client_rx)
}

fn handler(s: &mut Cursive, sender: ChannelSender) {
    s.pop_layer();
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
                    s.call_on_name("input", |view: &mut EditView| {
                        view.set_content("");
                    });
                }
            })
            .with_name("input")
            .full_width(),
    ).title("Input");
    let layout = LinearLayout::vertical().child(messages).child(input);
    s.add_layer(layout);
}
