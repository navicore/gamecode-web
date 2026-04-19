use crate::components::icons::{IconCheck, IconCopy};
use leptos::*;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

enum Segment {
    Html(String),
    Code {
        lang: String,
        source: String,
        highlighted: String,
    },
}

fn render_segments(text: &str) -> Vec<Segment> {
    let parser = Parser::new(text);
    let mut out: Vec<Segment> = Vec::new();
    let mut html_buf = String::new();
    let mut in_code = false;
    let mut code_src = String::new();
    let mut code_lang = String::new();
    let mut heading_level: Option<HeadingLevel> = None;

    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let theme = &theme_set.themes["base16-ocean.dark"];

    let flush_html = |buf: &mut String, out: &mut Vec<Segment>| {
        if !buf.is_empty() {
            out.push(Segment::Html(std::mem::take(buf)));
        }
    };

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_html(&mut html_buf, &mut out);
                in_code = true;
                code_src.clear();
                code_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code = false;
                let syntax = syntax_set
                    .find_syntax_by_token(&code_lang)
                    .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
                let highlighted =
                    highlighted_html_for_string(&code_src, &syntax_set, syntax, theme)
                        .unwrap_or_else(|_| {
                            format!(
                                "<pre><code>{}</code></pre>",
                                html_escape::encode_text(&code_src)
                            )
                        });
                out.push(Segment::Code {
                    lang: if code_lang.is_empty() {
                        "plaintext".into()
                    } else {
                        code_lang.clone()
                    },
                    source: code_src.clone(),
                    highlighted,
                });
                code_src.clear();
                code_lang.clear();
            }
            Event::Text(t) if in_code => code_src.push_str(&t),
            Event::Code(c) => {
                html_buf.push_str(&format!(
                    "<code class=\"inline-code\">{}</code>",
                    html_escape::encode_text(&c)
                ));
            }
            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = Some(level);
                html_buf.push_str(&format!("<h{}>", heading_num(level)));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = heading_level.take() {
                    html_buf.push_str(&format!("</h{}>", heading_num(level)));
                }
            }
            Event::Start(Tag::Paragraph) => html_buf.push_str("<p>"),
            Event::End(TagEnd::Paragraph) => html_buf.push_str("</p>"),
            Event::Start(Tag::Strong) => html_buf.push_str("<strong>"),
            Event::End(TagEnd::Strong) => html_buf.push_str("</strong>"),
            Event::Start(Tag::Emphasis) => html_buf.push_str("<em>"),
            Event::End(TagEnd::Emphasis) => html_buf.push_str("</em>"),
            Event::Start(Tag::List(ordered)) => {
                html_buf.push_str(if ordered.is_some() { "<ol>" } else { "<ul>" });
            }
            Event::End(TagEnd::List(ordered)) => {
                html_buf.push_str(if ordered { "</ol>" } else { "</ul>" });
            }
            Event::Start(Tag::Item) => html_buf.push_str("<li>"),
            Event::End(TagEnd::Item) => html_buf.push_str("</li>"),
            Event::Start(Tag::Link {
                dest_url, title, ..
            }) => {
                html_buf.push_str(&format!(
                    r#"<a href="{}" title="{}" target="_blank" rel="noopener">"#,
                    html_escape::encode_double_quoted_attribute(&dest_url),
                    html_escape::encode_double_quoted_attribute(&title)
                ));
            }
            Event::End(TagEnd::Link) => html_buf.push_str("</a>"),
            Event::Start(Tag::BlockQuote(_)) => html_buf.push_str("<blockquote>"),
            Event::End(TagEnd::BlockQuote(_)) => html_buf.push_str("</blockquote>"),
            Event::HardBreak => html_buf.push_str("<br>"),
            Event::SoftBreak => html_buf.push(' '),
            Event::Rule => html_buf.push_str("<hr>"),
            Event::Text(t) => html_buf.push_str(&html_escape::encode_text(&t)),
            _ => {}
        }
    }
    flush_html(&mut html_buf, &mut out);
    out
}

fn heading_num(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[component]
pub fn MarkdownRenderer(text: String, #[prop(into)] show_cursor: Signal<bool>) -> impl IntoView {
    let segments = render_segments(&text);
    let last_idx = segments.len().saturating_sub(1);

    let rendered = segments
        .into_iter()
        .enumerate()
        .map(|(i, seg)| {
            let is_last = i == last_idx;
            match seg {
                Segment::Html(html) => {
                    let cursor_view = if is_last {
                        view! {
                            <Show when=move || show_cursor.get() fallback=|| view! { <span></span> }>
                                <span class="streaming-cursor"></span>
                            </Show>
                        }
                        .into_view()
                    } else {
                        view! { <span></span> }.into_view()
                    };
                    view! {
                        <div class="md-seg">
                            <div inner_html=html></div>
                            {cursor_view}
                        </div>
                    }
                    .into_view()
                }
                Segment::Code { lang, source, highlighted } => {
                    view! { <CodeBlock lang=lang source=source highlighted=highlighted/> }
                        .into_view()
                }
            }
        })
        .collect_view();

    view! {
        <div class="markdown-content">{rendered}</div>
    }
}

#[component]
fn CodeBlock(lang: String, source: String, highlighted: String) -> impl IntoView {
    let (copied, set_copied) = create_signal(false);
    let src_clone = source.clone();

    let handle_copy = move |_| {
        let text = src_clone.clone();
        if let Some(nav) = web_sys::window().map(|w| w.navigator()) {
            let clipboard = nav.clipboard();
            spawn_local(async move {
                let _ = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&text)).await;
            });
        }
        set_copied.set(true);
        if let Some(window) = web_sys::window() {
            use wasm_bindgen::closure::Closure;
            use wasm_bindgen::JsCast;
            let closure = Closure::once(move || set_copied.set(false));
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                1400,
            );
            closure.forget();
        }
    };

    view! {
        <div class="code-block">
            <div class="code-head">
                <span>{lang}</span>
                <button
                    class="code-copy"
                    class:copied=move || copied.get()
                    on:click=handle_copy
                >
                    {move || if copied.get() {
                        view! { <IconCheck/> <span>"Copied"</span> }.into_view()
                    } else {
                        view! { <IconCopy/> <span>"Copy"</span> }.into_view()
                    }}
                </button>
            </div>
            <div inner_html=highlighted></div>
        </div>
    }
}
