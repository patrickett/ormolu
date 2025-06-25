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
    if let Some(stripped) = s.strip_suffix("ies") {
        format!("{stripped}y")
    } else if let Some(stripped) = s.strip_suffix('s') {
        if !stripped.ends_with('s') {
            stripped.to_string()
        } else {
            s.to_string()
        }
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

#[cfg(test)]
mod naming_tests {
    use super::*;

    #[test]
    fn test_to_singular() {
        // Test cases for singularization
        assert_eq!("cat".to_singular(), "cat");
        assert_eq!("dogs".to_singular(), "dog");
        assert_eq!("men".to_singular(), "man");
        assert_eq!("women".to_singular(), "woman");
        assert_eq!("studies".to_singular(), "study");

        // Ensure it works for `String` as well
        let s = String::from("boxes");
        assert_eq!(s.to_singular(), "box");

        let t = String::from("foxes");
        assert_eq!(t.to_singular(), "fox");
    }

    #[test]
    fn test_to_plural() {
        // Test cases for pluralization
        assert_eq!("cat".to_plural(), "cats");
        assert_eq!("dog".to_plural(), "dogs");
        assert_eq!("man".to_plural(), "men");
        assert_eq!("woman".to_plural(), "women");

        // Ensure it works for `String` as well
        let s = String::from("cat");
        assert_eq!(s.to_plural(), "cats");

        let t = String::from("dog");
        assert_eq!(t.to_plural(), "dogs");
    }
}
