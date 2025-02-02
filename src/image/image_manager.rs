use std::{
    path::{Path, PathBuf},
    usize,
};

use ahash::{HashMap, HashMapExt};

use super::image::Image;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageId(usize);

impl ImageId {
    pub const NULL: ImageId = ImageId(usize::MAX);
}

pub struct ImageManager {
    images: Vec<Image>,
    events: Vec<ImageManagerEvent>,
    images_by_path: HashMap<PathBuf, ImageId>,
}

impl ImageManager {
    pub fn new() -> Self {
        return Self {
            images: Vec::new(),
            events: Vec::new(),
            images_by_path: HashMap::new(),
        };
    }

    pub fn add_image(&mut self, image: Image) -> ImageId {
        self.images.push(image);
        let id = ImageId(self.images.len() - 1);
        self.events.push(ImageManagerEvent::ImageCreated(id));
        id
    }

    pub fn load_image_by_path(&mut self, path: &Path) -> (ImageId, &Image) {
        let image_id = match self.images_by_path.get(path) {
            Some(&image_id) => image_id,
            None => {
                let image = Image::from_path(path);
                let image_id = self.add_image(image);
                self.images_by_path.insert(path.to_path_buf(), image_id);
                image_id
            }
        };

        (image_id, self.get_image(image_id))
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
