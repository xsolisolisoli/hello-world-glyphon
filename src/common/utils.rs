
pub trait IsNullOrEmpty {
    fn is_null_or_empty(&self) -> bool;
}

impl IsNullOrEmpty for Option<&str> {
    fn is_null_or_empty(&self) -> bool {
        self.map_or(true, |s| s.is_empty())
    }
}

impl IsNullOrEmpty for Option<String> {
    fn is_null_or_empty(&self) -> bool {
        self.as_ref().map_or(true, |s| s.is_empty())
    }
}

