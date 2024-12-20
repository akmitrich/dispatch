mod authrequest;
mod channelmessage;
mod customviews;

use anyhow::Result;
use authrequest::AuthRequest;
use channelmessage::ChannelMessage;
use crossterm::style::Stylize;
use cursive::{
    theme::Theme,
    utils::markup::ansi::parse,
    view::{Nameable, Resizable},
    views::{
        Button, DummyView, EditView, LinearLayout, ListView, Panel, ScrollView, TextView, ViewRef,
    },
    CursiveRunnable, CursiveRunner,
};
use customviews::{MessageView, PasswordView, UsernameView};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use ureq::serde_json;

#[tokio::main]
async fn main() {
    let mut siv = cursive::default().into_runner();
    let mut theme = Theme::terminal_default();
    theme.shadow = false;
    siv.set_theme(theme);
    let layout = LinearLayout::vertical()
        .child(UsernameView::new("username"))
        .child(PasswordView::new("password"))
        .child(Button::new("No access? Sign up", |s| {
            let username: ViewRef<EditView> = s.find_name("username").unwrap();
            let password: ViewRef<EditView> = s.find_name("password").unwrap();
            let userdata = AuthRequest {
                username: &username.get_content().to_string(),
                password: &password.get_content().to_string(),
            };
            let mut status: ViewRef<TextView> = s.find_name("status").unwrap();
            if userdata.is_valid() {
                match ureq::post("http://localhost:3000/signup").send_json(&userdata) {
                    Ok(response) => status.set_content(parse(
                        response.into_string().unwrap().on_green().to_string(),
                    )),
                    Err(err) => match err.into_response() {
                        Some(response) => {
                            let response = response.into_string().unwrap();
                            status.set_content(parse(response.on_red().to_string()));
                        }
                        None => status.set_content(parse("Failed connection".on_red().to_string())),
                    },
                }
            } else {
                status.set_content(parse("Wrong format".on_red().to_string()));
            }
        }))
        .child(
            TextView::empty()
                .h_align(cursive::align::HAlign::Center)
                .with_name("status"),
        )
        .child(
            LinearLayout::horizontal()
                .child(DummyView.fixed_width(5))
                .child(Button::new("Quit", |s| s.quit()))
                .child(DummyView.fixed_width(2))
                .child(Button::new("Sign in", |s| {
                    let username: ViewRef<EditView> = s.find_name("username").unwrap();
                    let password: ViewRef<EditView> = s.find_name("password").unwrap();
                    let userdata = AuthRequest {
                        username: &username.get_content().to_string(),
                        password: &password.get_content().to_string(),
                    };
                    let mut status: ViewRef<TextView> = s.find_name("status").unwrap();
                    if userdata.is_valid() {
                        match ureq::post("http://localhost:3000/signin").send_json(&userdata) {
                            Ok(response) => s.set_user_data(response.into_string().unwrap()),
                            Err(err) => match err.into_response() {
                                Some(response) => {
                                    let response = response.into_string().unwrap();
                                    status.set_content(parse(response.on_red().to_string()));
                                }
                                None => status
                                    .set_content(parse("Failed connection".on_red().to_string())),
                            },
                        }
                    } else {
                        status.set_content(parse("Wrong format".on_red().to_string()));
                    }
                })),
        );
    siv.add_layer(Panel::new(layout).title("Dispatch"));
    siv.refresh();
    while siv.is_running() {
        siv.step();
        if let Some(token) = siv.take_user_data::<String>() {
            connect(&mut siv, token).await.expect("failed \"connect\"");
        }
    }
}

async fn connect(siv: &mut CursiveRunner<CursiveRunnable>, token: String) -> Result<()> {
    let (mut ws_stream, _) = connect_async("ws://localhost:3000/connect").await?;
    ws_stream.send(Message::Text(token)).await?;
    let (mut ws_tx, mut ws_rx) = ws_stream.split();
    let (sender, mut receiver) = mpsc::unbounded_channel::<ChannelMessage>();
    tokio::spawn(async move {
        while let Some(Ok(Message::Text(msg))) = ws_rx.next().await {
            let message =
                serde_json::from_str(&msg).expect("failed deserialize to \"ChannelMessage\"");
            sender.send(message).expect("failed to send to channel");
        }
    });
    let layout = LinearLayout::vertical()
        .child(
            ScrollView::new(ListView::new())
                .with_name("messages")
                .full_screen(),
        )
        .child(Panel::new(
            EditView::new()
                .filler(" ")
                .on_submit(|s, text| {
                    let text = text.trim_end().to_string();
                    if !text.is_empty() {
                        s.set_user_data(text);
                        let mut input: ViewRef<EditView> = s.find_name("input").unwrap();
                        input.set_content("");
                    }
                })
                .with_name("input")
                .full_width(),
        ));
    siv.add_layer(layout);
    siv.refresh();
    while siv.is_running() {
        siv.step();
        if let Some(msg) = siv.take_user_data::<String>() {
            ws_tx.send(Message::Text(msg)).await.unwrap();
        }
        if !receiver.is_empty() {
            if let Some(msg) = receiver.recv().await {
                siv.call_on_name("messages", |view: &mut ScrollView<ListView>| {
                    view.get_inner_mut().add_child("", MessageView::new(msg));
                    view.scroll_to_bottom()
                });
                siv.refresh();
            }
        }
    }
    Ok(())
}
