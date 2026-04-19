use crate::api::ProviderInfo;
use crate::components::icons::*;
use leptos::ev::MouseEvent;
use leptos::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

#[component]
pub fn ModelPicker(
    providers: ReadSignal<Vec<ProviderInfo>>,
    selected_provider: RwSignal<String>,
    selected_model: RwSignal<String>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let (open, set_open) = create_signal(false);
    let (query, set_query) = create_signal(String::new());

    let toggle = move |e: MouseEvent| {
        e.stop_propagation();
        if disabled.get_untracked() {
            return;
        }
        set_open.update(|o| *o = !*o);
    };

    create_effect(move |_| {
        if !open.get() {
            return;
        }
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            if let Some(target) = e.target() {
                if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                    if el.closest(".model-popover-anchor").ok().flatten().is_none() {
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

    let pick = move |provider: String, model: String| {
        selected_provider.set(provider);
        selected_model.set(model);
        set_open.set(false);
    };

    view! {
        <div class="popover-anchor model-popover-anchor">
            <button
                class="pill"
                disabled=move || disabled.get()
                on:click=toggle
            >
                <span class="dot" style="background: var(--persona-b);"></span>
                <span class="model-text">
                    {move || {
                        let m = selected_model.get();
                        if m.is_empty() { "Select model".to_string() } else { m }
                    }}
                </span>
                <span class="provider-badge">{move || selected_provider.get()}</span>
                <IconChevronDown/>
            </button>
            {move || if open.get() {
                let q = query.get().to_lowercase();
                let groups: Vec<_> = providers.get().into_iter().map(|p| {
                    let filtered_models: Vec<String> = p.models.iter()
                        .filter(|m| q.is_empty() || m.to_lowercase().contains(&q))
                        .cloned()
                        .collect();
                    (p.name, filtered_models)
                }).filter(|(_, m)| !m.is_empty()).collect();

                view! {
                    <div class="popover model-popover" on:click=|e| e.stop_propagation()>
                        <div class="popover-search">
                            <IconSearch/>
                            <input
                                type="text"
                                placeholder="Search models"
                                prop:value=move || query.get()
                                on:input=move |ev| set_query.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="popover-body">
                            {groups.into_iter().map(|(provider_name, models)| {
                                let pn = provider_name.clone();
                                view! {
                                    <div class="provider-group">
                                        <div class="provider-label">
                                            <span>{pn.clone()}</span>
                                            <span class="health"></span>
                                        </div>
                                        {models.into_iter().map(|m| {
                                            let pn2 = pn.clone();
                                            let m2 = m.clone();
                                            let pn_cmp = pn.clone();
                                            let m_cmp = m.clone();
                                            let is_selected = create_memo(move |_| {
                                                selected_provider.get() == pn_cmp
                                                    && selected_model.get() == m_cmp
                                            });
                                            view! {
                                                <div
                                                    class="model-row"
                                                    class:selected=move || is_selected.get()
                                                    on:click=move |_| pick(pn2.clone(), m2.clone())
                                                >
                                                    <div class="model-info">
                                                        <div class="model-name">{m}</div>
                                                    </div>
                                                    {move || if is_selected.get() {
                                                        view! {
                                                            <span class="model-check"><IconCheck/></span>
                                                        }.into_view()
                                                    } else {
                                                        view! { <span></span> }.into_view()
                                                    }}
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                }.into_view()
            } else {
                view! { <span></span> }.into_view()
            }}
        </div>
    }
}
