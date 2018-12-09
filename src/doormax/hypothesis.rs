use std::fmt;

use enum_map::EnumMap;

use doormax::condition::Condition;
use doormax::term::Term;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Require {
    None,
    True,
    False,
}

impl Default for Require {
    fn default() -> Self {
        Require::None
    }
}

impl From<bool> for Require {
    fn from(val: bool) -> Self {
        if val {
            Require::True
        } else {
            Require::False
        }
    }
}

#[derive(Clone, Debug)]
pub struct Hypothesis(EnumMap<Term, Require>);

impl Hypothesis {
    // pub fn combine(&self, &Hypothesis(ref other_map): &Hypothesis) -> Hypothesis {
    //     let &Hypothesis(ref self_map) = self;

    //     let mut result_map = *self_map;

    //     for (key, &value) in self_map {
    //         if value != Require::None {
    //             result_map[key] = if other_map[key] == value {
    //                 value
    //             } else {
    //                 Require::None
    //             };
    //         }
    //     }

    //     Hypothesis(result_map)
    // }

    pub fn combine_cond(&self, &Condition(ref cond_map): &Condition) -> Hypothesis {
        let &Hypothesis(ref self_map) = self;

        let mut result_map = *self_map;

        for (key, &value) in self_map {
            match value {
                Require::True => {
                    if !cond_map[key] {
                        result_map[key] = Require::None;
                    }
                }

                Require::False => {
                    if cond_map[key] {
                        result_map[key] = Require::None;
                    }
                }

                Require::None => {}
            }
        }

        Hypothesis(result_map)
    }

    pub fn matches(&self, &Hypothesis(ref other): &Hypothesis) -> bool {
        let &Hypothesis(ref self_map) = self;

        self_map
            .iter()
            .all(|(key, &value)| value == Require::None || other[key] == value)
    }

    pub fn matches_cond(&self, &Condition(ref cond_map): &Condition) -> bool {
        let &Hypothesis(ref self_map) = self;

        self_map.iter().all(|(key, &value)| match value {
            Require::None => true,
            Require::True => cond_map[key],
            Require::False => !cond_map[key],
        })
    }
}

impl fmt::Display for Hypothesis {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Hypothesis(ref hyp_map) = self;

        fn show_require(r: Require) -> &'static str {
            match r {
                Require::None => "*",
                Require::True => "1",
                Require::False => "0",
            }
        }

        write!(
            f,
            "Condition({}{}{}{} {}{}{})",
            show_require(hyp_map[Term::TouchWallN]),
            show_require(hyp_map[Term::TouchWallS]),
            show_require(hyp_map[Term::TouchWallE]),
            show_require(hyp_map[Term::TouchWallW]),
            show_require(hyp_map[Term::OnPassenger]),
            show_require(hyp_map[Term::OnDestination]),
            show_require(hyp_map[Term::HasPassenger]),
        )
    }
}

impl From<Condition> for Hypothesis {
    fn from(Condition(cond_map): Condition) -> Self {
        Hypothesis(enum_map! {
            Term::TouchWallN => Require::from(cond_map[Term::TouchWallN]),
            Term::TouchWallS => Require::from(cond_map[Term::TouchWallS]),
            Term::TouchWallE => Require::from(cond_map[Term::TouchWallE]),
            Term::TouchWallW => Require::from(cond_map[Term::TouchWallW]),
            Term::OnPassenger => Require::from(cond_map[Term::OnPassenger]),
            Term::OnDestination => Require::from(cond_map[Term::OnDestination]),
            Term::HasPassenger => Require::from(cond_map[Term::HasPassenger]),
        })
    }
}

#[cfg(test)]
mod test_hypthothesis {

    use super::*;

