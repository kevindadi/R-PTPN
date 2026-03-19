//! 线性约束系统：Fourier-Motzkin 消元、LP 可满足性

use crate::model::TransId;
use crate::rational::{q_to_f64, Q};
use std::collections::{HashMap, HashSet};

/// 约束变量标识
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VarId {
    H(TransId),
    W(TransId),
    Tau,
    TauStep(usize),
    HStep(usize, TransId),
    WStep(usize, TransId),
}

/// 线性表达式: Σ coeff_i * var_i + constant
#[derive(Clone, Debug)]
pub struct LinExpr {
    pub terms: HashMap<VarId, Q>,
    pub constant: Q,
}

impl LinExpr {
    pub fn new() -> Self {
        Self {
            terms: HashMap::new(),
            constant: Q::from(0),
        }
    }

    pub fn constant(c: Q) -> Self {
        Self {
            terms: HashMap::new(),
            constant: c,
        }
    }

    pub fn single_var(var: VarId, coeff: Q) -> Self {
        let mut terms = HashMap::new();
        terms.insert(var, coeff);
        Self {
            terms,
            constant: Q::from(0),
        }
    }

    pub fn add_term(&mut self, var: VarId, coeff: Q) {
        if coeff != Q::from(0) {
            *self.terms.entry(var).or_insert_with(|| Q::from(0)) += coeff;
        }
    }

    pub fn coeff(&self, var: &VarId) -> Q {
        self.terms.get(var).cloned().unwrap_or_else(|| Q::from(0))
    }

    pub fn substitute(&self, var: &VarId, expr: &LinExpr) -> LinExpr {
        let coeff = self.coeff(var);
        if coeff == Q::from(0) {
            return self.clone();
        }
        let mut result = LinExpr::new();
        result.constant = self.constant.clone() + &coeff * &expr.constant;
        for (v, c) in &self.terms {
            if v == var {
                continue;
            }
            result.add_term(v.clone(), c.clone());
        }
        for (v, c) in &expr.terms {
            result.add_term(v.clone(), &coeff * c);
        }
        result
    }

    pub fn negate(&self) -> LinExpr {
        let mut terms = HashMap::new();
        for (v, c) in &self.terms {
            terms.insert(v.clone(), -c.clone());
        }
        LinExpr {
            terms,
            constant: -&self.constant,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.terms.is_empty() && self.constant == Q::from(0)
    }
}

impl Default for LinExpr {
    fn default() -> Self {
        Self::new()
    }
}

/// 约束类型
#[derive(Clone, Debug)]
pub enum ConstraintOp {
    Le,
    Ge,
    Eq,
}

/// 单条线性约束: expr op 0
#[derive(Clone, Debug)]
pub struct LinearConstraint {
    pub expr: LinExpr,
    pub op: ConstraintOp,
}

/// 线性约束系统
#[derive(Clone, Debug)]
pub struct ConstraintSystem {
    pub constraints: Vec<LinearConstraint>,
    pub variables: HashSet<VarId>,
}

impl ConstraintSystem {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            variables: HashSet::new(),
        }
    }

    fn add_var(&mut self, var: &VarId) {
        self.variables.insert(var.clone());
    }

    pub fn add_eq(&mut self, var: VarId, val: Q) {
        self.add_var(&var);
        let mut expr = LinExpr::single_var(var, Q::from(1));
        expr.constant = -val;
        self.constraints.push(LinearConstraint {
            expr,
            op: ConstraintOp::Eq,
        });
    }

    pub fn add_le(&mut self, lhs: LinExpr, rhs: LinExpr) {
        for v in lhs.terms.keys().chain(rhs.terms.keys()) {
            self.add_var(v);
        }
        let mut expr = lhs;
        expr.constant -= &rhs.constant;
        for (v, c) in rhs.terms {
            expr.add_term(v, -c);
        }
        self.constraints.push(LinearConstraint {
            expr,
            op: ConstraintOp::Le,
        });
    }

    pub fn add_ge(&mut self, lhs: LinExpr, rhs: LinExpr) {
        self.add_le(rhs, lhs);
    }

    pub fn add_constraint(&mut self, c: LinearConstraint) {
        for v in c.expr.terms.keys() {
            self.add_var(v);
        }
        self.constraints.push(c);
    }

    pub fn substitute(&mut self, var: &VarId, expr: &LinExpr) {
        let mut new_constraints = Vec::new();
        for c in &self.constraints {
            let coeff = c.expr.coeff(var);
            if coeff != Q::from(0) {
                let new_expr = c.expr.substitute(var, expr);
                new_constraints.push(LinearConstraint {
                    expr: new_expr,
                    op: c.op.clone(),
                });
            } else {
                new_constraints.push(c.clone());
            }
        }
        self.constraints = new_constraints;
        self.variables.remove(var);
    }

    /// Fourier-Motzkin 消元：消除变量 var
    pub fn eliminate(&mut self, var: &VarId) {
        let mut pos = Vec::new();
        let mut neg = Vec::new();
        let mut zero = Vec::new();
        for c in std::mem::take(&mut self.constraints) {
            let coeff = c.expr.coeff(var);
            if coeff > Q::from(0) {
                pos.push(c);
            } else if coeff < Q::from(0) {
                neg.push(c);
            } else {
                zero.push(c);
            }
        }

        let mut new_constraints = zero;

        for c_pos in &pos {
            let coeff_pos = c_pos.expr.coeff(var);
            for c_neg in &neg {
                let coeff_neg = c_neg.expr.coeff(var);
                let mut combined = LinExpr::new();
                combined.constant = &c_neg.expr.constant * &coeff_pos / coeff_neg.clone()
                    - &c_pos.expr.constant * &coeff_neg / coeff_pos.clone();
                for (v, c) in &c_pos.expr.terms {
                    if v != var {
                        combined
                            .add_term(v.clone(), c.clone() * coeff_neg.clone() / coeff_pos.clone());
                    }
                }
                for (v, c) in &c_neg.expr.terms {
                    if v != var {
                        combined.add_term(
                            v.clone(),
                            -c.clone() * coeff_pos.clone() / coeff_neg.clone(),
                        );
                    }
                }
                if !combined.is_zero() {
                    new_constraints.push(LinearConstraint {
                        expr: combined,
                        op: ConstraintOp::Le,
                    });
                }
            }
        }

        self.constraints = new_constraints;
        self.variables.remove(var);
    }

    /// 投影到指定变量集
    pub fn project(&mut self, keep: &HashSet<VarId>) {
        let to_eliminate: Vec<_> = self
            .variables
            .iter()
            .filter(|v| !keep.contains(*v))
            .cloned()
            .collect();
        for var in to_eliminate {
            self.eliminate(&var);
        }
    }

    /// 可满足性检查
    pub fn is_satisfiable(&self) -> bool {
        crate::constraints::lp::lp_satisfiable(self)
    }

    /// 获取某变量的上下界
    pub fn get_bounds(&self, var: &VarId) -> (Option<Q>, Option<Q>) {
        crate::constraints::lp::lp_bounds(self, var)
    }

    /// 合取两个约束系统
    pub fn conjunction(&mut self, other: &ConstraintSystem) {
        for v in &other.variables {
            self.variables.insert(v.clone());
        }
        for c in &other.constraints {
            self.constraints.push(c.clone());
        }
    }

    pub fn clone_system(&self) -> ConstraintSystem {
        self.clone()
    }
}

