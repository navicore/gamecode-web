use crate::components::context_manager::ContextManager;
use crate::components::icons::*;
use leptos::ev::KeyboardEvent;
use leptos::*;

const MAX_TOKENS: usize = 4096;

#[component]
pub fn Composer(
    input_value: RwSignal<String>,
    is_streaming: ReadSignal<bool>,
    temperature: RwSignal<f32>,
    context_manager: ContextManager,
    on_submit: Callback<()>,
) -> impl IntoView {
    let handle_keydown = move |e: KeyboardEvent| {
        if e.key() == "Enter" && !e.shift_key() {
            e.prevent_default();
            on_submit.call(());
        }
    };

    let can_send = move || !is_streaming.get() && !input_value.get().trim().is_empty();

    let cm_tokens = context_manager.clone();
    let cm_pct = context_manager.clone();
    let pct = create_memo(move |_| cm_pct.get_usage_percentage());
    let tokens = create_memo(move |_| cm_tokens.get_total_tokens());
    let warn_cls = create_memo(move |_| pct.get() > 70.0);
    let crit_cls = create_memo(move |_| pct.get() > 90.0);

    let ctx_text = move || {
        let t = tokens.get();
        if t < 1000 {
            format!("{} tok", t)
        } else {
            format!("{:.1}k / {}k", t as f64 / 1000.0, MAX_TOKENS / 1000)
        }
    };

    view! {
        <div class="composer-wrap">
            <div class="composer">
                <textarea
                    class="composer-input"
                    placeholder="Message Gamecode..."
                    rows="1"
                    prop:value=move || input_value.get()
                    on:input=move |ev| input_value.set(event_target_value(&ev))
                    on:keydown=handle_keydown
                    disabled=move || is_streaming.get()
                />
                <div class="composer-toolbar">
                    <div class="temp-control" title="Temperature">
                        <IconThermometer/>
                        <span class="temp-label">"TEMP"</span>
                        <input
                            type="range"
                            class="temp-slider"
                            min="0.0"
                            max="1.0"
                            step="0.1"
                            prop:value=move || temperature.get().to_string()
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<f32>() {
                                    temperature.set(v);
                                }
                            }
                        />
                        <span class="temp-value">{move || format!("{:.1}", temperature.get())}</span>
                    </div>
                    <div class="toolbar-spacer"></div>
                    <button
                        class="send-btn"
                        class:streaming=move || is_streaming.get()
                        disabled=move || !can_send() && !is_streaming.get()
                        on:click=move |_| {
                            if !is_streaming.get() {
                                on_submit.call(());
                            }
                        }
                        title=move || if is_streaming.get() { "Stop" } else { "Send" }
                    >
                        {move || if is_streaming.get() {
                            view! { <IconStop/> }.into_view()
                        } else {
                            view! { <IconSend/> }.into_view()
                        }}
                    </button>
                </div>
            </div>
            <div class="composer-footnote">
                <div class="ctx-gauge" title="Context window usage">
                    <div class="ctx-bar">
                        <div
                            class="ctx-fill"
                            class:warn=move || warn_cls.get()
                            class:crit=move || crit_cls.get()
                            style:width=move || format!("{}%", pct.get().min(100.0))
                        ></div>
                    </div>
                    <span class="ctx-text">{ctx_text}</span>
                </div>
                <div>
                    <span class="kbd">"↵"</span>" send · "
                    <span class="kbd">"⇧↵"</span>" newline"
                </div>
            </div>
        </div>
    }
}
