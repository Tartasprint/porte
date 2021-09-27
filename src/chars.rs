use std::mem::size_of_val;

pub struct Chars {
    inner: Box<dyn Iterator<Item = u8>>,
    last_read: u32,
}

impl Chars {
    pub fn new(inner: Box<dyn Iterator<Item = u8>>) -> Self {
        Self {
            inner,
            last_read: 0u32,
        }
    }
}

impl Iterator for Chars {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        for _ in 1..=4 {
            let n = self.inner.next()?;
            self.last_read <<= size_of_val(&n);
            self.last_read |= n as u32;
            match std::char::from_u32(self.last_read) {
                Some(c) => return Some(c),
                None => continue,
            }
        }
        return None;
    }
}

impl From<&str> for Chars {
    fn from(x: &str) -> Self {
        let b: Vec<u8> = x.bytes().collect();
        Self::new(Box::new(b.into_iter()))
    }
}

impl From<String> for Chars {
    fn from(x: String) -> Self {
        let b: Vec<u8> = x.bytes().collect();
        Self::new(Box::new(b.into_iter()))
    }
}