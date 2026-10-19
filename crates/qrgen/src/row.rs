use std::borrow::Cow;

use textplate::{TemplateString, Vars};

/// Variables for one data row:
///
/// - `{{1}}`, `{{2}}`, ... — columns (1-based)
/// - `{{name}}` — columns by header name, when headers are given
/// - `{{ROW}}` — the row number (1-based)
/// - `{{data}}` — the QR data, once set with [`Row::with_data`]
#[derive(Clone, Debug)]
pub struct Row<'a> {
    pub index: usize,
    pub columns: &'a [String],
    pub headers: Option<&'a [String]>,
    pub data: Option<String>,
}

impl<'a> Row<'a> {
    /// `index` is 0-based; `{{ROW}}` is `index + 1`.
    pub fn new(index: usize, columns: &'a [String]) -> Self {
        Self {
            index,
            columns,
            headers: None,
            data: None,
        }
    }

    pub fn with_headers(mut self, headers: &'a [String]) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Sets `{{data}}` by filling `template` with this row's variables.
    pub fn with_data(mut self, template: &TemplateString) -> Result<Self, String> {
        let data = template.render(&self)?.into_owned();
        self.data = Some(data);
        Ok(self)
    }
}

impl Vars for Row<'_> {
    fn get(&self, name: &str) -> Option<Cow<'_, str>> {
        if name == "ROW" {
            return Some(Cow::Owned((self.index + 1).to_string()));
        }
        if name == "data" {
            if let Some(data) = &self.data {
                return Some(Cow::Borrowed(data));
            }
        }
        if let Ok(n) = name.parse::<usize>() {
            return n
                .checked_sub(1)
                .and_then(|i| self.columns.get(i))
                .map(|s| Cow::Borrowed(s.as_str()));
        }
        let i = self.headers?.iter().position(|h| h == name)?;
        self.columns.get(i).map(|s| Cow::Borrowed(s.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_variables() {
        let cols = vec!["Ann".to_string(), "ann@x.io".to_string()];
        let headers = vec!["name".to_string(), "email".to_string()];
        let row = Row::new(4, &cols).with_headers(&headers);
        assert_eq!(row.get("ROW").unwrap(), "5");
        assert_eq!(row.get("1").unwrap(), "Ann");
        assert_eq!(row.get("email").unwrap(), "ann@x.io");
        assert!(row.get("0").is_none() && row.get("3").is_none() && row.get("nope").is_none());
        let row = row
            .with_data(&TemplateString::parse("mailto:{{email}}"))
            .unwrap();
        assert_eq!(row.get("data").unwrap(), "mailto:ann@x.io");
        assert_eq!(
            Row::new(0, &cols)
                .with_data(&TemplateString::parse("{{9}}"))
                .unwrap_err(),
            "9"
        );
    }
}
