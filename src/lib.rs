mod ruuls;

pub use ruuls::{Checklist, ChecklistResult, Status};

use ruuls::Constraint;

pub struct Rules {}

impl Rules {
    pub fn and(checklists: Vec<Checklist>) -> Checklist {
        Checklist::And(checklists)
    }

    pub fn string_equals(description: &str, field: &str, val: &str) -> Checklist {
        Checklist::Rule(description.into(),
                        field.into(),
                        Constraint::StringEquals(val.into()))
    }

    pub fn int_equals(description: &str, field: &str, val: i32) -> Checklist {
        Checklist::Rule(description.into(), field.into(), Constraint::IntEquals(val))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use super::Rules;
    #[test]
    fn building() {
        let root = Rules::and(vec![Rules::int_equals("foo = 1", "foo", 1),
                                   Rules::string_equals("bar = 'bar'", "bar", "bar")]);

        let mut map = BTreeMap::new();
        map.insert("foo".into(), "1".into());
        map.insert("bar".into(), "bar".into());
        let res = root.check(&map);
        println!("{:?}", res);
    }
}
