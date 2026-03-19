//! 启用集、优先级过滤、变迁触发规则

use crate::model::{PTPN, PlaceId, State, TransId};
use crate::rational::Q;
use std::collections::{BTreeMap, HashMap, HashSet};

impl PTPN {
    /// 所有位（P1 ∪ P2）
    pub fn all_places(&self) -> impl Iterator<Item = &PlaceId> {
        self.p1.iter().chain(self.p2.iter())
    }

    /// 所有变迁（T1 ∪ T2）
    pub fn all_transitions(&self) -> impl Iterator<Item = &TransId> {
        self.t1.iter().chain(self.t2.iter())
    }

    /// 变迁 t 的前置位集合 •t
    pub fn pre_set(&self, t: &TransId) -> Vec<PlaceId> {
        self.all_places()
            .filter(|p| self.arc(p, t) > 0)
            .cloned()
            .collect()
    }

    /// 变迁 t 的后置位集合 t•
    pub fn post_set(&self, t: &TransId) -> Vec<PlaceId> {
        self.all_places()
            .filter(|p| self.arc(t, p) > 0)
            .cloned()
            .collect()
    }

    /// 位 p 到变迁 t 的弧权重，无弧则返回 0
    pub fn arc_weight(&self, from: &str, to: &str) -> u32 {
        self.arc(from, to)
    }

    /// 标识 M 下的启用变迁集 E(M)
    pub fn enabled(&self, m: &BTreeMap<PlaceId, u32>) -> HashSet<TransId> {
        let mut result = HashSet::new();
        for t in self.all_transitions() {
            let mut can_fire = true;
            for p in &self.pre_set(t) {
                let need = self.arc(&p, t);
                let have = *m.get(p.as_str()).unwrap_or(&0);
                if have < need {
                    can_fire = false;
                    break;
                }
            }
            if can_fire {
                result.insert(t.clone());
            }
        }
        result
    }

    /// 变迁 t 对资源位 r 的需求量
    pub fn demand(&self, t: &TransId, r: &PlaceId) -> u32 {
        if self.pre_set(t).contains(r) {
            self.arc(r, t)
        } else {
            0
        }
    }

    /// 触发变迁 t，返回新标识 M'
    pub fn fire_marking(
        &self,
        m: &BTreeMap<PlaceId, u32>,
        t: &TransId,
    ) -> BTreeMap<PlaceId, u32> {
        let mut m_prime = m.clone();
        for p in &self.pre_set(t) {
            let w = self.arc(p, t);
            *m_prime.get_mut(p.as_str()).unwrap() -= w;
        }
        for p in &self.post_set(t) {
            *m_prime.entry(p.clone()).or_insert(0) += self.arc(t, p);
        }
        m_prime
    }

    /// 新启用集 New(M, M', t_f)
    pub fn newly_enabled(
        &self,
        m: &BTreeMap<PlaceId, u32>,
        m_prime: &BTreeMap<PlaceId, u32>,
        t_f: &TransId,
    ) -> HashSet<TransId> {
        let e_m = self.enabled(m);
        let e_m_prime = self.enabled(m_prime);

        let mut result = HashSet::new();
        for t in &e_m_prime {
            if *t == *t_f {
                result.insert(t.clone());
                continue;
            }
            if !e_m.contains(t) {
                result.insert(t.clone());
                continue;
            }
            for p in &self.pre_set(t) {
                let had = *m.get(p.as_str()).unwrap_or(&0);
                let have = *m_prime.get(p.as_str()).unwrap_or(&0);
                if had == 0 && have > 0 {
                    result.insert(t.clone());
                    break;
                }
            }
        }
        result
    }

    /// 判断变迁是否可挂起
    pub fn is_suspendable(&self, t: &TransId) -> bool {
        self.t2.contains(t)
    }
}

/// 时间就绪集 F(S, τ)
pub fn time_ready_set(ptpn: &PTPN, state: &State, tau: Q) -> HashSet<TransId> {
    let e_m = ptpn.enabled(&state.m);
    let mut result = HashSet::new();
    for t in &e_m {
        if let Some(si) = ptpn.si.get(t) {
            let h = state.h.get(t).cloned().unwrap_or(Q::from(0));
            let val = h + tau.clone();
            if val >= si.lo.clone() {
                if let Some(ref hi) = si.hi {
                    if val <= hi.clone() {
                        result.insert(t.clone());
                    }
                } else {
                    result.insert(t.clone());
                }
            }
        }
    }
    result
}

/// 请求就绪集 F^req_r(S, τ)
pub fn request_ready_set(
    ptpn: &PTPN,
    state: &State,
    tau: Q,
    r: &PlaceId,
) -> HashSet<TransId> {
    time_ready_set(ptpn, state, tau)
        .into_iter()
        .filter(|t| {
            ptpn.req.get(t).map_or(false, |req| req.contains(r))
                && ptpn.demand(t, r) > 0
        })
        .collect()
}

