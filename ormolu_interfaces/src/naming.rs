pub trait ToSingular {
    fn to_singular(&self) -> String;
}

pub trait ToPlural {
    fn to_plural(&self) -> String;
}

fn naive_pluralize(s: &str) -> String {
    if s.ends_with('s') {
        s.to_string()
    } else {
        format!("{s}s")
    }
}

fn naive_singularize(s: &str) -> String {
    if s.ends_with("ies") {
        format!("{}y", &s[..s.len() - 3])
    } else if s.ends_with('s') && !s.ends_with("ss") {
        s[..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

impl ToSingular for &str {
    fn to_singular(&self) -> String {
        naive_singularize(self)
    }
}

impl ToPlural for &str {
    fn to_plural(&self) -> String {
        naive_pluralize(self)
    }
}

impl ToSingular for String {
    fn to_singular(&self) -> String {
        naive_singularize(self)
    }
}

impl ToPlural for String {
    fn to_plural(&self) -> String {
        naive_pluralize(self)
    }
}
