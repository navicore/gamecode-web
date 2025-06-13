use leptos::*;
use crate::notebook::{Cell, CellContent, DiagramFormat};

#[component]
pub fn CellView(cell: Cell) -> impl IntoView {
    match cell.content {
        CellContent::UserInput { text } => {
            view! {
                <div class="cell cell-input">
                    <div class="cell-prompt">">"</div>
                    <div class="cell-content">{text}</div>
                    <div class="cell-timestamp">{format_timestamp(&cell.timestamp)}</div>
                </div>
            }
        }
        
        CellContent::TextResponse { text, streaming } => {
            view! {
                <div class="cell cell-response">
                    <div class="cell-content">
                        <Markdown text=text/>
                        {if streaming {
                            view! { <span class="streaming-indicator">"●"</span> }.into_view()
                        } else {
                            view! { <span></span> }.into_view()
                        }}
                    </div>
                    {cell.metadata.provider.map(|p| view! {
                        <div class="cell-provider">{p}</div>
                    })}
                </div>
            }
        }
        
        CellContent::Code { language, source, rendered } => {
            view! {
                <div class="cell cell-code">
                    <div class="cell-header">
                        <span class="language-tag">{language.clone()}</span>
                        <button class="copy-btn" on:click={
                            let source = source.clone();
                            move |_| copy_to_clipboard(&source)
                        }>
                            "Copy"
                        </button>
                    </div>
                    <pre class="code-block">
                        <code class=format!("language-{}", language)>{source.clone()}</code>
                    </pre>
                    {rendered.map(|r| view! {
                        <div class="rendered-content">
                            {if let Some(svg) = r.svg {
                                view! { <div inner_html=svg></div> }.into_view()
                            } else if let Some(html) = r.html {
                                view! { <div inner_html=html></div> }.into_view()
                            } else if let Some(error) = r.error {
                                view! { <div class="render-error">{error}</div> }.into_view()
                            } else {
                                view! { <div></div> }.into_view()
                            }}
                        </div>
                    })}
                </div>
            }
        }
        
        CellContent::Diagram { format, source, rendered } => {
            let (show_source, set_show_source) = create_signal(false);
            
            view! {
                <div class="cell cell-diagram">
                    <div class="cell-header">
                        <span class="diagram-type">{format.to_string()}</span>
                        <div class="cell-actions">
                            <button on:click=move |_| set_show_source.update(|s| *s = !*s)>
                                {move || if show_source.get() { "Hide Source" } else { "Show Source" }}
                            </button>
                            <button on:click={
                                let source = source.clone();
                                move |_| copy_to_clipboard(&source)
                            }>
                                "Copy"
                            </button>
                        </div>
                    </div>
                    
                    {move || if show_source.get() {
                        view! {
                            <pre class="diagram-source">
                                <code>{source.clone()}</code>
                            </pre>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }}
                    
                    <div class="diagram-container">
                        {if let Some(r) = rendered {
                            if let Some(svg) = r.svg {
                                view! { <div class="diagram-svg" inner_html=svg></div> }.into_view()
                            } else if let Some(error) = r.error {
                                view! { <div class="render-error">{error}</div> }.into_view()
                            } else {
                                view! { <div class="render-pending">"Rendering..."</div> }.into_view()
                            }
                        } else {
                            view! { <div class="render-pending">"Click to render"</div> }.into_view()
                        }}
                    </div>
                </div>
            }
        }
        
        CellContent::Image { url, alt, dimensions } => {
            view! {
                <div class="cell cell-image">
                    <img 
                        src=url 
                        alt=alt
                        class="cell-image-content"
                        style=dimensions.map(|(w, h)| format!("max-width: {}px; max-height: {}px", w, h))
                    />
                </div>
            }
        }
        
        CellContent::Table { headers, rows } => {
            view! {
                <div class="cell cell-table">
                    <table class="data-table">
                        <thead>
                            <tr>
                                {headers.into_iter().map(|h| view! { <th>{h}</th> }).collect_view()}
                            </tr>
                        </thead>
                        <tbody>
                            {rows.into_iter().map(|row| {
                                view! {
                                    <tr>
                                        {row.into_iter().map(|cell| view! { <td>{cell}</td> }).collect_view()}
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            }
        }
        
        CellContent::Error { message, details } => {
            view! {
                <div class="cell cell-error">
                    <div class="error-message">{message}</div>
                    {details.map(|d| view! { <div class="error-details">{d}</div> })}
                </div>
            }
        }
        
        CellContent::Loading { message } => {
            view! {
                <div class="cell cell-loading">
                    <div class="loading-spinner"></div>
                    {message.map(|m| view! { <div class="loading-message">{m}</div> })}
                </div>
            }
        }
        
        _ => view! { <div class="cell">"Unsupported cell type"</div> }
    }
}

#[component]
fn Markdown(text: String) -> impl IntoView {
    // Simple markdown rendering - in production, use a proper markdown parser
    let html = text
        .replace("**", "<strong>")
        .replace("*", "<em>")
        .replace("\n\n", "</p><p>")
        .replace("\n", "<br>");
    
    view! {
        <div class="markdown-content" inner_html=format!("<p>{}</p>", html)></div>
    }
}

fn format_timestamp(dt: &chrono::DateTime<chrono::Utc>) -> String {
    use chrono::{Local, TimeZone};
    
    // Convert UTC to local time
    let local_dt = Local.from_utc_datetime(&dt.naive_utc());
    
    // Format based on how recent the timestamp is
    let now = Local::now();
    let duration = now.signed_duration_since(local_dt);
    
    if duration.num_days() == 0 {
        // Today - just show time
        local_dt.format("%H:%M:%S").to_string()
    } else if duration.num_days() == 1 {
        // Yesterday
        local_dt.format("Yesterday %H:%M").to_string()
    } else if duration.num_days() < 7 {
        // This week - show day name
        local_dt.format("%A %H:%M").to_string()
    } else {
        // Older - show full date
        local_dt.format("%b %d, %Y %H:%M").to_string()
    }
}

fn copy_to_clipboard(text: &str) {
    if let Some(window) = web_sys::window() {
        let navigator = window.navigator();
        let clipboard = navigator.clipboard();
        let text = text.to_string();
        spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(
                clipboard.write_text(&text)
            ).await;
        });
    }
}