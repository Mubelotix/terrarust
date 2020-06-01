use crate::progress_bar::ProgressBar;
use futures::{
    channel::{oneshot, oneshot::Receiver},
    future::join_all,
    join,
};
use std::time::Duration;
use wasm_bindgen::JsValue;
use wasm_game_lib::{
    graphics::{canvas::*, color::*, font::*, image::*, text::*},
    system::sleep,
};

pub async fn load_images(images: Vec<&str>, mut canvas: &mut Canvas) -> Vec<Image> {
    let mut receivers = Vec::new();
    let mut senders = Vec::new();
    for _i in 0..images.len() {
        let (sender, receiver) = oneshot::channel::<Result<Image, JsValue>>();
        receivers.push(receiver);
        senders.push(sender);
    }

    let mut futures = Vec::new();
    for image in images {
        futures.push(Image::load_and_send(image, senders.remove(0)))
    }

    let results = join!(loading_tracker(receivers, &mut canvas), join_all(futures)).0;
    let mut images = Vec::new();
    for result in results {
        images.push(result.expect("failed to load an image"));
    }

    log!("ressources loaded sucessfully");

    images
}

async fn loading_tracker(
    mut receivers: Vec<Receiver<Result<Image, JsValue>>>,
    canvas: &mut Canvas,
) -> Vec<Result<Image, JsValue>> {
    let mut images = Vec::new();
    for _ in 0..receivers.len() {
        images.push(None);
    }

    let arial = Font::arial();
    let message = Text::new_with_options(
        &arial,
        String::from("Loading ressources... Please wait"),
        (100, 200),
        TextStyle::default(),
        (30, "px"),
    );

    let mut progress_bar = ProgressBar::new(receivers.len(), (100.0, 100.0), (300.0, 50.0));
    progress_bar.style.color = Color::grey();
    progress_bar.border_radius = 5.0;
    progress_bar.style.size = 2.0;
    progress_bar.background_color = Color::new(229, 229, 229);

    loop {
        for i in 0..images.len() {
            if images[i].is_none() {
                if let Ok(Some(result)) = receivers[i].try_recv() {
                    progress_bar.inc();
                    images[i] = Some(result);
                }
            }
        }

        if !images.contains(&None) {
            // break when every image is ready
            break;
        }

        canvas.clear();
        canvas.draw(&progress_bar);
        canvas.draw(&message);

        sleep(Duration::from_millis(16)).await;
    }

    let mut unwraped_images = Vec::new();
    for image in images {
        unwraped_images.push(image.unwrap());
    }

    unwraped_images
}