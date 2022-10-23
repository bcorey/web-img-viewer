use hex_color::HexColor;

pub struct ColorList<'a> {
    colors: Vec<ColorEntry<'a>>,
    index: usize,
}

impl <'a> ColorList<'a> {
    pub fn new() -> Self {
        let mut list = ColorList { 
            colors: Vec::new(), 
            index: 0, 
        };
        list.instantiate_colors();
        list
    }

    fn add(&mut self, bg_hex: &'a str, fg_hex: &'a str, accent: &'a str) {
        self.colors.push(ColorEntry::new(bg_hex, fg_hex, accent));
    }

    fn instantiate_colors(&mut self) {
        self.add("#250EAE", "#FFFFFF", "#46FF5D");
        self.add("#330835", "#FFFFFF", "#d3e775");
        self.add("#0d183a", "#FFFFFF", "#c4f941");
        self.add("#592851", "#FFFFFF", "#f1e729");
    }

    pub fn next_color(&mut self) -> ColorEntry {
        let color: ColorEntry = self.colors[*&self.index];
        self.index += 1;
        color
    }

    pub fn get_color(&self, index: usize) -> ColorEntry {
        let mut i = index;
        if i >= self.colors.len() {
            i = self.colors.len() - 1;
        }
        if i < 0 {
            i = 0;
        }
        self.colors[i]
    }
}

#[derive(Copy, Clone)]
pub struct ColorEntry<'a> {
    pub bg_rgb: HexColor,
    pub fg_rgb: HexColor,
    pub accent: &'a str,
    pub bg_hex: &'a str,
    pub fg_hex: &'a str,
}

#[derive(Copy, Clone)]
pub struct ComponentRGB {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl <'a> ColorEntry<'a> {
    fn new(bg_hex: &'a str, fg_hex: &'a str, accent: &'a str) -> Self {

        ColorEntry {
            bg_rgb: HexColor::parse_rgb(bg_hex).expect("couldn't parse hex"),
            fg_rgb: HexColor::parse_rgb(fg_hex).expect("couldn't parse hex"),
            accent,
            bg_hex,
            fg_hex,
        }
    }

    pub fn bg_rgb(&self) -> ComponentRGB {
        ComponentRGB {
            r: self.bg_rgb.r as f64 / 255.0,
            b: self.bg_rgb.b as f64 / 255.0,
            g: self.bg_rgb.g as f64 / 255.0
        }
    }
    
}