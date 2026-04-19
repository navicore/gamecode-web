use crate::components::persona_picker::persona_color_var;
use crate::notebook::{Cell, CellContent, CellId, Notebook};
use leptos::*;

#[derive(Clone)]
pub struct CellContext {
    pub user_initial: String,
    pub persona_name: String,
}

#[component]
pub fn CellView(cell: Cell, ctx: CellContext, notebook: ReadSignal<Notebook>) -> impl IntoView {
    match cell.content {
        CellContent::UserInput { text } => {
            let initial = ctx.user_initial.clone();
            view! {
                <div class="msg">
                    <div class="msg-rail">
                        <div class="msg-avatar user">{initial}</div>
                        <div class="persona-line" style="background: var(--ink-4);"></div>
                    </div>
                    <div class="msg-body">
                        <div class="msg-head">
                            <span class="msg-author">"You"</span>
                            <span class="msg-meta">{format_timestamp(&cell.timestamp)}</span>
                        </div>
                        <div class="msg-content">
                            <p>{text}</p>
                        </div>
                    </div>
                </div>
            }
            .into_view()
        }

        CellContent::TextResponse { .. } => {
            let persona = ctx.persona_name.clone();
            let color = persona_color_var(&persona);
            let model_tag = cell
                .metadata
                .model
                .clone()
                .unwrap_or_else(|| "assistant".to_string());
            let cell_id = cell.id;
            let timestamp = cell.timestamp;

            let streaming = create_memo(move |_| {
                live_text_response(notebook, cell_id)
                    .map(|(_, s)| s)
                    .unwrap_or(false)
            });
            let text = create_memo(move |_| {
                live_text_response(notebook, cell_id)
                    .map(|(t, _)| t)
                    .unwrap_or_default()
            });

            view! {
                <div class="msg">
                    <div class="msg-rail">
                        <div class="msg-avatar assistant">"ai"</div>
                        <div class="persona-line" style:background=color.to_string()></div>
                    </div>
                    <div class="msg-body">
                        <div class="msg-head">
                            <span class="msg-author">{persona.clone()}</span>
                            <span class="msg-persona-tag">
                                <span class="dot" style:background=color.to_string()></span>
                                <span>{model_tag}</span>
                            </span>
                            <span class="msg-meta">{format_timestamp(&timestamp)}</span>
                        </div>
                        <div class="msg-content">
                            {move || if streaming.get() {
                                view! {
                                    <pre class="streaming-text">
                                        {move || text.get()}
                                        <span class="streaming-cursor"></span>
                                    </pre>
                                }.into_view()
                            } else {
                                view! {
                                    <crate::markdown::MarkdownRenderer
                                        text=text.get()
                                        show_cursor=Signal::derive(|| false)
                                    />
                                }.into_view()
                            }}
                        </div>
                    </div>
                </div>
            }
            .into_view()
        }

        CellContent::Code {
            language,
            source,
            rendered: _,
        } => view! {
            <div class="msg">
                <div class="msg-rail">
                    <div class="msg-avatar assistant">"ai"</div>
                </div>
                <div class="msg-body">
                    <div class="msg-content">
                        <div class="code-block">
                            <div class="code-head">
                                <span>{language.clone()}</span>
                            </div>
                            <pre><code class=format!("language-{}", language)>{source}</code></pre>
                        </div>
                    </div>
                </div>
            </div>
        }
        .into_view(),

        CellContent::Error { message, details } => view! {
            <div class="err-card">
                <div class="err-title">{message}</div>
                {details.map(|d| view! { <div class="err-details">{d}</div> })}
            </div>
        }
        .into_view(),

        CellContent::Loading { message: _ } => view! {
            <div class="msg">
                <div class="msg-rail">
                    <div class="msg-avatar assistant">"ai"</div>
                </div>
                <div class="msg-body">
                    <div class="msg-content">
                        <span class="streaming-cursor"></span>
                    </div>
                </div>
            </div>
        }
        .into_view(),

        _ => view! { <div></div> }.into_view(),
    }
}

fn live_text_response(notebook: ReadSignal<Notebook>, cell_id: CellId) -> Option<(String, bool)> {
    notebook
        .get()
        .cells
        .iter()
        .find(|c| c.id == cell_id)
        .and_then(|c| match &c.content {
            CellContent::TextResponse { text, streaming } => Some((text.clone(), *streaming)),
            _ => None,
        })
}

fn format_timestamp(dt: &chrono::DateTime<chrono::Utc>) -> String {
    use chrono::{Local, TimeZone};
    let local = Local.from_utc_datetime(&dt.naive_utc());
    let days = Local::now().signed_duration_since(local).num_days();
    if days <= 0 {
        local.format("%H:%M").to_string()
    } else if days == 1 {
        format!("Yesterday {}", local.format("%H:%M"))
    } else if days < 7 {
        local.format("%a %H:%M").to_string()
    } else {
        local.format("%b %d, %H:%M").to_string()
    }
}
