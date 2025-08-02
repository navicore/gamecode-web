use leptos::*;
use pulldown_cmark::{Parser, Event, Tag, TagEnd, CodeBlockKind, HeadingLevel};
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use wasm_bindgen::prelude::*;
use web_sys::window;

#[component]
pub fn MarkdownRenderer(text: String) -> impl IntoView {
    let parser = Parser::new(&text);
    let mut html_output = String::new();
    let mut in_code_block = false;
    let mut code_block_content = String::new();
    let mut code_block_lang = String::new();
    let mut code_block_id = 0;
    let mut current_heading_level = None;
    
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let theme = &theme_set.themes["base16-ocean.dark"];
    
    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_block_content.clear();
                code_block_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                code_block_id += 1;
                
                let syntax = syntax_set.find_syntax_by_token(&code_block_lang)
                    .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
                
                let highlighted = highlighted_html_for_string(
                    &code_block_content,
                    &syntax_set,
                    syntax,
                    theme,
                ).unwrap_or_else(|_| {
                    format!("<pre><code>{}</code></pre>", 
                        html_escape::encode_text(&code_block_content))
                });
                
                html_output.push_str(&format!(
                    r#"<div class="code-block-wrapper">
                        <div class="code-block-header">
                            <span class="code-block-lang">{}</span>
                            <button class="code-copy-btn" data-code-id="code-{}" onclick="copyCode('code-{}')">
                                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                                </svg>
                                <span class="copy-tooltip">Copy code</span>
                            </button>
                        </div>
                        <div class="code-block-content" id="code-{}" data-raw-code="{}">
                            {}
                        </div>
                    </div>"#,
                    if code_block_lang.is_empty() { "plaintext" } else { &code_block_lang },
                    code_block_id,
                    code_block_id,
                    code_block_id,
                    html_escape::encode_double_quoted_attribute(&code_block_content),
                    highlighted
                ));
                
                code_block_content.clear();
            }
            Event::Text(text) if in_code_block => {
                code_block_content.push_str(&text);
            }
            Event::Code(code) => {
                html_output.push_str(&format!(
                    "<code class=\"inline-code\">{}</code>",
                    html_escape::encode_text(&code)
                ));
            }
            Event::Start(Tag::Heading { level, .. }) => {
                current_heading_level = Some(level);
                let level_num = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                html_output.push_str(&format!("<h{}>", level_num));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = current_heading_level {
                    let level_num = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    html_output.push_str(&format!("</h{}>", level_num));
                    current_heading_level = None;
                }
            }
            Event::Start(Tag::Paragraph) => {
                html_output.push_str("<p>");
            }
            Event::End(TagEnd::Paragraph) => {
                html_output.push_str("</p>");
            }
            Event::Start(Tag::Strong) => {
                html_output.push_str("<strong>");
            }
            Event::End(TagEnd::Strong) => {
                html_output.push_str("</strong>");
            }
            Event::Start(Tag::Emphasis) => {
                html_output.push_str("<em>");
            }
            Event::End(TagEnd::Emphasis) => {
                html_output.push_str("</em>");
            }
            Event::Start(Tag::List(ordered)) => {
                if ordered.is_some() {
                    html_output.push_str("<ol>");
                } else {
                    html_output.push_str("<ul>");
                }
            }
            Event::End(TagEnd::List(ordered)) => {
                if ordered {
                    html_output.push_str("</ol>");
                } else {
                    html_output.push_str("</ul>");
                }
            }
            Event::Start(Tag::Item) => {
                html_output.push_str("<li>");
            }
            Event::End(TagEnd::Item) => {
                html_output.push_str("</li>");
            }
            Event::Start(Tag::Link { dest_url, title, .. }) => {
                html_output.push_str(&format!(
                    r#"<a href="{}" title="{}" target="_blank" rel="noopener">"#,
                    html_escape::encode_double_quoted_attribute(&dest_url),
                    html_escape::encode_double_quoted_attribute(&title)
                ));
            }
            Event::End(TagEnd::Link) => {
                html_output.push_str("</a>");
            }
            Event::Start(Tag::BlockQuote(_)) => {
                html_output.push_str("<blockquote>");
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                html_output.push_str("</blockquote>");
            }
            Event::HardBreak => {
                html_output.push_str("<br>");
            }
            Event::SoftBreak => {
                html_output.push_str(" ");
            }
            Event::Rule => {
                html_output.push_str("<hr>");
            }
            Event::Text(text) => {
                html_output.push_str(&html_escape::encode_text(&text));
            }
            _ => {}
        }
    }
    
    view! {
        <div class="markdown-content">
            <div inner_html=html_output></div>
            <script>
                "function copyCode(codeId) {
                    const codeElement = document.getElementById(codeId);
                    const rawCode = codeElement.getAttribute('data-raw-code');
                    const button = document.querySelector(`button[data-code-id='${codeId}']`);
                    const tooltip = button.querySelector('.copy-tooltip');
                    
                    navigator.clipboard.writeText(rawCode).then(() => {
                        tooltip.textContent = 'Copied!';
                        button.classList.add('copied');
                        setTimeout(() => {
                            tooltip.textContent = 'Copy code';
                            button.classList.remove('copied');
                        }, 2000);
                    }).catch(err => {
                        console.error('Failed to copy:', err);
                        tooltip.textContent = 'Failed to copy';
                        setTimeout(() => {
                            tooltip.textContent = 'Copy code';
                        }, 2000);
                    });
                }"
            </script>
        </div>
    }
}

pub fn copy_to_clipboard(text: &str) -> Result<(), JsValue> {
    let window = window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let navigator = window.navigator();
    let clipboard = navigator.clipboard();
    
    let promise = clipboard.write_text(text);
    wasm_bindgen_futures::spawn_local(async move {
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    });
    
    Ok(())
}