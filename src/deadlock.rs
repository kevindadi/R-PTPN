//! 全局死锁 + 饥饿 SCC 检测（Algorithm 6）

use crate::model::{PTPN, SCG, TaskId};
use petgraph::algo::tarjan_scc;
use petgraph::graph::DiGraph;
use std::collections::{HashMap, HashSet};

/// 全局死锁检测：E(M) = ∅ 的状态类
pub fn detect_global_deadlocks(ptpn: &PTPN, scg: &SCG) -> Vec<usize> {
    scg.classes
        .iter()
        .enumerate()
        .filter(|(_, c)| ptpn.enabled(&c.m).is_empty())
        .map(|(i, _)| i)
        .collect()
}

/// 饥饿检测结果
pub struct StarvationResult {
    pub scc_classes: Vec<usize>,
    pub pending_tasks: HashSet<TaskId>,
}

/// Algorithm 6: 饥饿 SCC 检测
pub fn detect_starvation_sccs(ptpn: &PTPN, scg: &SCG) -> Vec<StarvationResult> {
    let t_p = &ptpn.periodic_transitions;

    let mut graph = DiGraph::new();
    let mut node_map: HashMap<usize, _> = HashMap::new();
    for (i, _) in scg.classes.iter().enumerate() {
        let n = graph.add_node(i);
        node_map.insert(i, n);
    }

    for (src, trans, dst) in &scg.edges {
        if t_p.contains(trans) {
            if let (Some(&sn), Some(&dn)) = (node_map.get(src), node_map.get(dst)) {
                graph.add_edge(sn, dn, ());
            }
        }
    }

    let sccs = tarjan_scc(&graph);
    let mut results = Vec::new();

    for scc in sccs {
        if scc.len() <= 1 {
            continue;
        }

        let scc_ids: HashSet<usize> = scc
            .iter()
            .map(|n| graph[*n])
            .collect();

        let mut pending = HashSet::new();
        for (task_id, places) in &ptpn.task_places {
            for c_id in &scc_ids {
                let class = &scg.classes[*c_id];
                if *class.m.get(places.ready.as_str()).unwrap_or(&0) > 0
                    || *class.m.get(places.running.as_str()).unwrap_or(&0) > 0
                {
                    pending.insert(*task_id);
                    break;
                }
            }
        }

        if pending.is_empty() {
            continue;
        }

        let mut only_periodic = true;
        for &c_id in &scc_ids {
            for (src, trans, _) in &scg.edges {
                if *src == c_id && !t_p.contains(trans) {
                    only_periodic = false;
                    break;
                }
            }
        }

        if only_periodic {
            results.push(StarvationResult {
                scc_classes: scc_ids.into_iter().collect(),
                pending_tasks: pending,
            });
        }
    }

    results
}
