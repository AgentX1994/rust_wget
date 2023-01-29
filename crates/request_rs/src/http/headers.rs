use std::collections::HashMap;

use unicase::UniCase;

#[derive(Debug, Default)]
pub struct Headers {
    headers: Vec<(String, String)>,
    index_map: HashMap<UniCase<String>, usize>,
}

impl Headers {
    pub fn get<K>(&self, key: &K) -> Option<&str>
    where
        K: ?Sized,
        K: AsRef<str>,
    {
        self.index_map
            .get(&UniCase::new(key.as_ref().into()))
            .map(|index| &self.headers[*index].1[..])
    }

    pub fn add<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        let key = UniCase::new(key.into());
        if let Some(index) = self.index_map.get(&key) {
            // TODO some headers add values when specified multiple times
            self.headers[*index].1 = value.into();
        } else {
            let index = self.headers.len();
            self.headers.push((key.clone().into(), value.into()));
            if self.index_map.insert(key, index).is_some() {
                panic!("Header exists even though it just didn't!");
            }
        }
    }

    pub fn remove<K>(&mut self, key: &K) -> Option<String>
    where
        K: ?Sized,
        K: AsRef<str>,
    {
        if let Some(index) = self.index_map.remove(&UniCase::new(key.as_ref().into())) {
            let header_value = self.headers.remove(index).1;
            // Need to modify all of the other indices that occured after `index`
            for (_key, i) in self.index_map.iter_mut().filter(|(_k, i)| **i > index) {
                *i -= 1;
            }
            Some(header_value)
        } else {
            None
        }
    }
}

pub struct HeaderIter<'a> {
    headers: &'a Headers,
    index: usize,
}

impl<'a> Iterator for HeaderIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.headers.headers.len() {
            let cur_header = &self.headers.headers[self.index];
            self.index += 1;
            Some((&cur_header.0[..], &cur_header.1[..]))
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a Headers {
    type Item = (&'a str, &'a str);

    type IntoIter = HeaderIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        HeaderIter {
            headers: self,
            index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_make_default() {
        let _ = Headers::default();
    }

    #[test]
    fn can_add_headers() {
        let mut headers = Headers::default();
        headers.add("My Header", "My Value");
        headers.add("My Header 2", "My Value 2");
        assert_eq!(headers.get("My Header"), Some("My Value"));
        assert_eq!(headers.get("My Header 2"), Some("My Value 2"));
    }

    #[test]
    fn can_iter_headers() {
        let mut headers = Headers::default();
        headers.add("My Header", "My Value");
        headers.add("My Header 2", "My Value 2");

        let mut my_iter = (&headers).into_iter();
        assert_eq!(my_iter.next(), Some(("My Header", "My Value")));
        assert_eq!(my_iter.next(), Some(("My Header 2", "My Value 2")));
        assert_eq!(my_iter.next(), None);
    }

    #[test]
    fn can_remove_headers() {
        let mut headers = Headers::default();
        headers.add("My Header", "My Value");
        headers.add("My Header 2", "My Value 2");
        assert_eq!(headers.get("My Header"), Some("My Value"));
        assert_eq!(headers.get("My Header 2"), Some("My Value 2"));
        assert_eq!(headers.remove("My Header"), Some("My Value".to_string()));
        assert_eq!(headers.get("My Header 2"), Some("My Value 2"));
    }
}
