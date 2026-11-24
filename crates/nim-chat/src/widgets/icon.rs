#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Icon {
    Plus,
    Minus,
    Check,
    X,
    Pencil,
    Trash,
    Copy,
    Save,
    Send,
    Search,
    Download,
    Upload,
    RefreshCw,
    ArrowLeft,
    ArrowRight,
    ChevronUp,
    ChevronDown,
    ExternalLink,
    Ellipsis,
    Settings,
    Info,
    AlertTriangle,
}

#[non_exhaustive]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IconPosition {
    #[default]
    Before,
    After,
}
