//! SCG 构建（Algorithms 1-3）

use crate::constraints::{ConstraintSystem, LinExpr, VarId};
use crate::model::{PTPN, StateClass, SCG};
use crate::rational::Q;
use std::collections::HashSet;

/// Algorithm 1: isSchedulable — 计算触发域 Θ_{t_f}(C)
pub fn is_schedulable(
    ptpn: &PTPN,
    class: &StateClass,
    t_f: &str,
) -> Option<ConstraintSystem> {
    let t_f = t_f.to_string();
    let e_m = ptpn.enabled(&class.m);
    if !e_m.contains(&t_f) {
        return None;
    }

    let mut phi_prime = class.phi.clone_system();

    let var_tau = VarId::Tau;
    phi_prime.variables.insert(var_tau.clone());

    phi_prime.add_ge(
        LinExpr::single_var(var_tau.clone(), Q::from(1)),
        LinExpr::constant(Q::from(0)),
    );

    if let Some(si) = ptpn.si.get(&t_f) {
        let mut lhs = LinExpr::new();
        lhs.add_term(VarId::H(t_f.clone()), Q::from(1));
        lhs.add_term(var_tau.clone(), Q::from(1));
        phi_prime.add_ge(lhs, LinExpr::constant(si.lo.clone()));
    }

    for t in &e_m {
        if let Some(si) = ptpn.si.get(t) {
            if let Some(hi) = &si.hi {
                let mut lhs = LinExpr::new();
                lhs.add_term(VarId::H(t.clone()), Q::from(1));
                lhs.add_term(var_tau.clone(), Q::from(1));
                phi_prime.add_le(lhs, LinExpr::constant(hi.clone()));
            }
        }
    }

    add_psi_prio(ptpn, class, &t_f, &e_m, &mut phi_prime);

    let keep: HashSet<VarId> = [var_tau.clone()].into_iter().collect();
    phi_prime.project(&keep);

    if phi_prime.is_satisfiable() {
        Some(phi_prime)
    } else {
        None
    }
}

fn add_psi_prio(
    ptpn: &PTPN,
    class: &StateClass,
    t_f: &str,
    e_m: &HashSet<String>,
    phi: &mut ConstraintSystem,
) {
    let req = match ptpn.req.get(t_f) {
        Some(r) => r.clone(),
        None => return,
    };
    if req.is_empty() {
        return;
    }

    for r in &req {
        let m_r = *class.m.get(r.as_str()).unwrap_or(&0) as usize;
        let higher_prio: Vec<_> = e_m
            .iter()
            .filter(|t| {
                ptpn.req.get(t.as_str()).map_or(false, |req_set| req_set.contains(r.as_str()))
                    && ptpn.demand(t, r) > 0
                    && ptpn.pri.get(t.as_str()).copied().unwrap_or(0) > ptpn.pri.get(t_f).copied().unwrap_or(0)
            })
            .cloned()
            .collect();

        if higher_prio.len() >= m_r {
            for t_h in &higher_prio[..m_r] {
                if let Some(si) = ptpn.si.get(t_h) {
                    let mut lhs = LinExpr::new();
                    lhs.add_term(VarId::H(t_h.clone()), Q::from(1));
                    lhs.add_term(VarId::Tau, Q::from(1));
                    lhs.constant = -si.lo.clone();
                    phi.add_le(lhs, LinExpr::constant(Q::from(0)));
                }
            }
        }
    }
}

