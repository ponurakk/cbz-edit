use std::{io::Cursor, sync::mpsc};

use image::ImageReader;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

use crate::ui::spinner::SpinnerState;

pub enum ImagesState {
    Loading,
    Ready(Vec<StatefulProtocol>),
}

pub struct ImageManager {
    pub picker: Picker,
    pub images: ImagesState,
    pub raw_images_rx: Option<mpsc::Receiver<Vec<Vec<u8>>>>,
    pub images_rx: Option<mpsc::Receiver<Vec<StatefulProtocol>>>,
    pub current: usize,
    pub spinner: SpinnerState,
}

impl ImageManager {
    pub fn new(picker: Picker) -> Self {
        Self {
            picker,
            images: ImagesState::Loading,
            raw_images_rx: None,
            images_rx: None,
            current: 0,
            spinner: SpinnerState::default(),
        }
    }

    pub fn next(&mut self) {
        if let ImagesState::Ready(images) = &self.images
            && self.current < images.len() - 1
        {
            self.current += 1;
        }
    }

    pub fn prev(&mut self) {
        self.current = self.current.saturating_sub(1);
    }

    pub fn replace_images(&mut self, images: Vec<Vec<u8>>) {
        let (tx, rx) = mpsc::channel();
        self.images_rx = Some(rx);
        self.images = ImagesState::Loading;
        self.current = 0;

        let picker = self.picker.clone();

        tokio::spawn(async move {
            let mut protocols: Vec<StatefulProtocol> = Vec::new();
            for img_bytes in images {
                let decoded = (|| -> Result<_, image::ImageError> {
                    let reader = ImageReader::new(Cursor::new(img_bytes)).with_guessed_format()?;
                    reader.decode()
                })();

                match decoded {
                    Ok(dyn_img) => {
                        let proto = picker.new_resize_protocol(dyn_img);
                        protocols.push(proto);
                    }
                    Err(err) => error!("Image decode failed: {err}"),
                }
            }

            let _ = tx.send(protocols);
        });
    }

    pub fn poll_image_updates(&mut self) {
        if let Some(rx) = &self.images_rx
            && let Ok(protocols) = rx.try_recv()
        {
            self.images = ImagesState::Ready(protocols);
            self.images_rx = None;
        }
    }
}
