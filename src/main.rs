use dioxus::prelude::*;
use dioxus::events::{MouseEvent, FormEvent};
use img_render::run;
use img_render::WebImage;
use img_render::FrontendEvent;
use winit::event_loop::{EventLoopProxy};
// use image::DynamicImage;


use wasm_logger;

mod image_decode;
use image_decode::{get_file, manual_decode, canvas_decode, save_canvas};

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    dioxus::web::launch(app);
}

fn dims(id: &str) -> (f64, f64) {
    let parent_rect = web_sys::window()
    .unwrap().document()
    .unwrap().get_element_by_id(id)
    .unwrap().get_bounding_client_rect();
    (parent_rect.width(), parent_rect.height())
}

fn prepare_img(cx: Scope, proxy: &Option<EventLoopProxy<FrontendEvent>>) {
    cx.spawn({
        let proxy_2 = proxy.clone();
        async move {
            if let Some(file) = get_file("img") {
                let n = canvas_decode(file).await;
                match n {
                    Some(img) => send_img(&proxy_2, img),
                    _ => ()
                };
            }
        }
    })
}

fn send_shader_event(cx: Scope, event: FrontendEvent) {
    let proxy = use_read(&cx, PROXY);

    if let Some(proxy) = proxy {
        proxy.send_event(event);
    }
}

fn send_img(proxy: &Option<EventLoopProxy<FrontendEvent>>, img: WebImage) {
    if let Some(proxy) = proxy {
        proxy.send_event(FrontendEvent::NewImage(img));
    }
}

static PROXY: Atom<Option<EventLoopProxy<FrontendEvent>>> = |_| None;
static PIPELINE_STATUS: Atom<bool> = |_| false;
static INITIAL_PAGE_LOAD: Atom<bool> = |_| false;
static VALUE: Atom<i32> = |_| 0;

fn start(cx: Scope) {
    let pipeline_status = use_read(&cx, PIPELINE_STATUS);
    let set_pipeline_status = use_set(&cx, PIPELINE_STATUS);
    let set_proxy = use_set(&cx, PROXY);
    let initial_load = use_read(&cx, INITIAL_PAGE_LOAD);
    let set_initial_load = use_set(&cx, INITIAL_PAGE_LOAD);


    if !pipeline_status && *initial_load {
        let (view_width, view_height) = dims("parent");
        set_proxy(Some(run(view_width, view_height)));
        set_pipeline_status(true);
    } 

    if !initial_load {
        set_initial_load(true);
    }
}

fn app(cx: Scope) -> Element {
    //let mut value = use_state(&cx, || 0);
    let value = use_read(&cx, VALUE);
    let set_value = use_set(&cx, VALUE);
    let proxy = use_read(&cx, PROXY);

    log::info!("updating ui");
    start(cx);

    cx.render(rsx!{ 
        DecodeCanvas {}
        DownloadAnchor {}
        div {
            class: "row row-center",
            Canvas {}
        }  

        div{
            class: "container",
            div {
                class: "row row-center",
                VoteButton {
                    name: "+",
                    onclick: move |_| set_value(value + 1),
                }
                div {
                    class: "column column-25",
                    style: "display: table;",
                    h6 { 
                        style: "display: table-cell; vertical-align: middle;",
                        "{value}" 
                    }
                }
                VoteButton {
                    name: "-",
                    onclick: move |_| set_value(value - 1),
                }
            }
            div {
                class: "row row-center",
                FileInput {
                    file_types: "image",
                    id: "img",
                    oninput: move |_| prepare_img(cx, proxy),
                }
                VoteButton {
                    name: "Step",
                    onclick: move |_| send_shader_event(cx, FrontendEvent::STEP),
                }
                VoteButton {
                    name: "Fill Mode",
                    onclick: move |_| send_shader_event(cx, FrontendEvent::FILL_MODE),
                }
                VoteButton {
                    name: "Save",
                    onclick: move |_| save_canvas(),
                }
            }

        }
    })
}

//borrowed props (no partialeq)
#[derive(Props)] 
struct VoteButtonProps<'a> {
    name: &'a str,
    onclick: EventHandler<'a, MouseEvent>
}

fn VoteButton<'a>(cx: Scope<'a,VoteButtonProps<'a>>) -> Element {
    cx.render(rsx!{
        div {
            class: "column column-25",
            button {
                class: "button button-outline",
                onclick: move |evt| cx.props.onclick.call(evt),
                "{cx.props.name}"
            }
        }
    })
}

fn Canvas(cx: Scope) -> Element {
    cx.render(rsx!{
        div {
            class: "",
            id: "parent",
            canvas {
                id: "canvas",
                prevent_default: "onclick",
                onclick: move |_|{
                    // handle the event without navigating the page.
                }
            }
        }
    })
}

fn DecodeCanvas(cx: Scope) -> Element {
    cx.render(rsx!{
        canvas {
            id: "decode-canvas",
            style: "display: none"
        }
    })
}

fn DownloadAnchor(cx: Scope) -> Element {
    cx.render(rsx!{
        a {
            id: "download-anchor",
            style: "display: none"
        }
    })
}

#[derive(Props)]
struct FileInputProps<'a> {
    file_types: &'a str,
    id: &'a str,
    oninput: EventHandler<'a, FormEvent>
}

fn FileInput<'a>(cx: Scope<'a, FileInputProps<'a>>) -> Element {
    cx.render(rsx!{
        div {
            class: "column column-25",
            label {
                r#for: format_args!("{}", cx.props.id),
                class: "button button-solid",
                "Choose File"
            }
            input {
                r#type: "file",
                id: format_args!("{}", cx.props.id),
                name: format_args!("{}", cx.props.id),
                accept: format_args!("{}", cx.props.file_types),
                oninput: move |evt| cx.props.oninput.call(evt)
            }
        }
    })
}