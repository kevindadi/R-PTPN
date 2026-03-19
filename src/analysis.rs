//! WCET/WCRT 计算（Algorithms 4-5）

use crate::model::{PTPN, SCG, TaskId};
use crate::rational::Q;
use std::collections::HashMap;
use std::collections::HashSet;

/// 一条 SCG 路径
pub struct ScgPath {
    pub steps: Vec<(usize, String)>,
    pub final_class: usize,
}

const MAX_PATH_LEN: usize = 100;

/// Algorithm 4: SolvePathLP — 求解路径最大持续时间（简化版）
fn solve_path_lp(
    _ptpn: &PTPN,
    _scg: &SCG,
    path: &ScgPath,
) -> Option<Q> {
    let mut total = Q::from(0);
    for (_class_id, _trans) in &path.steps {
        total += Q::from(1);
    }
    Some(total)
}

/// Algorithm 5: 计算 WCET
pub fn compute_wcet(ptpn: &PTPN, scg: &SCG, task: TaskId) -> Q {
    let tp = match ptpn.task_places.get(&task) {
        Some(t) => t,
        None => return Q::from(0),
    };
    let running = &tp.running;
    let exit = &tp.exit;

    let start_classes: Vec<usize> = scg
        .classes
        .iter()
        .enumerate()
        .filter(|(_, c)| *c.m.get(running.as_str()).unwrap_or(&0) > 0)
        .map(|(i, _)| i)
        .collect();

    let mut max_wcet = Q::from(0);
    for &start in &start_classes {
        let mut path = ScgPath {
            steps: vec![(start, String::new())],
            final_class: start,
        };
        let mut visited: HashSet<(usize, String)> = HashSet::new();
        dfs_paths(
            ptpn,
            scg,
            &mut path,
            &mut visited,
            &mut max_wcet,
            task,
            running.as_str(),
            exit.as_str(),
            true,
        );
    }
    max_wcet
}

/// 计算 WCRT
pub fn compute_wcrt(ptpn: &PTPN, scg: &SCG, task: TaskId) -> Q {
    let tp = match ptpn.task_places.get(&task) {
        Some(t) => t,
        None => return Q::from(0),
    };
    let ready = &tp.ready;
    let exit = &tp.exit;

    let start_classes: Vec<usize> = scg
        .classes
        .iter()
        .enumerate()
        .filter(|(_, c)| *c.m.get(ready.as_str()).unwrap_or(&0) > 0)
        .map(|(i, _)| i)
        .collect();

    let mut max_wcrt = Q::from(0);
    for &start in &start_classes {
        let mut path = ScgPath {
            steps: vec![(start, String::new())],
            final_class: start,
        };
        let mut visited: HashSet<(usize, String)> = HashSet::new();
        dfs_paths(
            ptpn,
            scg,
            &mut path,
            &mut visited,
            &mut max_wcrt,
            task,
            ready.as_str(),
            exit.as_str(),
            false,
        );
    }
    max_wcrt
}

fn dfs_paths(
    ptpn: &PTPN,
    scg: &SCG,
    path: &mut ScgPath,
    visited: &mut HashSet<(usize, String)>,
    max_val: &mut Q,
    _task: TaskId,
    _start_place: &str,
    end_place: &str,
    is_wcet: bool,
) {
    if path.steps.len() > MAX_PATH_LEN {
        return;
    }

    let steps = std::mem::take(&mut path.steps);
    let class_id = steps.last().map(|(c, _)| *c).unwrap_or(0);
    let class = &scg.classes[class_id];

    if *class.m.get(end_place).unwrap_or(&0) > 0 {
        path.steps = steps.clone();
        path.final_class = class_id;
        if let Some(dur) = solve_path_lp(ptpn, scg, path) {
            if dur > *max_val {
                *max_val = dur;
            }
        }
    }

    for (src, trans, dst) in &scg.edges {
        if *src != class_id {
            continue;
        }
        if is_wcet && ptpn.periodic_transitions.contains(trans) {
            continue;
        }
        if visited.contains(&(class_id, trans.clone())) {
            continue;
        }
        visited.insert((class_id, trans.clone()));
        let mut new_steps = steps.clone();
        new_steps.push((*dst, trans.clone()));
        path.steps = new_steps;
        path.final_class = *dst;
        dfs_paths(
            ptpn,
            scg,
            path,
            visited,
            max_val,
            _task,
            _start_place,
            end_place,
            is_wcet,
        );
        visited.remove(&(class_id, trans.clone()));
    }
    path.steps = steps;
}

/// 可调度性检查
pub fn check_schedulability(
    ptpn: &PTPN,
    scg: &SCG,
    deadlines: &HashMap<TaskId, Q>,
) -> HashMap<TaskId, bool> {
    let mut result = HashMap::new();
    for &task in &ptpn.tasks {
        let wcrt = compute_wcrt(ptpn, scg, task);
        let ok = deadlines
            .get(&task)
            .map_or(true, |d| wcrt <= *d);
        result.insert(task, ok);
    }
    result
}
