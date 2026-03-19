//! P-TPN 11元组、State、StateClass 数据结构

use crate::rational::Q;
use indexmap::IndexSet;
use std::collections::{BTreeMap, HashMap, HashSet};

/// 时间区间 [lo, hi]，hi 为 None 表示 +∞
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Interval {
    pub lo: Q,
    pub hi: Option<Q>,
}

impl Interval {
    pub fn new(lo: Q, hi: Option<Q>) -> Self {
        Self { lo, hi }
    }

    pub fn point(val: Q) -> Self {
        Self {
            lo: val.clone(),
            hi: Some(val),
        }
    }
}

/// 位标识符
pub type PlaceId = String;
/// 变迁标识符
pub type TransId = String;
/// 任务标识符
pub type TaskId = u32;

/// 弧的 key：(source, target)，用于区分 (place, trans) 和 (trans, place)
pub type ArcKey = (String, String);

/// 每个任务的关键位
#[derive(Clone, Debug)]
pub struct TaskPlaces {
    pub ready: PlaceId,
    pub running: PlaceId,
    pub exit: PlaceId,
}

/// P-TPN 11元组定义
#[derive(Clone, Debug)]
pub struct PTPN {
    pub p1: IndexSet<PlaceId>,
    pub p2: IndexSet<PlaceId>,
    pub t1: IndexSet<TransId>,
    pub t2: IndexSet<TransId>,
    pub f: HashMap<ArcKey, u32>,
    pub m0: BTreeMap<PlaceId, u32>,
    pub si: HashMap<TransId, Interval>,
    pub tasks: IndexSet<TaskId>,
    pub tak: HashMap<TransId, TaskId>,
    pub req: HashMap<TransId, HashSet<PlaceId>>,
    pub pri: HashMap<TransId, u32>,

    pub periodic_transitions: HashSet<TransId>,
    pub task_places: HashMap<TaskId, TaskPlaces>,
}

impl PTPN {
    pub fn new() -> Self {
        Self {
            p1: IndexSet::new(),
            p2: IndexSet::new(),
            t1: IndexSet::new(),
            t2: IndexSet::new(),
            f: HashMap::new(),
            m0: BTreeMap::new(),
            si: HashMap::new(),
            tasks: IndexSet::new(),
            tak: HashMap::new(),
            req: HashMap::new(),
            pri: HashMap::new(),
            periodic_transitions: HashSet::new(),
            task_places: HashMap::new(),
        }
    }

    /// 弧权重：F(from, to)
    pub fn arc(&self, from: &str, to: &str) -> u32 {
        *self.f.get(&(from.to_string(), to.to_string())).unwrap_or(&0)
    }
}

impl Default for PTPN {
    fn default() -> Self {
        Self::new()
    }
}

/// 具体状态
#[derive(Clone, Debug)]
pub struct State {
    pub m: BTreeMap<PlaceId, u32>,
    pub h: HashMap<TransId, Q>,
    pub w: HashMap<TransId, Q>,
}

/// 符号化状态类（ConstraintSystem 在 constraints 模块定义，此处前向引用）
#[derive(Clone, Debug)]
pub struct StateClass {
    pub id: usize,
    pub m: BTreeMap<PlaceId, u32>,
    pub phi: crate::constraints::ConstraintSystem,
}

/// 状态类图
pub struct SCG {
    pub classes: Vec<StateClass>,
    pub edges: Vec<(usize, TransId, usize)>,
    pub initial: usize,
}
