use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod cell;
pub mod renderer;
pub mod parser;

pub use cell::*;
pub use renderer::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Notebook {
    pub cells: Vec<Cell>,
    pub cursor_position: CellId,
    pub active_input: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CellId(pub usize);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cell {
    pub id: CellId,
    pub content: CellContent,
    pub timestamp: DateTime<Utc>,
    pub metadata: CellMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CellMetadata {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub hidden: bool,
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CellContent {
    UserInput {
        text: String,
    },
    TextResponse {
        text: String,
        streaming: bool,
    },
    Code {
        language: String,
        source: String,
        rendered: Option<RenderedContent>,
    },
    Diagram {
        format: DiagramFormat,
        source: String,
        rendered: Option<RenderedContent>,
    },
    Image {
        url: String,
        alt: String,
        dimensions: Option<(u32, u32)>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Chart {
        chart_type: ChartType,
        data: serde_json::Value,
        rendered: Option<RenderedContent>,
    },
    Error {
        message: String,
        details: Option<String>,
    },
    Loading {
        message: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiagramFormat {
    Graphviz,
    PlantUML,
    Mermaid,
    D2,
    Excalidraw,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Scatter,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderedContent {
    pub svg: Option<String>,
    pub html: Option<String>,
    pub error: Option<String>,
}

impl DiagramFormat {
    pub fn from_language(lang: &str) -> Option<Self> {
        match lang.to_lowercase().as_str() {
            "dot" | "graphviz" => Some(DiagramFormat::Graphviz),
            "plantuml" | "puml" => Some(DiagramFormat::PlantUML),
            "mermaid" => Some(DiagramFormat::Mermaid),
            "d2" => Some(DiagramFormat::D2),
            "excalidraw" => Some(DiagramFormat::Excalidraw),
            _ => None,
        }
    }
    
    pub fn file_extension(&self) -> &'static str {
        match self {
            DiagramFormat::Graphviz => "dot",
            DiagramFormat::PlantUML => "puml",
            DiagramFormat::Mermaid => "mmd",
            DiagramFormat::D2 => "d2",
            DiagramFormat::Excalidraw => "excalidraw",
            DiagramFormat::Unknown(_) => "txt",
        }
    }
}

impl fmt::Display for DiagramFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagramFormat::Graphviz => write!(f, "Graphviz"),
            DiagramFormat::PlantUML => write!(f, "PlantUML"),
            DiagramFormat::Mermaid => write!(f, "Mermaid"),
            DiagramFormat::D2 => write!(f, "D2"),
            DiagramFormat::Excalidraw => write!(f, "Excalidraw"),
            DiagramFormat::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl Notebook {
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            cursor_position: CellId(0),
            active_input: String::new(),
        }
    }
    
    pub fn add_cell(&mut self, content: CellContent) -> CellId {
        let id = CellId(self.cells.len());
        let cell = Cell {
            id,
            content,
            timestamp: Utc::now(),
            metadata: CellMetadata::default(),
        };
        self.cells.push(cell);
        id
    }
    
    pub fn get_cell(&self, id: CellId) -> Option<&Cell> {
        self.cells.get(id.0)
    }
    
    pub fn get_cell_mut(&mut self, id: CellId) -> Option<&mut Cell> {
        self.cells.get_mut(id.0)
    }
    
    pub fn update_streaming_response(&mut self, id: CellId, text: &str) {
        if let Some(cell) = self.get_cell_mut(id) {
            if let CellContent::TextResponse { text: content, streaming } = &mut cell.content {
                // Trim leading whitespace on first chunk
                if content.is_empty() && !text.trim_start().is_empty() {
                    content.push_str(text.trim_start());
                } else {
                    content.push_str(text);
                }
                *streaming = true;
            }
        }
    }
    
    pub fn finalize_streaming_response(&mut self, id: CellId) {
        if let Some(cell) = self.get_cell_mut(id) {
            if let CellContent::TextResponse { text: content, streaming } = &mut cell.content {
                // Trim trailing whitespace when finalizing
                *content = content.trim_end().to_string();
                *streaming = false;
            }
            // Trigger diagram detection after streaming completes
            cell.detect_and_render_diagrams();
        }
    }
}

impl Cell {
    fn detect_and_render_diagrams(&mut self) {
        // This will be implemented to detect diagram code blocks
        // and convert them to Diagram cells
    }
}

impl Default for CellMetadata {
    fn default() -> Self {
        Self {
            provider: None,
            model: None,
            hidden: false,
            pinned: false,
        }
    }
}