use leptos::ev::MouseEvent;
use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::window;

const MIN_WIDTH: i32 = 220;
const MAX_WIDTH: i32 = 480;
const DEFAULT_WIDTH: i32 = 280;
const STORAGE_KEY: &str = "sidebar_width";

pub fn load_saved_width() -> i32 {
    if let Ok(Some(storage)) = window().unwrap().local_storage() {
        if let Ok(Some(saved)) = storage.get_item(STORAGE_KEY) {
            if let Ok(w) = saved.parse::<i32>() {
                return w.clamp(MIN_WIDTH, MAX_WIDTH);
            }
        }
    }
    DEFAULT_WIDTH
}

fn set_css_var(width: i32) {
    if let Some(doc) = window().and_then(|w| w.document()) {
        if let Some(root) = doc.document_element() {
            if let Ok(html) = root.dyn_into::<web_sys::HtmlElement>() {
                let _ = html
                    .style()
                    .set_property("--sidebar-width", &format!("{}px", width));
            }
        }
    }
}

#[component]
pub fn SidebarResize(width: RwSignal<i32>) -> impl IntoView {
    let (is_dragging, set_is_dragging) = create_signal(false);
    let (start_x, set_start_x) = create_signal(0);
    let (start_width, set_start_width) = create_signal(DEFAULT_WIDTH);

    create_effect(move |_| {
        set_css_var(width.get());
    });

    let handle_mouse_down = move |e: MouseEvent| {
        e.prevent_default();
        set_is_dragging.set(true);
        set_start_x.set(e.client_x());
        set_start_width.set(width.get_untracked());

        if let Some(body) = window().and_then(|w| w.document()).and_then(|d| d.body()) {
            let _ = body.class_list().add_1("resizing-col");
        }
    };

    let _move_listener = window_event_listener(ev::mousemove, move |e| {
        if !is_dragging.get_untracked() {
            return;
        }
        let delta = e.client_x() - start_x.get_untracked();
        let new_w = (start_width.get_untracked() + delta).clamp(MIN_WIDTH, MAX_WIDTH);
        width.set(new_w);
    });

    let _up_listener = window_event_listener(ev::mouseup, move |_| {
        if is_dragging.get_untracked() {
            set_is_dragging.set(false);
            if let Some(body) = window().and_then(|w| w.document()).and_then(|d| d.body()) {
                let _ = body.class_list().remove_1("resizing-col");
            }
            if let Ok(Some(storage)) = window().unwrap().local_storage() {
                let _ = storage.set_item(STORAGE_KEY, &width.get_untracked().to_string());
            }
        }
    });

    view! {
        <div
            class="sidebar-resize"
            class:dragging=move || is_dragging.get()
            on:mousedown=handle_mouse_down
        ></div>
    }
}
