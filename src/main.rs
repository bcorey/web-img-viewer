use dioxus::prelude::*;
use dioxus::events::{MouseEvent, FormEvent};

fn main() {
    dioxus::web::launch(app);
}

fn app(cx: Scope) -> Element {
    let mut value = use_state(&cx, || 0);

    cx.render(rsx!{    
        div{
            class: "container",
            div {
                class: "row row-center",
                h3 { 
                    "hello dioxus!"
                }
            }
            div {
                class: "row row-center",
                VoteButton {
                    name: "+",
                    onclick: move |_| value += 1,
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
                    onclick: move |_| value -= 1,
                }
                FileInput {
                    file_types: "image",
                    id: "img",
                    oninput: move |_| value += 100,
                }
            }
            div {
                class: "row row-center",
                Canvas {}
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
            class: "column column-100",
            canvas {
                id: "drawing-canvas",
            }
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