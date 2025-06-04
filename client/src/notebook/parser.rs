use regex::Regex;
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub language: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone)]
pub struct DiagramHint {
    pub text: String,
    pub suggested_format: crate::notebook::DiagramFormat,
}

static CODE_BLOCK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"```(\w+)?\n([\s\S]*?)```").unwrap()
});

static DIAGRAM_HINT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(flow|sequence|diagram|graph|chart):\s*(.+)").unwrap()
});

pub fn extract_code_blocks(text: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let mut line_count = 0;
    
    for capture in CODE_BLOCK_REGEX.captures_iter(text) {
        let language = capture.get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "text".to_string());
        
        let content = capture.get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        
        // Count lines to determine position
        let full_match = capture.get(0).unwrap().as_str();
        let start_line = text[..capture.get(0).unwrap().start()]
            .lines()
            .count();
        let block_lines = full_match.lines().count();
        
        blocks.push(CodeBlock {
            language,
            content,
            start_line,
            end_line: start_line + block_lines,
        });
    }
    
    blocks
}

pub fn detect_diagram_hints(text: &str) -> Vec<DiagramHint> {
    let mut hints = Vec::new();
    
    // Check for explicit diagram descriptions
    for capture in DIAGRAM_HINT_REGEX.captures_iter(text) {
        if let Some(hint_text) = capture.get(2) {
            let format = guess_diagram_format(hint_text.as_str());
            hints.push(DiagramHint {
                text: hint_text.as_str().to_string(),
                suggested_format: format,
            });
        }
    }
    
    // Check for arrow notations that suggest diagrams
    if text.contains("->") || text.contains("=>") || text.contains("-->") {
        if let Some(format) = analyze_arrow_notation(text) {
            hints.push(DiagramHint {
                text: text.to_string(),
                suggested_format: format,
            });
        }
    }
    
    hints
}

fn guess_diagram_format(text: &str) -> crate::notebook::DiagramFormat {
    let lower = text.to_lowercase();
    
    if lower.contains("sequence") || lower.contains("actor") {
        crate::notebook::DiagramFormat::PlantUML
    } else if lower.contains("flow") || lower.contains("graph") {
        crate::notebook::DiagramFormat::Mermaid
    } else if lower.contains("->") && lower.contains("digraph") {
        crate::notebook::DiagramFormat::Graphviz
    } else {
        crate::notebook::DiagramFormat::Mermaid // Default
    }
}

fn analyze_arrow_notation(text: &str) -> Option<crate::notebook::DiagramFormat> {
    // Simple heuristic: if it looks like a graph notation
    if text.contains("digraph") || text.contains("subgraph") {
        Some(crate::notebook::DiagramFormat::Graphviz)
    } else if text.contains("@startuml") || text.contains("@enduml") {
        Some(crate::notebook::DiagramFormat::PlantUML)
    } else if text.lines().any(|line| line.trim().starts_with("graph ") || line.trim().starts_with("flowchart ")) {
        Some(crate::notebook::DiagramFormat::Mermaid)
    } else {
        None
    }
}

pub fn extract_structured_data(text: &str) -> Option<serde_json::Value> {
    // Try to parse as JSON
    if let Ok(value) = serde_json::from_str(text) {
        return Some(value);
    }
    
    // Try to extract JSON from markdown code block
    let blocks = extract_code_blocks(text);
    for block in blocks {
        if block.language == "json" {
            if let Ok(value) = serde_json::from_str(&block.content) {
                return Some(value);
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_code_blocks() {
        let text = r#"
Here's some code:
```rust
fn main() {
    println!("Hello");
}
```

And a diagram:
```dot
digraph G {
    A -> B;
}
```
"#;
        
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].language, "rust");
        assert_eq!(blocks[1].language, "dot");
    }
    
    #[test]
    fn test_detect_diagram_hints() {
        let text = "Here's the flow: User -> Server -> Database -> Response";
        let hints = detect_diagram_hints(text);
        assert!(!hints.is_empty());
    }
}