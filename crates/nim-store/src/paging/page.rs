use crate::Cursor;

#[derive(Debug, Clone)]
pub struct Page {
    pub cursor: Option<Cursor>,
    pub limit: usize,
}
