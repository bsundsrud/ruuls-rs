//! Simple rules engine that represents requirements as a tree, with each node having one or more requirements in order to be "Met".
//!
//! A tree of rules is constructed, and then the [`.check()`][1] method is called.
//! `map` is a `field: value` mapping of facts that will be given to each node in the tree for testing.
//!
//! Status output can be either `Met`, `NotMet`, or `Unknown` if the tested field is not present in the map.
//!
//! To construct a tree, see the following methods.
//!
//! ## Example
//!
//! ```rust
//! use std::collections::BTreeMap;
//! extern crate ruuls;
//!
//! let tree = ruuls::and(vec![
//!     ruuls::string_equals("Name is John Doe", "name", "John Doe"),
//!     ruuls::or(vec![
//!         ruuls::int_equals("Favorite number is 5", "fav_number", 5),
//!         ruuls::int_range("Thinking of a number between 5 and 10", "thinking_of", 5, 10)
//!     ])
//! ]);
//! let mut facts = BTreeMap::new();
//! facts.insert("name".into(), "John Doe".into());
//! facts.insert("fav_number".into(), "5".into());
//! let result = tree.check(&facts);
//! println!("{:?}", result);
//! assert!(result.status == ruuls::Status::Met);
//! // result = RuleResult { name: "And", status: Met, children: [RuleResult { name: "Name is John Doe", status: Met, children: [] }, RuleResult { name: "Or", status: Met, children: [RuleResult { name: "Favorite number is 5", status: Met, children: [] }, RuleResult { name: "Thinking of a number between 5 and 10", status: Unknown, children: [] }] }] }
//! ```
//!
//! This creates a tree like the following:
//!
//! ```text
//!                              +---------+
//!                              |   AND   |
//!                              +---------+
//!           _____________________/\_______________
//!          |                                      |
//!          V                                      V
//! +-------------------+                       +--------+
//! | Name is John Doe  |                       |   OR   |
//! +-------------------+                       +--------+
//! | field: "name"     |             ______________/\___________
//! | value: "John Doe" |            |                           |
//! +-------------------+            V                           V
//!                       +----------------------+  +-------------------------+
//!                       | Favorite number is 5 |  | Number between 5 and 10 |
//!                       +----------------------+  +-------------------------+
//!                       | field: "fav_number"  |  | field: "thinking_of"    |
//!                       | value: 5             |  | start: 5                |
//!                       +----------------------+  | end: 10                 |
//!                                                 +-------------------------+
//! ```
//!
//! [1]: enum.Rule.html#method.check


#![feature(structural_match, rustc_attrs, proc_macro)]
#[cfg(feature = "serde")]
extern crate serde;
#[cfg(feature = "serde")]
#[macro_use] extern crate serde_derive;

mod ruuls;

pub use ruuls::{Constraint, Rule, RuleResult, Status};

/// Creates a `Rule` where all child `Rule`s must be `Met`
/// 
/// * If any are `NotMet`, the result will be `NotMet`
/// * If the results contain only `Met` and `Unknown`, the result will be `Unknown`
/// * Only results in `Met` if all children are `Met`
pub fn and(rules: Vec<Rule>) -> Rule {
    Rule::And(rules)
}

/// Creates a `Rule` where any child `Rule` must be `Met`
/// 
/// * If any are `Met`, the result will be `Met`
/// * If the results contain only `NotMet` and `Unknown`, the result will be `Unknown`
/// * Only results in `NotMet` if all children are `NotMet`
pub fn or(rules: Vec<Rule>) -> Rule {
    Rule::Or(rules)
}

/// Creates a `Rule` where `n` child `Rule`s must be `Met`
/// 
/// * If `>= n` are `Met`, the result will be `Met`
/// * If `>= children.len() - n + 1` are `NotMet`, the result will be `NotMet` (No combination of `Met` + `Unknown` can be >= `n`)
/// * If neither of the above are met, the result is `Unknown`
pub fn n_of(n: usize, rules: Vec<Rule>) -> Rule {
    Rule::NumberOf(n, rules)
}

/// Creates a rule for string comparison
pub fn string_equals(description: &str, field: &str, val: &str) -> Rule {
    Rule::Rule(description.into(),
               field.into(),
               Constraint::StringEquals(val.into()))
}

/// Creates a rule for int comparison.  
///
///If the checked value is not convertible to an integer, the result is `NotMet`
pub fn int_equals(description: &str, field: &str, val: i32) -> Rule {
    Rule::Rule(description.into(), field.into(), Constraint::IntEquals(val))
}

/// Creates a rule for int range comparison with the interval `[start, end]`.  
///
/// If the checked value is not convertible to an integer, the result is `NotMet`
pub fn int_range(description: &str, field: &str, start: i32, end: i32) -> Rule {
    Rule::Rule(description.into(),
               field.into(),
               Constraint::IntRange(start, end))
}

/// Creates a rule for boolean comparison.  
///
/// Only input values of `"true"` (case-insensitive) are considered `true`, all others are considered `false`
pub fn boolean(description: &str, field: &str, val: bool) -> Rule {
    Rule::Rule(description.into(), field.into(), Constraint::Boolean(val))
}