/// 计算 Grant_r(S, τ)
pub fn grant_set(
    ptpn: &PTPN,
    state: &State,
    tau: Q,
    r: &PlaceId,
) -> HashSet<TransId> {
    let f_req = request_ready_set(ptpn, state, tau.clone(), r);
    let m_r = *state.m.get(r.as_str()).unwrap_or(&0) as usize;

    let mut by_prio: Vec<_> = f_req.into_iter().collect();
    by_prio.sort_by(|a, b| {
        let pa = ptpn.pri.get(a.as_str()).copied().unwrap_or(0);
        let pb = ptpn.pri.get(b.as_str()).copied().unwrap_or(0);
        pb.cmp(&pa)
    });

    let mut result = HashSet::new();
    for (rank, t) in by_prio.into_iter().enumerate() {
        if rank < m_r {
            result.insert(t);
        }
    }
    result
}

/// F^sus_r: 可挂起且需求 r 为 0 的变迁
fn f_sus_r(ptpn: &PTPN, state: &State, tau: Q, r: &PlaceId) -> HashSet<TransId> {
    time_ready_set(ptpn, state, tau)
        .into_iter()
        .filter(|t| {
            ptpn.is_suspendable(t)
                && ptpn.req.get(t.as_str()).map_or(false, |req| req.contains(r.as_str()))
                && ptpn.demand(t, r) == 0
        })
        .collect()
}

/// 计算 Vict_r(S, τ) — 挂起受害者集
pub fn victim_set(
    ptpn: &PTPN,
    state: &State,
    tau: Q,
    r: &PlaceId,
) -> HashSet<TransId> {
    let f_sus = f_sus_r(ptpn, state, tau, r);
    if f_sus.is_empty() {
        return HashSet::new();
    }
    let min_prio = f_sus
        .iter()
        .map(|t| ptpn.pri.get(t.as_str()).copied().unwrap_or(0))
        .min()
        .unwrap_or(0);
    f_sus
        .into_iter()
        .filter(|t| ptpn.pri.get(t.as_str()).copied().unwrap_or(0) == min_prio)
        .collect()
}

/// Filter_r(S, τ)
pub fn filter_r(
    ptpn: &PTPN,
    state: &State,
    tau: Q,
    r: &PlaceId,
) -> HashSet<TransId> {
    let g = grant_set(ptpn, state, tau.clone(), r);
    if !g.is_empty() {
        g
    } else {
        victim_set(ptpn, state, tau, r)
    }
}

/// 判断变迁 t 在 (S, τ) 下是否优先级可接受
pub fn is_priority_admissible(
    ptpn: &PTPN,
    state: &State,
    tau: Q,
    t: &TransId,
) -> bool {
    if !time_ready_set(ptpn, state, tau.clone()).contains(t) {
        return false;
    }
    if let Some(req) = ptpn.req.get(t.as_str()) {
        for r in req {
            if !filter_r(ptpn, state, tau.clone(), r).contains(t) {
                return false;
            }
        }
    }
    true
}

/// 完整的状态变迁：延迟 τ 后触发 t
pub fn fire_state(ptpn: &PTPN, state: &State, t: &TransId, tau: Q) -> State {
    let e_m = ptpn.enabled(&state.m);
    let m_prime = ptpn.fire_marking(&state.m, t);
    let new_enabled = ptpn.newly_enabled(&state.m, &m_prime, t);
    let pe: HashSet<_> = ptpn
        .enabled(&m_prime)
        .difference(&new_enabled)
        .cloned()
        .collect();

    let mut h_mid: HashMap<TransId, Q> = HashMap::new();
    for t_enabled in &e_m {
        let h = state.h.get(t_enabled).cloned().unwrap_or(Q::from(0));
        h_mid.insert(t_enabled.clone(), h + tau.clone());
    }

    let mut h_prime = HashMap::new();
    let mut w_prime = HashMap::new();

    for t_enabled in &e_m {
        if t_enabled == t {
            h_prime.insert(t_enabled.clone(), Q::from(0));
            w_prime.insert(t_enabled.clone(), Q::from(0));
        } else if pe.contains(t_enabled) {
            h_prime.insert(
                t_enabled.clone(),
                h_mid.get(t_enabled).cloned().unwrap_or(Q::from(0)),
            );
            w_prime.insert(
                t_enabled.clone(),
                state.w.get(t_enabled).cloned().unwrap_or(Q::from(0)),
            );
        } else {
            if ptpn.is_suspendable(t_enabled) {
                w_prime.insert(
                    t_enabled.clone(),
                    h_mid.get(t_enabled).cloned().unwrap_or(Q::from(0)),
                );
                h_prime.insert(t_enabled.clone(), Q::from(0));
            } else {
                w_prime.insert(t_enabled.clone(), Q::from(0));
                h_prime.insert(t_enabled.clone(), Q::from(0));
            }
        }
    }

    for t_new in &new_enabled {
        if ptpn.is_suspendable(t_new) {
            h_prime.insert(
                t_new.clone(),
                state.w.get(t_new).cloned().unwrap_or(Q::from(0)),
            );
            w_prime.insert(t_new.clone(), Q::from(0));
        } else {
            h_prime.insert(t_new.clone(), Q::from(0));
            w_prime.insert(t_new.clone(), Q::from(0));
        }
    }

    for t_pe in &pe {
        if ptpn.is_suspendable(t_pe) && *t_pe != *t {
            w_prime.insert(t_pe.clone(), Q::from(0));
        }
    }

    State {
        m: m_prime,
        h: h_prime,
        w: w_prime,
    }
}
