use leptos::*;
use leptos::ev::MouseEvent;
use web_sys::window;
use std::rc::Rc;

const MIN_INPUT_HEIGHT: i32 = 120;  // Minimum height for input area
const MAX_INPUT_HEIGHT: i32 = 600;  // Maximum height for input area  
const DEFAULT_INPUT_HEIGHT: i32 = 200; // Default height

#[component]
pub fn ResizeHandle(
    on_resize: impl Fn(i32) + 'static,
) -> impl IntoView {
    let (is_dragging, set_is_dragging) = create_signal(false);
    let (start_y, set_start_y) = create_signal(0);
    let (start_height, set_start_height) = create_signal(DEFAULT_INPUT_HEIGHT);
    
    // Wrap the callback in Rc to allow sharing
    let on_resize = Rc::new(on_resize);
    let on_resize_init = Rc::clone(&on_resize);
    let on_resize_drag = Rc::clone(&on_resize);
    
    // Load saved height from localStorage on mount
    create_effect(move |_| {
        if let Ok(Some(storage)) = window().unwrap().local_storage() {
            if let Ok(Some(saved_height)) = storage.get_item("input_area_height") {
                if let Ok(height) = saved_height.parse::<i32>() {
                    let clamped = height.max(MIN_INPUT_HEIGHT).min(MAX_INPUT_HEIGHT);
                    on_resize_init(clamped);
                }
            } else {
                on_resize_init(DEFAULT_INPUT_HEIGHT);
            }
        } else {
            on_resize_init(DEFAULT_INPUT_HEIGHT);
        }
    });
    
    let handle_mouse_down = move |e: MouseEvent| {
        e.prevent_default();
        set_is_dragging.set(true);
        set_start_y.set(e.client_y());
        
        // Get current height from localStorage or use default
        let current_height = if let Ok(Some(storage)) = window().unwrap().local_storage() {
            if let Ok(Some(saved_height)) = storage.get_item("input_area_height") {
                saved_height.parse::<i32>().unwrap_or(DEFAULT_INPUT_HEIGHT)
            } else {
                DEFAULT_INPUT_HEIGHT
            }
        } else {
            DEFAULT_INPUT_HEIGHT
        };
        
        set_start_height.set(current_height);
        
        // Add class to body to show resize cursor everywhere
        if let Some(body) = window().unwrap().document().and_then(|d| d.body()) {
            let _ = body.class_list().add_1("resizing");
        }
    };
    
    // Handle mouse move globally when dragging
    let _move_listener = window_event_listener(ev::mousemove, move |e| {
        if !is_dragging.get() {
            return;
        }
        
        let delta_y = start_y.get() - e.client_y();
        let new_height = start_height.get() + delta_y;
        
        // Clamp to min/max
        let clamped = new_height.max(MIN_INPUT_HEIGHT).min(MAX_INPUT_HEIGHT);
        on_resize_drag(clamped);
        
        // Save to localStorage
        if let Ok(Some(storage)) = window().unwrap().local_storage() {
            let _ = storage.set_item("input_area_height", &clamped.to_string());
        }
    });
    
    // Handle mouse up globally
    let _up_listener = window_event_listener(ev::mouseup, move |_| {
        if is_dragging.get() {
            set_is_dragging.set(false);
            
            // Remove resize class from body
            if let Some(body) = window().unwrap().document().and_then(|d| d.body()) {
                let _ = body.class_list().remove_1("resizing");
            }
        }
    });
    
    view! {
        <div 
            class="resize-handle"
            class:dragging=move || is_dragging.get()
            on:mousedown=handle_mouse_down
        >
            <div class="resize-handle-bar"></div>
        </div>
    }
}