use crate::api::SystemPrompt;
use crate::components::icons::*;
use leptos::ev::MouseEvent;
use leptos::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub fn persona_color_var(name: &str) -> &'static str {
    let mut sum: u32 = 0;
    for b in name.bytes() {
        sum = sum.wrapping_add(b as u32);
    }
    match sum % 5 {
        0 => "var(--persona-a)",
        1 => "var(--persona-b)",
        2 => "var(--persona-c)",
        3 => "var(--persona-d)",
        _ => "var(--persona-e)",
    }
}

#[component]
pub fn PersonaPicker(
    prompts: ReadSignal<Vec<SystemPrompt>>,
    selected_name: RwSignal<String>,
    custom_prompt: RwSignal<String>,
) -> impl IntoView {
    let (open, set_open) = create_signal(false);

    let toggle = move |e: MouseEvent| {
        e.stop_propagation();
        set_open.update(|o| *o = !*o);
    };

    create_effect(move |_| {
        if !open.get() {
            return;
        }
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            if let Some(target) = e.target() {
                if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                    if el
                        .closest(".persona-popover-anchor")
                        .ok()
                        .flatten()
                        .is_none()
                    {
                        set_open.set(false);
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            let _ =
                doc.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref());
        }
        closure.forget();
    });

    view! {
        <div class="popover-anchor persona-popover-anchor">
            <button class="pill" on:click=toggle>
                <span
                    class="dot"
                    style:background=move || persona_color_var(&selected_name.get()).to_string()
                ></span>
                <span>{move || selected_name.get()}</span>
                <IconChevronDown/>
            </button>
            {move || if open.get() {
                view! {
                    <div class="popover persona-popover" on:click=|e| e.stop_propagation()>
                        <div class="popover-body">
                            {prompts.get().into_iter().map(|p| {
                                let name = p.name.clone();
                                let name_sel = p.name.clone();
                                let name_cmp = p.name.clone();
                                let color = persona_color_var(&p.name);
                                let suggested = p.suggested_models.join(", ");
                                let is_selected = create_memo(move |_| selected_name.get() == name_cmp);
                                view! {
                                    <div
                                        class="persona-row"
                                        class:selected=move || is_selected.get()
                                        on:click=move |_| {
                                            selected_name.set(name_sel.clone());
                                        }
                                    >
                                        <span class="persona-swatch" style:background=color.to_string()></span>
                                        <div class="persona-info">
                                            <div class="persona-name">{name}</div>
                                            {if suggested.is_empty() {
                                                view! { <span></span> }.into_view()
                                            } else {
                                                view! { <div class="persona-desc">{suggested}</div> }.into_view()
                                            }}
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                        {move || if selected_name.get() == "Custom" {
                            view! {
                                <div class="persona-custom">
                                    <div class="persona-custom-label">"Custom prompt"</div>
                                    <textarea
                                        placeholder="Enter your custom system prompt..."
                                        prop:value=move || custom_prompt.get()
                                        on:input=move |ev| custom_prompt.set(event_target_value(&ev))
                                    />
                                </div>
                            }.into_view()
                        } else {
                            view! { <span></span> }.into_view()
                        }}
                    </div>
                }.into_view()
            } else {
                view! { <span></span> }.into_view()
            }}
        </div>
    }
}
