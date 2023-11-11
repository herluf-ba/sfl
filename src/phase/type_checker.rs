use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::{
    language::{
        ast::Ast,
        token::{Token, TokenKind},
        types::{Context, FreeVar, Substitutable, Substitution, TType, TypeFunc},
    },
    message::{Content, Message, Severity},
    phase::{Phase, PhaseResult},
};

pub struct TypeChecker {
    source_path: PathBuf,
    errors: Vec<Message>,
    variable_counter: usize,
}

impl TypeChecker {
    fn new(path: &PathBuf) -> Self {
        Self {
            source_path: path.clone(),
            errors: Vec::new(),
            variable_counter: 0,
        }
    }

    fn built_in(token: &Token) -> Option<TType> {
        match token.kind {
            TokenKind::Plus | TokenKind::Minus => Some(TType::Application(TypeFunc::Func {
                input: Box::new(TType::Application(TypeFunc::Int)),
                output: Box::new(TType::Application(TypeFunc::Func {
                    input: Box::new(TType::Application(TypeFunc::Int)),
                    output: Box::new(TType::Application(TypeFunc::Int)),
                })),
            })),
            _ => None,
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

    fn unify(&mut self, expr: &Ast, a: &TType, b: &TType) -> Result<Substitution, ()> {
        match (a, b) {
            (TType::Variable(x), TType::Variable(y)) if x == y => Ok(Substitution::new()),
            (TType::Variable(x), _) => {
                if a.contains(b) {
                    self.errors.push(Message {
                        severity: Severity::Error,
                        position: expr.to_owned().into(),
                        content: Content {
                            message: "infinite type detected".to_string(),
                            indicator_message: Some(" here".to_string()),
                            fix_hint: None,
                        },
                        source_path: self.source_path.clone(),
                    });
                }
                Ok(Substitution::from([(x.to_string(), b.clone())]))
            }
            (_, TType::Variable(_)) => self.unify(expr, b, a),
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
                    let s1 = self.unify(expr, i1, i2)?;
                    let s2 = self.unify(expr, &s1.apply(&**o1), &s1.apply(&**o2))?;
                    Ok(s1.apply(&s2))
                }
                (x, y) if x == y => Ok(Substitution::new()),
                (_, _) => {
                    self.errors.push(Message {
                        severity: Severity::Error,
                        position: expr.to_owned().into(),
                        content: Content {
                            message: format!("expected `{}` but found `{}`", a, b),
                            indicator_message: Some(" here".to_string()),
                            fix_hint: None,
                        },
                        source_path: self.source_path.clone(),
                    });
                    Err(())
                }
            },
            (_, _) => {
                self.errors.push(Message {
                    severity: Severity::Error,
                    position: expr.to_owned().into(),
                    content: Content {
                        message: format!("expected `{}` found `{}`", a, b),
                        indicator_message: Some(" here".to_string()),
                        fix_hint: None,
                    },
                    source_path: self.source_path.clone(),
                });
                Err(())
            }
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
                if let Some(t) = ctx.get(&n.text()) {
                    Ok((Substitution::new(), self.instantiate(t, None)))
                } else if let Some(t) = TypeChecker::built_in(n) {
                    Ok((Substitution::new(), self.instantiate(&t, None)))
                } else {
                    self.errors.push(Message {
                        severity: Severity::Error,
                        position: n.position,
                        content: Content {
                            message: format!("'{}' is not defined here", n.text()),
                            indicator_message: None,
                            fix_hint: None,
                        },
                        source_path: n.source_path.clone(),
                    });
                    Err(())
                }
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
                    e2,
                    &s2.apply(&e1_t),
                    &TType::Application(TypeFunc::Func {
                        input: Box::new(e2_t),
                        output: Box::new(var.clone()),
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
            Ast::BinaryOp(t, e1, e2) => {
                // Built an ast where the operator is a function application and type check that.
                // So "1 + 2" -> (+ 1) 2
                let ast: Ast = Ast::Application(
                    Box::new(Ast::Application(Box::new(Ast::Name(t.clone())), e1.clone())),
                    e2.clone(),
                );

                self.w(ctx, &ast)
            }
        }
    }
}

pub type Input = crate::phase::ast_builder::Output;
pub type Output = HashMap<PathBuf, TType>;
impl Phase<Input, Output> for TypeChecker {
    fn new() -> Self {
        TypeChecker::new(&PathBuf::new())
    }

    fn run(self: &mut Self, _config: &crate::config::Config, input: &Input) -> PhaseResult<Output> {
        let mut out = HashMap::new();
        let mut errs = Vec::new();

        for (source_path, ast) in input {
            *self = TypeChecker::new(source_path);
            match self.w(&Context::new(), ast) {
                Err(_) => {
                    errs.append(&mut self.errors);
                }
                Ok((s, t)) => {
                    println!("{:?}", s);
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
