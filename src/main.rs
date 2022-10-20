use std::collections::HashMap;
use std::fmt::Pointer;
use std::hash::Hash;

use dioxus::prelude::*;
use dioxus::events::{MouseEvent, FormEvent, PointerData};
use dioxus::core::UiEvent;
use img_render::run;
use img_render::WebImage;
use img_render::FrontendEvent;
use winit::event_loop::{EventLoopProxy};
// use image::DynamicImage;


use wasm_logger;

mod image_decode;
use image_decode::{get_file, canvas_decode, save_canvas};

mod color_management;
use color_management::*;

#[allow(non_snake_case)]

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

static POSITION: Atom<(f64, f64)> = |_| (40f64, 40f64);

static POSITIONS: Atom<HashMap<String, (f64, f64)>> = |_| HashMap::new();


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

struct DraggableState {
    id: String,
    start_pos: (f64, f64),
    pos: (f64, f64),
}

fn app(cx: Scope) -> Element {
    //let mut value = use_state(&cx, || 0);
    let value = use_read(&cx, VALUE);
    let set_value = use_set(&cx, VALUE);
    let proxy = use_read(&cx, PROXY);

    log::info!("updating ui");
    start(cx);  

    // Position on the square where drag was started
    // If we are not currently dragging, `None`
    let active_draggable: &UseState<Option<DraggableState>> = use_state(&cx, || None);
    let positions: &UseState<HashMap<String, (f64, f64)>> = use_state(&cx, || HashMap::new());  

    let mut draggables: Vec<DraggableState> = Vec::new();

    let (view_width, view_height) = dims("main");

    let max = 10;
    for i in 0..max {
        let el = format!("el-{}", i);

        if !positions.current().contains_key(&el) { // if key does not yet exist
            positions.make_mut().insert(el.clone(), ((view_width / 2.0) + (100.0 * i as f64) - (30.0 * max as f64), view_height / 2.0 )); // set position to initial value
        }

        let mut drag = DraggableState { // make new draggable
            pos: *positions.current().get(&el).expect("missing position"),
            start_pos: (0.0, 0.0),
            id: el,
        };

        draggables.push(drag);
    }

    // When user starts dragging, track where on the square they started
    let mouse_down_handler =
        move |event: UiEvent<PointerData>, el: String| {
            log::info!("mouse down!");
            let pos = *positions.current().get(&el).expect("missing position");
            active_draggable.set(Some(DraggableState {
                start_pos: (event.data.page_x as f64 - pos.0, 
                    event.data.page_y as f64 - pos.1),
                pos: *positions.current().get(&el).expect("missing mousedown position"),
                id: el,
            }));
            
        };
    
    // When the mouse moves on the container
    let mut mouse_move_handler = move |event: UiEvent<PointerData>| {
        // If we are currently dragging the square
        if let Some(active) = &**active_draggable {
            // Calculate the new coordinates
            // (Offset by the square coordinates we started dragging on, otherwise, we would drag the top-left corner)
            let (s_x, s_y) = active.start_pos;
            let x = (event.data.page_x as f64 - s_x as f64);
            let y = (event.data.page_y as f64 - s_y as f64);

            positions.make_mut().insert(active.id.clone(), (x, y));
        }
    };
    
    // When mouse is released, stop dragging
    let mouse_up_handler = move |_: UiEvent<PointerData>| {
        active_draggable.set(None);
    };

    let pos1 = draggables[0].pos;
    let pos2 = draggables[1].pos;
    let pos3 = draggables[2].pos;
    let pos4 = draggables[3].pos;
    let pos5 = draggables[4].pos;
    let pos6 = draggables[5].pos;
    let pos7 = draggables[6].pos;


    for draggable in draggables {

    }

    cx.render(rsx!{ 
        DecodeCanvas {}
        DownloadAnchor {}
        
        div {
            class: "row row-center",
            Canvas {}
        }  
        div {
            width: "100vw",
            height: "100vh",
            style: "overflow: hidden; height: 100vh; width: 100vw; position: absolute; top: 0; left: 0",
            //u
            onpointermove: mouse_move_handler,
            onpointerup: mouse_up_handler,
            Draggable {
                onpointerdown: move |evt| mouse_down_handler(evt, "el-0".to_string()),
                pos: pos1,
                div {
                    class: "button-row",
                    div {
                        class: "button-column",
                        VoteButton {
                            name: "+",
                            onclick: move |_| set_value(value + 1),
                        }
                    }
                    div {
                        class: "button-spacer"
                    }
                    div {
                        class: "button-column",
                        VoteButton {
                            name: "-",
                            onclick: move |_| set_value(value - 1),
                        }
                    }
                }
                div {
                    style: "display: table; width: 100%",
                    h6 { 
                        style: "display: table-cell; vertical-align: middle; width: 100%; text-align: center; height: 3.8rem",
                        "{value}" 
                    }
                }
                div {
                    class: "button-row",
                    FileInput {
                        file_types: "image",
                        id: "img",
                        oninput: move |_| prepare_img(cx, proxy),
                    }
                }
                div {
                    class: "button-row",
                    VoteButton {
                        name: "Effect",
                        onclick: move |_| send_shader_event(cx, FrontendEvent::STEP),
                    }
                }
                div {
                    class: "button-row",
                    VoteButton {
                        name: "Fill",
                        onclick: move |_| send_shader_event(cx, FrontendEvent::FILL_MODE),
                    }
                }
                div {
                    class: "button-row",
                    VoteButton {
                        name: "Save",
                        onclick: move |_| save_canvas(),
                    }
                } 
            }
            Draggable { // light
                onpointerdown: move |evt| mouse_down_handler(evt, "el-1".to_string()),
                pos: pos2,
                div {
                    class: "button-row",
                    VoteButton {
                        name: "Light",
                        onclick: move |_| (),
                    }
                }
                div {
                    class: "button-row",
                    VoteButton {
                        name: "Contrast",
                        onclick: move |_| (),
                    }
                }

            }
            Draggable { // position
                onpointerdown: move |evt| mouse_down_handler(evt, "el-2".to_string()),
                pos: pos3,
                div {
                    class: "button-row",
                    VoteButton {
                        name: "Fill",
                        onclick: move |_| (),
                    }
                }
                div {
                    class: "button-row",
                    VoteButton {
                        name: "Fit",
                        onclick: move |_| (),
                    }
                }
                div {
                    class: "button-row",
                    div {
                        class: "button-column",
                        VoteButton {
                            name: "&#xbf;",
                            onclick: move |_| set_value(value + 1),
                        }
                    }
                    div {
                        class: "button-spacer"
                    }
                    div {
                        class: "button-column",
                        VoteButton {
                            name: "?",
                            onclick: move |_| set_value(value - 1),
                        }
                    }
                }
                
            }
            Draggable { // layers
                onpointerdown: move |evt| mouse_down_handler(evt, "el-3".to_string()),
                pos: pos4,
                div {
                    class: "button-row",
                    VoteButton {
                        name: "+",
                        onclick: move |_| (),
                    }
                }
                div {
                    class: "button-row",
                    VoteButton {
                        name: "Two",
                        onclick: move |_| (),
                    }
                }
                div {
                    class: "button-row",
                    VoteButton {
                        name: "One",
                        onclick: move |_| (),
                    }
                }
            }
            Draggable { //color
                onpointerdown: move |evt| mouse_down_handler(evt, "el-4".to_string()),
                pos: pos5,
                div {
                    class: "button-row",
                    div {
                        class: "button-column",
                        VoteButton {
                            name: "<",
                            onclick: move |_| set_value(value + 1),
                        }
                    }
                    div {
                        class: "button-spacer"
                    }
                    div {
                        class: "button-column",
                        VoteButton {
                            name: ">",
                            onclick: move |_| set_value(value - 1),
                        }
                    }
                }
                div {
                    class: "button-row",
                    VoteButton {
                        name: "Invert",
                        onclick: move |_| (),
                    }
                }
            }
            Draggable { //color
                onpointerdown: move |evt| mouse_down_handler(evt, "el-5".to_string()),
                pos: pos6,
                div {
                    class: "button-row",
                    div {
                        class: "button-column",
                        VoteButton {
                            name: "^",
                            onclick: move |_| set_value(value + 1),
                        }
                    }
                    div {
                        class: "button-spacer"
                    }
                    div {
                        class: "button-column",
                        VoteButton {
                            name: "v",
                            onclick: move |_| set_value(value - 1),
                        }
                    }
                }
                div {
                    class: "button-row",
                    VoteButton {
                        name: "Invert",
                        onclick: move |_| (),
                    }
                }
                div {
                    class: "button-row",
                    VoteButton {
                        name: "Invert",
                        onclick: move |_| (),
                    }
                }
                div {
                    class: "button-row",
                    VoteButton {
                        name: "Invert",
                        onclick: move |_| (),
                    }
                }
                div {
                    class: "button-row",
                    VoteButton {
                        name: "Invert",
                        onclick: move |_| (),
                    }
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
            button {
                class: "button button-outline",
                width: "100%",
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
                width: "100%",
                "Image"
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

#[derive(Props)] 
struct DraggableProps<'a> {
    onpointerdown: EventHandler<'a, UiEvent<PointerData>>,
    pos: (f64, f64),
    children: Element<'a>
}

fn Draggable<'a>(cx: Scope<'a, DraggableProps<'a>>) -> Element {
    cx.render(rsx!{
        div {
            class: "draggable",
            left: "{cx.props.pos.0}px",
            top: "{cx.props.pos.1}px",
            DragIcon {
                onpointerdown:  move |evt| cx.props.onpointerdown.call(evt),
            }
            &cx.props.children,
        }
    })
}

#[derive(Props)]
struct DragIconProps<'a> {
    onpointerdown: EventHandler<'a, UiEvent<PointerData>>,
}

fn DragIcon<'a>(cx: Scope<'a, DragIconProps<'a>>) -> Element {
    cx.render(rsx!{
        div {
            class: "arrows-box",
            div {
                onpointerdown: move |evt| cx.props.onpointerdown.call(evt),

                class: "arrows",
                div {
                    class: "arrow arrow-left",
                }
                div {
                    class: "arrow arrow-up",
                }
                div {
                    class: "arrow arrow-right",
                }
                div {
                    class: "arrow arrow-down",
                }
            }
        }
    })
}
