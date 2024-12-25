use std::usize;

use super::image::Image;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageId(usize);

impl ImageId {
    pub const NULL: ImageId = ImageId(usize::MAX);
}

pub struct ImageManager {
    images: Vec<Image>,
    events: Vec<ImageManagerEvent>,
}

impl ImageManager {
    pub fn new() -> Self {
        return Self {
            images: Vec::new(),
            events: Vec::new(),
        };
    }

    pub fn add_image(&mut self, image: Image) -> ImageId {
        self.images.push(image);
        let id = ImageId(self.images.len() - 1);
        self.events.push(ImageManagerEvent::ImageCreated(id));
        id
    }

    pub fn get_image(&self, image_id: ImageId) -> &Image {
        &self.images[image_id.0]
    }

    pub fn collect_events(&mut self) -> Vec<ImageManagerEvent> {
        std::mem::replace(&mut self.events, Vec::new())
    }
}

pub enum ImageManagerEvent {
    ImageCreated(ImageId),
}
