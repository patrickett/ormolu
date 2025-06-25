#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VarChar<const LENGTH: usize> {
    inner: [u8; LENGTH],
}

impl<const LENGTH: usize> VarChar<LENGTH> {
    /// Try to create a VarChar from a `&str`. Will error if too long.
    pub fn new(s: &str) -> Result<Self, &'static str> {
        let bytes = s.as_bytes();
        if bytes.len() > LENGTH {
            return Err("String too long");
        }

        let mut inner = [0u8; LENGTH];
        inner[..bytes.len()].copy_from_slice(bytes);
        Ok(Self { inner })
    }

    /// Get the inner string (trims trailing zeros).
    pub fn as_str(&self) -> &str {
        let end = self.inner.iter().position(|&c| c == 0).unwrap_or(LENGTH);
        str::from_utf8(&self.inner[..end]).unwrap()
    }
}

// impl<const LENGTH: usize> std::fmt::Debug for VarChar<LENGTH> {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         f.debug_tuple("VarChar").field(&self.as_str()).finish()
//     }
// }

impl<const LENGTH: usize> std::fmt::Display for VarChar<LENGTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const LENGTH: usize> From<VarChar<LENGTH>> for String {
    fn from(v: VarChar<LENGTH>) -> Self {
        v.as_str().to_string()
    }
}

impl<const LENGTH: usize> TryFrom<&str> for VarChar<LENGTH> {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<const LENGTH: usize> TryFrom<String> for VarChar<LENGTH> {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}
