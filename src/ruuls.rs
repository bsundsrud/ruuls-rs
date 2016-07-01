use std::collections::BTreeMap;
use std::ops::{BitOr, BitAnd};

/// ***********************************************************************
/// STATUS
/// **********************************************************************
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Status {
    Met,
    NotMet,
    Unknown,
}

impl BitAnd for Status {
    type Output = Status;
    fn bitand(self, rhs: Status) -> Status {
        match (self, rhs) {
            (Status::Met, Status::Met) => Status::Met,
            (Status::NotMet, _) |
            (_, Status::NotMet) => Status::NotMet,
            (_, _) => Status::Unknown,
        }
    }
}

impl BitOr for Status {
    type Output = Status;
    fn bitor(self, rhs: Status) -> Status {
        match (self, rhs) {
            (Status::NotMet, Status::NotMet) => Status::NotMet,
            (Status::Met, _) | (_, Status::Met) => Status::Met,
            (_, _) => Status::Unknown,
        }
    }
}

/// ***********************************************************************
/// CHECKLIST
/// **********************************************************************
#[derive(Debug)]
pub enum Checklist {
    And(Vec<Checklist>),
    Or(Vec<Checklist>),
    NumberOf(usize, Vec<Checklist>),
    // Rule(Description, Field, Constraint)
    Rule(String, String, Constraint),
}

impl Checklist {
    pub fn check(&self, info: &BTreeMap<String, String>) -> ChecklistResult {
        match *self {
            Checklist::And(ref checklists) => {
                let mut status = Status::Met;
                let children = checklists.iter()
                    .map(|c| c.check(info))
                    .inspect(|r| status = status & r.status)
                    .collect::<Vec<_>>();
                ChecklistResult {
                    name: "And".into(),
                    status: status,
                    children: children,
                }
            }
            Checklist::Or(ref checklists) => {
                let mut status = Status::NotMet;
                let children = checklists.iter()
                    .map(|c| c.check(info))
                    .inspect(|r| status = status | r.status)
                    .collect::<Vec<_>>();
                ChecklistResult {
                    name: "Or".into(),
                    status: status,
                    children: children,
                }
            }
            Checklist::NumberOf(count, ref checklists) => {
                let mut met_count = 0;
                let mut failed_count = 0;
                let children = checklists.iter()
                    .map(|c| c.check(info))
                    .inspect(|r| {
                        if r.status == Status::Met {
                            met_count += 1;
                        } else if r.status == Status::NotMet {
                            failed_count += 1;
                        }
                    })
                    .collect::<Vec<_>>();
                let status = if met_count >= count {
                    Status::Met
                } else if failed_count >= count {
                    Status::NotMet
                } else {
                    Status::Unknown
                };
                ChecklistResult {
                    name: format!("At least {} of", count),
                    status: status,
                    children: children,
                }


            }
            Checklist::Rule(ref name, ref field, ref constraint) => {
                let status = if let Some(s) = info.get(field) {
                    constraint.check(s)
                } else {
                    Status::Unknown
                };
                ChecklistResult {
                    name: name.to_owned(),
                    status: status,
                    children: Vec::new(),
                }
            }
        }
    }

    pub fn rule(description: &str, field: &str, constraint: Constraint) -> Checklist {
        Checklist::Rule(description.into(), field.into(), constraint)
    }
}

/// ***********************************************************************
/// CONSTRAINT
/// **********************************************************************
#[derive(Debug)]
pub enum Constraint {
    StringEquals(String),
    IntEquals(i32),
    IntRange(i32, i32),
    Boolean(bool),
}

impl Constraint {
    pub fn check(&self, val: &str) -> Status {
        match *self {
            Constraint::StringEquals(ref s) => {
                if val == s {
                    Status::Met
                } else {
                    Status::NotMet
                }
            }
            Constraint::IntEquals(i) => {
                let parse_res = val.parse::<i32>();
                if let Ok(val) = parse_res {
                    if val == i {
                        Status::Met
                    } else {
                        Status::NotMet
                    }
                } else {
                    Status::NotMet
                }
            }
            Constraint::IntRange(start, end) => {
                let parse_res = val.parse::<i32>();
                if let Ok(val) = parse_res {
                    if start <= val && val <= end {
                        Status::Met
                    } else {
                        Status::NotMet
                    }
                } else {
                    Status::NotMet
                }
            }
            Constraint::Boolean(b) => {
                let bool_val = &val.to_lowercase() == "true";
                if bool_val == b {
                    Status::Met
                } else {
                    Status::NotMet
                }
            }
        }
    }
}

/// ***********************************************************************
/// CHECKLIST RESULT
/// **********************************************************************
#[derive(Debug)]
pub struct ChecklistResult {
    name: String,
    status: Status,
    children: Vec<ChecklistResult>,
}

/// ***********************************************************************
/// TESTS
/// **********************************************************************
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use super::{Constraint, Checklist};
        use std::collections::BTreeMap;
        let mut map = BTreeMap::new();
        map.insert("foo".into(), "1".into());
        map.insert("baz".into(), "true".into());
        map.insert("bar".into(), "3".into());
        let tree = Checklist::NumberOf(2,
                                       vec![Checklist::Rule("foo is 1".into(),
                                                            "foo".into(),
                                                            Constraint::StringEquals("1".into())),
                                            Checklist::Rule("bar is 2".into(),
                                                            "bar".into(),
                                                            Constraint::IntEquals(2)),
                                            Checklist::Rule("baz is true".into(),
                                                            "baz".into(),
                                                            Constraint::Boolean(true))]);
        let res = tree.check(&map);
        println!("{:?}", res);
    }
}