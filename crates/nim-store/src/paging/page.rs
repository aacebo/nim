use crate::Cursor;

#[derive(Debug, Copy, Clone)]
pub struct Page {
    pub cursor: Option<Cursor>,
    pub limit: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PageResponse<T> {
    pub size: usize,
    pub next: Option<Cursor>,
    pub data: Vec<T>,
}

impl<T> PageResponse<T> {
    pub fn into<V>(self) -> PageResponse<V>
    where
        T: Into<V>,
    {
        PageResponse {
            size: self.size,
            next: self.next,
            data: self.data.into_iter().map(Into::into).collect(),
        }
    }

    pub fn try_into<V>(self) -> Result<PageResponse<V>, T::Error>
    where
        T: TryInto<V>,
    {
        let mut data = Vec::with_capacity(self.data.len());

        for item in self.data {
            data.push(TryInto::try_into(item)?);
        }

        Ok(PageResponse {
            size: self.size,
            next: self.next,
            data,
        })
    }
}

impl<T> std::ops::Deref for PageResponse<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> IntoIterator for &'a PageResponse<T> {
    type IntoIter = std::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}
