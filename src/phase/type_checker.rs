use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::{
    language::{
        ast::Ast,
        token::TokenKind,
        types::{Context, FreeVar, Substitutable, Substitution, TType, TypeFunc},
    },
    message::{Content, Message, Severity},
    phase::{Phase, PhaseResult},
};

pub struct TypeChecker {
    errors: Vec<Message>,
    variable_counter: usize,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            variable_counter: 0,
        }
    }

    fn variable(&mut self) -> TType {
        let v = format!("t{}", self.variable_counter);
        self.variable_counter += 1;
        TType::Variable(v)
    }

    fn instantiate(&mut self, p: &TType, mappings: Option<Substitution>) -> TType {
        let mut m = mappings.unwrap_or_else(|| Substitution::new());

        match p {
            TType::Quantifier { variable, inner } => {
                m.insert(variable.to_owned(), self.variable());
                self.instantiate(inner, Some(m))
            }
            x => x.apply(&m),
        }
    }

    fn generalize(&mut self, ctx: &Context, p: &TType) -> TType {
        let quantifiers = p
            .free_variables()
            .difference(&ctx.free_variables())
            .map(|v| v.to_owned())
            .collect::<HashSet<String>>();
        quantifiers
            .into_iter()
            .fold(p.clone(), |p, quantifier| TType::Quantifier {
                variable: quantifier.to_owned(),
                inner: Box::new(p),
            })
    }

    fn unify(&self, a: &TType, b: &TType) -> Result<Substitution, ()> {
        match (a, b) {
            (TType::Variable(x), TType::Variable(y)) if x == y => Ok(Substitution::new()),
            (TType::Variable(x), _) => {
                if a.contains(b) {
                    // TODO: Return Err here and add error message to errors
                    panic!("Infinite type detected!");
                }
                Ok(Substitution::from([(x.to_string(), b.clone())]))
            }
            (_, TType::Variable(_)) => self.unify(b, a),
            (TType::Application(x), TType::Application(y)) => match (x, y) {
                (
                    TypeFunc::Func {
                        input: i1,
                        output: o1,
                    },
                    TypeFunc::Func {
                        input: i2,
                        output: o2,
                    },
                ) => {
                    // TODO: this may be buggy '(\x -> true) false' becomes generic
                    let mut s = Substitution::new();
                    s = self.unify(i1, i2)?.apply(&s);
                    s = self.unify(&s.apply(&**o1), &s.apply(&**o2))?.apply(&s);
                    Ok(s)
                }
                (x, y) if x == y => Ok(Substitution::new()),
                // TODO: Return Err here and add error message to errors
                (x, y) => panic!("cannot unify '{:?}' and '{:?}'", x, y),
            },
            (x, y) => panic!("cannot unify '{:?}' and '{:?}'", x, y),
        }
    }

    fn w(&mut self, ctx: &Context, expr: &Ast) -> Result<(Substitution, TType), ()> {
        match expr {
            Ast::Expr(e) => self.w(ctx, e),
            Ast::Literal(l) => match l.kind {
                TokenKind::LiteralInt(_) => {
                    Ok((Substitution::new(), TType::Application(TypeFunc::Int)))
                }
                TokenKind::LiteralBool(_) => {
                    Ok((Substitution::new(), TType::Application(TypeFunc::Bool)))
                }
                _ => Err(()),
            },
            Ast::Name(n) => {
                let name = n.text();
                let Some(t) = ctx.get(&name) else {
                    self.errors.push(Message {
                        severity: Severity::Error,
                        position: n.position,
                        content: Content {
                            message: format!("'{}' is not defined here", name),
                            indicator_message: None,
                            fix_hint: None,
                        },
                        source_path: n.source_path.clone(),
                    });
                    return Err(());
                };

                Ok((Substitution::new(), self.instantiate(t, None)))
            }
            Ast::Abstraction(n, e) => {
                let var = self.variable();
                let mut ctx_with_var = ctx.clone();
                ctx_with_var.insert(n.text(), var.clone());
                let (s, e_t) = self.w(&ctx_with_var, e)?;
                let t = s.apply(&TType::Application(TypeFunc::Func {
                    input: Box::new(var),
                    output: Box::new(e_t),
                }));
                Ok((s, t))
            }
            Ast::Application(e1, e2) => {
                let (s1, e1_t) = self.w(ctx, e1)?;
                let (s2, e2_t) = self.w(&s1.apply(ctx), e2)?;
                let var = self.variable();
                let s3 = self.unify(
                    &s2.apply(&e1_t),
                    &TType::Application(TypeFunc::Func {
                        input: Box::new(var.clone()),
                        output: Box::new(e2_t),
                    }),
                )?;

                Ok((s3.apply(&s2.apply(&s1)), s3.apply(&var)))
            }
            Ast::Let(n, e1, e2) => {
                let (s1, e1_t) = self.w(ctx, e1)?;
                let mut new_ctx = s1.apply(ctx);
                new_ctx.insert(n.text(), self.generalize(&new_ctx, &e1_t));
                let (s2, e2_t) = self.w(&new_ctx, e2)?;

                Ok((s2.apply(&s1), e2_t))
            }
            Ast::Err => Err(()),
        }
    }
}

pub type Input = crate::phase::ast_builder::Output;
pub type Output = HashMap<PathBuf, TType>;
impl Phase<Input, Output> for TypeChecker {
    fn new() -> Self {
        TypeChecker::new()
    }

    fn run(self: &mut Self, _config: &crate::config::Config, input: &Input) -> PhaseResult<Output> {
        let mut out = HashMap::new();
        let mut errs = HashMap::new();

        for (source_path, ast) in input {
            *self = TypeChecker::new();
            match self.w(&Context::new(), ast) {
                Err(_) => {
                    errs.insert(source_path.clone(), self.errors.clone());
                }
                Ok((_, t)) => {
                    out.insert(source_path.clone(), t);
                }
            };
        }

        if !errs.is_empty() {
            PhaseResult::Err(errs)
        } else {
            PhaseResult::Ok(out)
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
