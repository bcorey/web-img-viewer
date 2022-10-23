use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use web_sys::File;

pub fn get_file(id: &str) -> Option<File> {
    log::info!("getting file");
    let input_el = match web_sys::window()
        .unwrap().document()
        .unwrap().get_element_by_id(id)
        {
            Some(element) => element,
            None => return None
        };
    let input = input_el.dyn_into::<web_sys::HtmlInputElement>()
    .map_err(|_| ())
    .expect("Should have found input field");


    let items = match input.files() {
        Some(file) => file,
        None => return None
    };
    let file = match items.get(0) {
        Some(n) => return Some(n),
        None => return None
    };
     
}


use web_sys::Url;
use web_sys::ImageData;
use web_sys::CanvasRenderingContext2d;
use web_sys::ImageBitmap;
use web_sys::HtmlImageElement;

use crate::render_pipeline::WebImage;
use wasm_logger;


pub async fn canvas_decode(file: File) -> Option<WebImage> {
    let url = Url::create_object_url_with_blob(file.as_ref()).unwrap();
    let canvas = get_element_by_id("decode-canvas").dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();    
    
    let img_el = HtmlImageElement::new().unwrap();
    img_el.set_src(url.as_str());

    async {
        while !img_el.complete() {
            wasm_timer::Delay::new(core::time::Duration::from_millis(1)).await;
        }
    }.await;

    let img_width = img_el.natural_width();
    let img_height = img_el.natural_height();
    
    canvas.set_height(img_height);
    canvas.set_width(img_width);
    
    if let Some(canvas_context) = canvas.get_context("2d").unwrap() {
        let canvas_context = canvas_context.dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
        canvas_context.draw_image_with_html_image_element(&img_el, 0.0, 0.0).expect("failed to draw image");
        let data = canvas_context.get_image_data(0.0, 0.0, img_width as f64, img_height as f64)
            .expect("failed to get image data")
            .data()
            .to_vec();

        return Some(WebImage {
            width: img_width,
            height: img_height,
            data: data
        });
    };

    None
}

use web_sys::Element;
pub fn get_element_by_id(id: &str) -> Element {
    web_sys::window()
    .and_then(|win| win.document())
    .and_then(|doc| {
        doc.get_element_by_id(id)
    })
    .unwrap()
}


pub fn save_canvas() {
    let canvas = get_element_by_id("canvas").dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    
    let image = canvas.to_data_url().unwrap();

    let anchor = get_element_by_id("download-anchor").dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| ())
        .unwrap();
    
    anchor.set_attribute("download", "render.png");
    anchor.set_href(image.as_str());
    anchor.click();
    
}