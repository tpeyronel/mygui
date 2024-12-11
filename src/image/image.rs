#[derive(Debug, Clone)]
pub struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
    pitch: u32, // Bytes per row. Not necessarily the same as width.
}

impl Image {
    pub fn new_empty(width: u32, height: u32) -> Self {
        Self {
            data: vec![0u8; (width * height) as usize],
            width,
            height,
            pitch: width,
        }
    }

    pub fn from_data(data: Vec<u8>, width: u32, height: u32, pitch: u32) -> Self {
        Self {
            data,
            width,
            height,
            pitch,
        }
    }

    fn coords_to_index(&self, x: u32, y: u32) -> usize {
        assert!(x < self.width);
        assert!(y < self.height);

        (y * self.pitch + x) as usize
    }

    pub fn get(&self, x: u32, y: u32) -> u8 {
        self.data[self.coords_to_index(x, y)]
    }

    pub fn set(&mut self, x: u32, y: u32, value: u8) {
        let index = self.coords_to_index(x, y);
        self.data[index] = value;
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn pitch(&self) -> u32 {
        self.pitch
    }
}

impl ToString for Image {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for y in 0..self.height {
            let y = self.height - y - 1;
            for x in 0..self.width {
                let c = match self.get(x, y) {
                    0 => ' ',
                    255 => '$',
                    _ => '+',
                };
                s.push(c);
            }
            s.push('\n');
        }
        s
    }
}
