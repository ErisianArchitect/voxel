use toml::Table;

struct PathSplit<'a> {
    raw: &'a str,
    index: usize,
    split_start: usize,
}

impl<'a> PathSplit<'a> {
    #[must_use]
    #[inline(always)]
    pub const fn new(path: &'a str) -> Self {
        Self {
            raw: path,
            index: 0,
            split_start: 0,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnclosedQuote(usize);

impl<'a> Iterator for PathSplit<'a> {
    type Item = Result<&'a str, UnclosedQuote>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut i = self.index;
        let raw = self.raw;
        let rb = raw.as_bytes();
        if i >= raw.len() {
            return None;
        }
        while i < rb.len() {
            match rb[i] {
                b'.' => {
                    let part = &raw[self.split_start..i];
                    self.index = i + 1;
                    self.split_start = i;
                    return Some(Ok(part));
                }
                b'\\' => {
                    // this essentially means `if `\` is followed by `'`, `\`, or `.`, add 2, otherwise add 1.
                    // I wrote it like this because I thought it would be fun. I don't care if it's hard to read.
                    i += 1 + ((i + 1 < rb.len() && [b'\'', b'.', b'\\'].contains(&rb[i + 1])) as usize);
                }
                b'\'' => {
                    let quote_start = i;
                    'success: {
                        while i < rb.len() {
                            match rb[i] {
                                b'\'' => break 'success,
                                b'\\' => {
                                    // this essentially means `if `\` is followed by `'`, `\`, or `.`, add 2, otherwise add 1.
                                    // I wrote it like this because I thought it would be fun. I don't care if it's hard to read.
                                    i += 1 + ((i + 1 < rb.len() && [b'\'', b'\\'].contains(&rb[i + 1])) as usize);
                                }
                                _ => i += 1,
                            }
                        }
                        self.index = i;
                        return Some(Err(UnclosedQuote(quote_start)));
                    }
                }
                _ => i += 1,
            }
        }
        None
    }
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct Env {
    table: Table,
}

impl Env {
    pub fn get(&self, path: &str) -> Option<Result<&toml::Value, UnclosedQuote>> {
        let mut parts = PathSplit::new(path);
        let first = match parts.next() {
            Some(Ok(first)) => first,
            Some(Err(err)) => return Some(Err(err)),
            None => return None,
        };
        let Some(mut lookup) = self.table.get(first) else {
            return None;
        };
        loop {
            match parts.next() {
                Some(Ok(part)) => {
                    let Some(found) = lookup.get(part) else {
                        return None;
                    };
                    lookup = found;
                }
                Some(Err(err)) => return Some(Err(err)),
                None => break,
            }
        }
        Some(Ok(lookup))
    }
}