#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use super::{and, or, n_of, string_equals, int_equals, int_range, boolean, Status};

    fn get_test_data() -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        map.insert("foo".into(), "1".into());
        map.insert("bar".into(), "bar".into());
        map.insert("baz".into(), "true".into());
        map
    }

    #[test]
    fn and_rules() {
        let map = get_test_data();
        // Met & Met == Met
        let mut root = and(vec![int_equals("foo = 1", "foo", 1),
                                string_equals("bar = 'bar'", "bar", "bar")]);
        let mut res = root.check(&map);

        assert!(res.status == Status::Met);

        // Met & NotMet == NotMet
        root = and(vec![int_equals("foo = 2", "foo", 2),
                        string_equals("bar = 'bar'", "bar", "bar")]);
        res = root.check(&map);

        assert!(res.status == Status::NotMet);

        // Met & Unknown == Unknown
        root = and(vec![int_equals("quux = 2", "quux", 2),
                        string_equals("bar = 'bar'", "bar", "bar")]);
        res = root.check(&map);

        assert!(res.status == Status::Unknown);

        // NotMet & Unknown == NotMet
        root = and(vec![int_equals("quux = 2", "quux", 2),
                        string_equals("bar = 'baz'", "bar", "baz")]);
        res = root.check(&map);

        assert!(res.status == Status::NotMet);

        // Unknown & Unknown == Unknown
        root = and(vec![int_equals("quux = 2", "quux", 2),
                        string_equals("fizz = 'bar'", "fizz", "bar")]);
        res = root.check(&map);

        assert!(res.status == Status::Unknown);
    }

    #[test]
    fn or_rules() {
        let map = get_test_data();
        // Met | Met == Met
        let mut root = or(vec![int_equals("foo = 1", "foo", 1),
                               string_equals("bar = 'bar'", "bar", "bar")]);
        let mut res = root.check(&map);

        assert!(res.status == Status::Met);

        // Met | NotMet == Met
        root = or(vec![int_equals("foo = 2", "foo", 2),
                       string_equals("bar = 'bar'", "bar", "bar")]);
        res = root.check(&map);

        assert!(res.status == Status::Met);

        // Met | Unknown == Met
        root = or(vec![int_equals("quux = 2", "quux", 2),
                       string_equals("bar = 'bar'", "bar", "bar")]);
        res = root.check(&map);

        assert!(res.status == Status::Met);

        // NotMet | Unknown == Unknown
        root = or(vec![int_equals("quux = 2", "quux", 2),
                       string_equals("bar = 'baz'", "bar", "baz")]);
        res = root.check(&map);

        assert!(res.status == Status::Unknown);

        // Unknown | Unknown == Unknown
        root = or(vec![int_equals("quux = 2", "quux", 2),
                       string_equals("fizz = 'bar'", "fizz", "bar")]);
        res = root.check(&map);

        assert!(res.status == Status::Unknown);
    }

    #[test]
    fn n_of_rules() {
        let map = get_test_data();
        // 2 Met, 1 NotMet == Met
        let mut root = n_of(2,
                            vec![int_equals("foo = 1", "foo", 1),
                                 string_equals("bar = 'bar'", "bar", "bar"),
                                 boolean("baz is false", "baz", false)]);
        let mut res = root.check(&map);

        assert!(res.status == Status::Met);

        // 1 Met, 1 NotMet, 1 Unknown == Unknown
        root = n_of(2,
                    vec![int_equals("foo = 1", "foo", 1),
                         string_equals("quux = 'bar'", "quux", "bar"),
                         boolean("baz is false", "baz", false)]);
        res = root.check(&map);

        assert!(res.status == Status::Unknown);

        // 2 NotMet, _ == NotMet
        root = n_of(2,
                    vec![int_equals("quux = 2", "quux", 2),
                         string_equals("bar = 'baz'", "bar", "baz"),
                         boolean("baz is false", "baz", false)]);
        res = root.check(&map);

        assert!(res.status == Status::NotMet);

    }

    #[test]
    fn string_equals_rule() {
        let map = get_test_data();
        let mut rule = string_equals("bar = 'bar'", "bar", "bar");
        let mut res = rule.check(&map);
        assert!(res.status == Status::Met);

        rule = string_equals("bar = 'baz'", "bar", "baz");
        res = rule.check(&map);
        assert!(res.status == Status::NotMet);
    }

    #[test]
    fn int_equals_rule() {
        let map = get_test_data();
        let mut rule = int_equals("foo = 1", "foo", 1);
        let mut res = rule.check(&map);
        assert!(res.status == Status::Met);

        rule = int_equals("foo = 2", "foo", 2);
        res = rule.check(&map);
        assert!(res.status == Status::NotMet);

        // Values not convertible to int should be NotMet
        rule = int_equals("bar = 2", "bar", 2);
        res = rule.check(&map);
        assert!(res.status == Status::NotMet);
    }

    #[test]
    fn int_range_rule() {
        let map = get_test_data();
        let mut rule = int_range("1 <= foo <= 3", "foo", 1, 3);
        let mut res = rule.check(&map);
        assert!(res.status == Status::Met);

        rule = int_range("2 <= foo <= 3", "foo", 2, 3);
        res = rule.check(&map);
        assert!(res.status == Status::NotMet);

        // Values not convertible to int should be NotMet
        rule = int_range("1 <= bar <= 3", "bar", 1, 3);
        res = rule.check(&map);
        assert!(res.status == Status::NotMet);
    }

    #[test]
    fn boolean_rule() {
        let mut map = get_test_data();
        let mut rule = boolean("baz is true", "baz", true);
        let mut res = rule.check(&map);
        assert!(res.status == Status::Met);

        rule = boolean("baz is false", "baz", false);
        res = rule.check(&map);
        assert!(res.status == Status::NotMet);

        rule = boolean("bar is true", "bar", true);
        res = rule.check(&map);
        assert!(res.status == Status::NotMet);

        rule = boolean("bar is false", "bar", false);
        res = rule.check(&map);
        assert!(res.status == Status::Met);

        map.insert("quux".into(), "tRuE".into());
        rule = boolean("quux is true", "quux", true);
        res = rule.check(&map);
        assert!(res.status == Status::Met);

    }
}