impl Default for ConstraintSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// LP 求解（good_lp）
pub mod lp {
    use super::*;
    use good_lp::{variable, ProblemVariables, Solution, SolverModel, Variable};
    use std::collections::HashMap;

    #[allow(dead_code)]
    fn var_to_name(v: &VarId) -> String {
        match v {
            VarId::H(t) => format!("h_{}", t),
            VarId::W(t) => format!("w_{}", t),
            VarId::Tau => "tau".to_string(),
            VarId::TauStep(i) => format!("tau_{}", i),
            VarId::HStep(i, t) => format!("h_{}_{}", i, t),
            VarId::WStep(i, t) => format!("w_{}_{}", i, t),
        }
    }

    pub fn lp_satisfiable(cs: &ConstraintSystem) -> bool {
        if cs.variables.is_empty() {
            return cs.constraints.iter().all(|c| {
                let val = &c.expr.constant;
                match &c.op {
                    ConstraintOp::Le => val <= &Q::from(0),
                    ConstraintOp::Ge => val >= &Q::from(0),
                    ConstraintOp::Eq => val == &Q::from(0),
                }
            });
        }

        let mut problem = ProblemVariables::new();
        let mut var_map: HashMap<VarId, Variable> = HashMap::new();
        for v in &cs.variables {
            let var = problem.add(variable().min(0));
            var_map.insert(v.clone(), var);
        }

        let mut obj = good_lp::Expression::from(0);
        for (v, _) in &var_map {
            obj = obj + var_map[v];
        }

        let mut prob = problem.minimise(obj).using(good_lp::microlp);

        for c in &cs.constraints {
            let mut expr = good_lp::Expression::from(q_to_f64(&c.expr.constant));
            for (v, coeff) in &c.expr.terms {
                if let Some(var) = var_map.get(v) {
                    expr = expr + q_to_f64(coeff) * *var;
                }
            }
            match &c.op {
                ConstraintOp::Le => {
                    prob = prob.with(expr.leq(0.));
                }
                ConstraintOp::Ge => {
                    prob = prob.with(expr.geq(0.));
                }
                ConstraintOp::Eq => {
                    prob = prob.with(expr.eq(0.));
                }
            }
        }

        prob.solve().is_ok()
    }

    pub fn lp_bounds(cs: &ConstraintSystem, var: &VarId) -> (Option<Q>, Option<Q>) {
        if !cs.variables.contains(var) {
            return (None, None);
        }

        let build_problem = |sense: f64| {
            let mut problem = ProblemVariables::new();
            let mut var_map: HashMap<VarId, Variable> = HashMap::new();
            for v in &cs.variables {
                let rvar = problem.add(variable().min(0));
                var_map.insert(v.clone(), rvar);
            }
            let target = var_map[var];
            let mut obj = good_lp::Expression::from(0.);
            obj = obj + sense * target;

            let mut prob = problem.minimise(obj).using(good_lp::microlp);
            for c in &cs.constraints {
                let mut expr = good_lp::Expression::from(q_to_f64(&c.expr.constant));
                for (v, coeff) in &c.expr.terms {
                    if let Some(rvar) = var_map.get(v) {
                        expr = expr + q_to_f64(coeff) * *rvar;
                    }
                }
                match &c.op {
                    ConstraintOp::Le => prob = prob.with(expr.leq(0.)),
                    ConstraintOp::Ge => prob = prob.with(expr.geq(0.)),
                    ConstraintOp::Eq => prob = prob.with(expr.eq(0.)),
                }
            }
            (prob, target)
        };

        let (prob_min, target) = build_problem(1.);
        let lb = prob_min
            .solve()
            .ok()
            .and_then(|s| crate::rational::f64_to_q(s.value(target)));

        let (prob_max, target) = build_problem(-1.);
        let ub = prob_max
            .solve()
            .ok()
            .and_then(|s| crate::rational::f64_to_q(s.value(target)));

        (lb, ub)
    }
}