/// Algorithm 2: succ — 计算后继状态类
pub fn successor(
    ptpn: &PTPN,
    class: &StateClass,
    t_f: &str,
    theta: &ConstraintSystem,
    next_id: usize,
) -> StateClass {
    let t_f = t_f.to_string();
    let m_prime = ptpn.fire_marking(&class.m, &t_f);
    let new_enabled = ptpn.newly_enabled(&class.m, &m_prime, &t_f);
    let e_m_prime = ptpn.enabled(&m_prime);
    let pe: HashSet<_> = e_m_prime.difference(&new_enabled).cloned().collect();
    let e_m = ptpn.enabled(&class.m);

    let mut phi_prime = class.phi.clone_system();

    let var_tau = VarId::Tau;
    phi_prime.variables.insert(var_tau.clone());
    for c in &theta.constraints {
        phi_prime.add_constraint(c.clone());
    }

    for t in &e_m {
        let mut expr = LinExpr::new();
        expr.add_term(VarId::H(t.clone()), Q::from(1));
        expr.add_term(var_tau.clone(), -Q::from(1));
        phi_prime.substitute(&VarId::H(t.clone()), &expr);
    }

    let disabled_t2: Vec<_> = e_m
        .iter()
        .filter(|t| !e_m_prime.contains(*t) && ptpn.is_suspendable(*t))
        .cloned()
        .collect();

    for t in &disabled_t2 {
        let mut expr = LinExpr::new();
        expr.add_term(VarId::W(t.clone()), Q::from(1));
        expr.add_term(VarId::H(t.clone()), Q::from(-1));
        phi_prime.add_constraint(crate::constraints::LinearConstraint {
            expr,
            op: crate::constraints::ConstraintOp::Eq,
        });
    }

    let keep_vars: HashSet<VarId> = e_m_prime
        .iter()
        .flat_map(|t| {
            let mut s = HashSet::new();
            s.insert(VarId::H(t.clone()));
            if ptpn.is_suspendable(t) {
                s.insert(VarId::W(t.clone()));
            }
            s
        })
        .collect();

    phi_prime.project(&keep_vars);

    for t in &new_enabled {
        phi_prime.add_eq(VarId::H(t.clone()), Q::from(0));
        phi_prime.add_eq(VarId::W(t.clone()), Q::from(0));
    }

    for t in &pe {
        if ptpn.is_suspendable(t) && *t != t_f {
            phi_prime.add_eq(VarId::W(t.clone()), Q::from(0));
        }
    }

    StateClass {
        id: next_id,
        m: m_prime,
        phi: phi_prime,
    }
}

/// 状态类等价判定
pub fn classes_equivalent(a: &StateClass, b: &StateClass) -> bool {
    if a.m != b.m {
        return false;
    }
    let vars_a: HashSet<_> = a.phi.variables.iter().collect();
    let vars_b: HashSet<_> = b.phi.variables.iter().collect();
    if vars_a != vars_b {
        return false;
    }
    for v in &vars_a {
        let (la, ua) = a.phi.get_bounds(v);
        let (lb, ub) = b.phi.get_bounds(v);
        if la != lb || ua != ub {
            return false;
        }
    }
    true
}

/// Algorithm 3: 构建完整 SCG
pub fn build_scg(ptpn: &PTPN) -> SCG {
    let e_m0 = ptpn.enabled(&ptpn.m0);
    let mut phi0 = ConstraintSystem::new();
    for t in &e_m0 {
        phi0.add_eq(VarId::H(t.clone()), Q::from(0));
    }
    for t in &ptpn.t2 {
        phi0.add_eq(VarId::W(t.clone()), Q::from(0));
    }

    let c0 = StateClass {
        id: 0,
        m: ptpn.m0.clone(),
        phi: phi0,
    };

    let mut classes = vec![c0];
    let mut edges = Vec::new();
    let mut worklist = vec![0];
    let mut visited = vec![classes[0].clone()];

    while let Some(idx) = worklist.pop() {
        let class = classes[idx].clone();
        let enabled: Vec<_> = ptpn.enabled(&class.m).into_iter().collect();
        for t_f in enabled {
            if let Some(theta) = is_schedulable(ptpn, &class, t_f.as_str()) {
                let c_prime = successor(ptpn, &class, t_f.as_str(), &theta, classes.len());
                let dst_id = classes.len();

                let mut found = None;
                for (i, v) in visited.iter().enumerate() {
                    if classes_equivalent(&c_prime, v) {
                        found = Some(i);
                        break;
                    }
                }

                let dst = if let Some(i) = found {
                    i
                } else {
                    classes.push(c_prime.clone());
                    visited.push(c_prime.clone());
                    worklist.push(dst_id);
                    dst_id
                };

                edges.push((idx, t_f.clone(), dst));
            }
        }
    }

    SCG {
        classes,
        edges,
        initial: 0,
    }
}
