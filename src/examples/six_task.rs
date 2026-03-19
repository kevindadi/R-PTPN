//! 论文图5：2处理器6任务示例（简化版）

use crate::model::{PTPN, TaskPlaces};
use std::collections::{BTreeMap, HashMap};

pub fn build_six_task_ptpn() -> PTPN {
    let mut ptpn = PTPN::new();

    ptpn.p2.insert("c0".to_string());
    ptpn.p2.insert("c1".to_string());

    let add_arc = |f: &mut HashMap<_, _>, from: &str, to: &str, w: u32| {
        f.insert((from.to_string(), to.to_string()), w);
    };

    add_arc(&mut ptpn.f, "c0", "t1", 1);
    add_arc(&mut ptpn.f, "c1", "t2", 1);

    let mut m0 = BTreeMap::new();
    m0.insert("c0".to_string(), 1);
    m0.insert("c1".to_string(), 1);
    ptpn.m0 = m0;

    ptpn.tasks.insert(1);
    ptpn.tasks.insert(2);
    ptpn.task_places.insert(
        1,
        TaskPlaces {
            ready: "r1".to_string(),
            running: "run1".to_string(),
            exit: "e1".to_string(),
        },
    );
    ptpn.task_places.insert(
        2,
        TaskPlaces {
            ready: "r2".to_string(),
            running: "run2".to_string(),
            exit: "e2".to_string(),
        },
    );

    ptpn
}