    #[test]
    fn hypothesis_matches() {
        let hyp_a = Hypothesis(enum_map! {
            Term::TouchWallN => Require::True,
            Term::TouchWallS => Require::True,
            Term::TouchWallE => Require::True,
            Term::TouchWallW => Require::True,
            Term::OnPassenger => Require::True,
            Term::OnDestination => Require::True,
            Term::HasPassenger => Require::True,
        });

        let hyp_b = Hypothesis(enum_map! {
            Term::TouchWallN => Require::False,
            Term::TouchWallS => Require::False,
            Term::TouchWallE => Require::False,
            Term::TouchWallW => Require::False,
            Term::OnPassenger => Require::False,
            Term::OnDestination => Require::False,
            Term::HasPassenger => Require::False,
        });

        let hyp_c = Hypothesis(enum_map! {
            Term::TouchWallN => Require::True,
            Term::TouchWallS => Require::None,
            Term::TouchWallE => Require::None,
            Term::TouchWallW => Require::None,
            Term::OnPassenger => Require::None,
            Term::OnDestination => Require::None,
            Term::HasPassenger => Require::None,
        });

        let hyp_d = Hypothesis(enum_map! {
            Term::TouchWallN => Require::True,
            Term::TouchWallS => Require::False,
            Term::TouchWallE => Require::None,
            Term::TouchWallW => Require::None,
            Term::OnPassenger => Require::None,
            Term::OnDestination => Require::None,
            Term::HasPassenger => Require::None,
        });

        let hyp_e = Hypothesis(enum_map! {
            Term::TouchWallN => Require::True,
            Term::TouchWallS => Require::False,
            Term::TouchWallE => Require::True,
            Term::TouchWallW => Require::True,
            Term::OnPassenger => Require::True,
            Term::OnDestination => Require::True,
            Term::HasPassenger => Require::True,
        });

        assert!(hyp_a.matches(&hyp_a));
        assert!(!hyp_a.matches(&hyp_b));
        assert!(!hyp_a.matches(&hyp_c));
        assert!(!hyp_a.matches(&hyp_d));
        assert!(!hyp_a.matches(&hyp_e));

        assert!(!hyp_b.matches(&hyp_a));
        assert!(hyp_b.matches(&hyp_b));
        assert!(!hyp_b.matches(&hyp_c));
        assert!(!hyp_b.matches(&hyp_d));
        assert!(!hyp_b.matches(&hyp_e));

        assert!(hyp_c.matches(&hyp_a));
        assert!(!hyp_c.matches(&hyp_b));
        assert!(hyp_c.matches(&hyp_c));
        assert!(hyp_c.matches(&hyp_d));
        assert!(hyp_c.matches(&hyp_e));

        assert!(!hyp_d.matches(&hyp_a));
        assert!(!hyp_d.matches(&hyp_b));
        assert!(!hyp_d.matches(&hyp_c));
        assert!(hyp_d.matches(&hyp_d));
        assert!(hyp_d.matches(&hyp_e));

        assert!(!hyp_e.matches(&hyp_a));
        assert!(!hyp_e.matches(&hyp_b));
        assert!(!hyp_e.matches(&hyp_c));
        assert!(!hyp_e.matches(&hyp_d));
        assert!(hyp_e.matches(&hyp_e));
    }

    #[test]
    fn hypothesis_matches_cond() {
        let hyp_a = Hypothesis(enum_map! {
            Term::TouchWallN => Require::True,
            Term::TouchWallS => Require::None,
            Term::TouchWallE => Require::None,
            Term::TouchWallW => Require::None,
            Term::OnPassenger => Require::None,
            Term::OnDestination => Require::None,
            Term::HasPassenger => Require::None,
        });

        let hyp_b = Hypothesis(enum_map! {
            Term::TouchWallN => Require::True,
            Term::TouchWallS => Require::False,
            Term::TouchWallE => Require::None,
            Term::TouchWallW => Require::None,
            Term::OnPassenger => Require::None,
            Term::OnDestination => Require::None,
            Term::HasPassenger => Require::None,
        });

        let hyp_c = Hypothesis(enum_map! {
            Term::TouchWallN => Require::True,
            Term::TouchWallS => Require::False,
            Term::TouchWallE => Require::True,
            Term::TouchWallW => Require::True,
            Term::OnPassenger => Require::True,
            Term::OnDestination => Require::True,
            Term::HasPassenger => Require::True,
        });

        let cond_a = Condition(enum_map! {
            Term::TouchWallN => true,
            Term::TouchWallS => false,
            Term::TouchWallE => false,
            Term::TouchWallW => false,
            Term::OnPassenger => false,
            Term::OnDestination => false,
            Term::HasPassenger => false,
        });

        let cond_b = Condition(enum_map! {
            Term::TouchWallN => true,
            Term::TouchWallS => false,
            Term::TouchWallE => false,
            Term::TouchWallW => false,
            Term::OnPassenger => false,
            Term::OnDestination => false,
            Term::HasPassenger => true,
        });

        let cond_c = Condition(enum_map! {
            Term::TouchWallN => false,
            Term::TouchWallS => false,
            Term::TouchWallE => false,
            Term::TouchWallW => false,
            Term::OnPassenger => false,
            Term::OnDestination => false,
            Term::HasPassenger => false,
        });

        assert!(hyp_a.matches_cond(&cond_a));
        assert!(hyp_a.matches_cond(&cond_b));
        assert!(!hyp_a.matches_cond(&cond_c));

        assert!(hyp_b.matches_cond(&cond_a));
        assert!(hyp_b.matches_cond(&cond_b));
        assert!(!hyp_b.matches_cond(&cond_c));

        assert!(!hyp_c.matches_cond(&cond_a));
        assert!(!hyp_c.matches_cond(&cond_b));
        assert!(!hyp_c.matches_cond(&cond_c));
    }

}
