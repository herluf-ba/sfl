use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
};

#[derive(Clone, Debug, PartialEq)]
pub enum TypeFunc {
    Func {
        input: Box<TType>,
        output: Box<TType>,
    },
    Bool,
    Number,
}

impl Display for TypeFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TypeFunc::Func { input, output } => format!("{} -> {}", input, output),
                TypeFunc::Bool => "bool".to_string(),
                TypeFunc::Number => "int".to_string(),
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TType {
    Variable(String),
    Application(TypeFunc),
    Quantifier { variable: String, inner: Box<TType> },
}

impl TType {
    pub fn contains(&self, other: &TType) -> bool {
        let TType::Variable(n) = other else {
            return false;
        };
        match self {
            TType::Variable(name) => name == n,
            TType::Quantifier { variable: _, inner } => inner.contains(other),
            TType::Application(f) => match f {
                TypeFunc::Func { input, output } => input.contains(other) || output.contains(other),
                _ => false,
            },
        }
    }
}

impl Display for TType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TType::Variable(v) => v.to_owned(),
                TType::Application(f) => format!("{}", f),
                TType::Quantifier { variable, inner } => format!("∀{} {}", variable, inner),
            }
        )
    }
}

pub type Context = HashMap<String, TType>;

pub struct Substitution(HashMap<String, TType>);

impl Substitution {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn from<T: IntoIterator<Item = (String, TType)>>(cases: T) -> Self {
        Self(HashMap::from_iter::<T>(cases.into()))
    }

    pub fn apply<T: Substitutable>(&self, v: &T) -> T {
        v.apply(self)
    }

    pub fn insert(&mut self, k: String, v: TType) -> Option<TType> {
        self.0.insert(k, v)
    }
}

impl Debug for Substitution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = self
            .0
            .iter()
            .map(|(k, v)| format!("{:?} -> {:?}", k, v))
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", text)
    }
}

pub trait FreeVar {
    fn free_variables(&self) -> HashSet<String>;
}

impl FreeVar for TType {
    fn free_variables(&self) -> HashSet<String> {
        match self {
            TType::Variable(name) => HashSet::from([name.to_owned()]),
            TType::Application(f) => match f {
                TypeFunc::Func { input, output } => {
                    let mut ifvs = input.free_variables();
                    let ofvs = output.free_variables();
                    ifvs.extend(ofvs.into_iter());
                    ifvs
                }
                _ => HashSet::new(),
            },
            TType::Quantifier { variable, inner } => inner
                .free_variables()
                .into_iter()
                .filter(|v| v != variable)
                .collect(),
        }
    }
}

impl FreeVar for Context {
    fn free_variables(&self) -> HashSet<String> {
        self.values().flat_map(|v| v.free_variables()).collect()
    }
}

pub trait Substitutable {
    fn apply(&self, s: &Substitution) -> Self;
}

impl Substitutable for TType {
    fn apply(&self, s: &Substitution) -> Self {
        match self {
            TType::Variable(name) => {
                s.0.get(name)
                    .map_or(TType::Variable(name.to_owned()), |to| to.clone())
            }
            TType::Application(func) => match func {
                TypeFunc::Func { input, output } => TType::Application(TypeFunc::Func {
                    input: Box::new(input.apply(s)),
                    output: Box::new(output.apply(s)),
                }),
                _ => self.clone(),
            },
            TType::Quantifier { variable, inner } => TType::Quantifier {
                variable: variable.to_owned(),
                inner: Box::new(inner.apply(s)),
            },
        }
    }
}

impl Substitutable for Context {
    fn apply(&self, s: &Substitution) -> Self {
        self.iter()
            .map(|(k, v)| (k.to_owned(), v.apply(s)))
            .collect::<HashMap<String, TType>>()
    }
}

impl Substitutable for Substitution {
    fn apply(&self, s: &Substitution) -> Self {
        let mut new = self.0.clone();
        new.extend(s.0.iter().map(|(k, v)| (k.to_owned(), v.apply(self))));
        Substitution::from(new)
    }
}
