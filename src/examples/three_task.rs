//! 论文图2：2处理器3任务示例

use crate::model::{Interval, TaskPlaces, PTPN};
use crate::rational::Q;
use std::collections::{BTreeMap, HashSet};

pub fn build_three_task_ptpn() -> PTPN {
    let mut ptpn = PTPN::new();

    ptpn.p1.insert("p1".to_string());
    ptpn.p1.insert("p2".to_string());
    ptpn.p1.insert("p3".to_string());
    ptpn.p1.insert("p4".to_string());
    ptpn.p1.insert("p5".to_string());
    ptpn.p1.insert("p6".to_string());
    ptpn.p1.insert("p_period_a".to_string());
    ptpn.p1.insert("p_period_b".to_string());
    ptpn.p1.insert("p_period_c".to_string());
    ptpn.p2.insert("cpu".to_string());

    ptpn.t1.insert("t1".to_string());
    ptpn.t1.insert("t3".to_string());
    ptpn.t1.insert("t4".to_string());
    ptpn.t1.insert("t6".to_string());
    ptpn.t1.insert("t9".to_string());
    ptpn.t1.insert("t11".to_string());
    ptpn.t2.insert("t2".to_string());
    ptpn.t2.insert("t5".to_string());
    ptpn.t2.insert("t10".to_string());

    let add_arc = |f: &mut std::collections::HashMap<_, _>, from: &str, to: &str, w: u32| {
        f.insert((from.to_string(), to.to_string()), w);
    };
    add_arc(&mut ptpn.f, "p1", "t1", 1);
    add_arc(&mut ptpn.f, "cpu", "t1", 1);
    add_arc(&mut ptpn.f, "t1", "p2", 1);
    add_arc(&mut ptpn.f, "p2", "t2", 1);
    add_arc(&mut ptpn.f, "cpu", "t2", 1);
    add_arc(&mut ptpn.f, "t2", "p3", 1);
    add_arc(&mut ptpn.f, "p3", "t3", 1);
    add_arc(&mut ptpn.f, "t3", "p_period_a", 1);
    add_arc(&mut ptpn.f, "p_period_a", "t1", 1);

    add_arc(&mut ptpn.f, "p4", "t4", 1);
    add_arc(&mut ptpn.f, "cpu", "t4", 1);
    add_arc(&mut ptpn.f, "t4", "p5", 1);
    add_arc(&mut ptpn.f, "p5", "t5", 1);
    add_arc(&mut ptpn.f, "cpu", "t5", 1);
    add_arc(&mut ptpn.f, "t5", "p6", 1);
    add_arc(&mut ptpn.f, "p6", "t6", 1);
    add_arc(&mut ptpn.f, "t6", "p_period_b", 1);
    add_arc(&mut ptpn.f, "p_period_b", "t4", 1);

    add_arc(&mut ptpn.f, "p_period_c", "t9", 1);
    add_arc(&mut ptpn.f, "cpu", "t9", 1);
    add_arc(&mut ptpn.f, "t9", "p2", 1);
    add_arc(&mut ptpn.f, "p2", "t10", 1);
    add_arc(&mut ptpn.f, "cpu", "t10", 1);
    add_arc(&mut ptpn.f, "t10", "p2", 1);
    add_arc(&mut ptpn.f, "p2", "t11", 1);
    add_arc(&mut ptpn.f, "t11", "p_period_c", 1);

    let mut m0 = BTreeMap::new();
    m0.insert("p1".to_string(), 1);
    m0.insert("p4".to_string(), 1);
    m0.insert("p_period_a".to_string(), 1);
    m0.insert("p_period_b".to_string(), 1);
    m0.insert("p_period_c".to_string(), 1);
    m0.insert("cpu".to_string(), 2);
    ptpn.m0 = m0;

    ptpn.si
        .insert("t1".to_string(), Interval::point(Q::from(0)));
    ptpn.si.insert(
        "t2".to_string(),
        Interval::new(Q::from(9), Some(Q::from(11))),
    );
    ptpn.si
        .insert("t3".to_string(), Interval::point(Q::from(0)));
    ptpn.si
        .insert("t4".to_string(), Interval::point(Q::from(0)));
    ptpn.si
        .insert("t5".to_string(), Interval::point(Q::from(5)));
    ptpn.si
        .insert("t6".to_string(), Interval::point(Q::from(0)));
    ptpn.si.insert(
        "t9".to_string(),
        Interval::new(Q::from(3), Some(Q::from(30))),
    );
    ptpn.si
        .insert("t10".to_string(), Interval::point(Q::from(0)));
    ptpn.si
        .insert("t11".to_string(), Interval::point(Q::from(0)));

    ptpn.tasks.insert(1);
    ptpn.tasks.insert(2);
    ptpn.tasks.insert(3);

    ptpn.tak.insert("t1".to_string(), 1);
    ptpn.tak.insert("t2".to_string(), 1);
    ptpn.tak.insert("t3".to_string(), 1);
    ptpn.tak.insert("t4".to_string(), 2);
    ptpn.tak.insert("t5".to_string(), 2);
    ptpn.tak.insert("t6".to_string(), 2);
    ptpn.tak.insert("t9".to_string(), 3);
    ptpn.tak.insert("t10".to_string(), 3);
    ptpn.tak.insert("t11".to_string(), 3);

    let mut req_t1 = HashSet::new();
    req_t1.insert("cpu".to_string());
    ptpn.req.insert("t1".to_string(), req_t1);
    let mut req_t2 = HashSet::new();
    req_t2.insert("cpu".to_string());
    ptpn.req.insert("t2".to_string(), req_t2);
    let mut req_t4 = HashSet::new();
    req_t4.insert("cpu".to_string());
    ptpn.req.insert("t4".to_string(), req_t4);
    let mut req_t5 = HashSet::new();
    req_t5.insert("cpu".to_string());
    ptpn.req.insert("t5".to_string(), req_t5);
    let mut req_t9 = HashSet::new();
    req_t9.insert("cpu".to_string());
    ptpn.req.insert("t9".to_string(), req_t9);
    let mut req_t10 = HashSet::new();
    req_t10.insert("cpu".to_string());
    ptpn.req.insert("t10".to_string(), req_t10);

    ptpn.pri.insert("t1".to_string(), 1);
    ptpn.pri.insert("t2".to_string(), 1);
    ptpn.pri.insert("t3".to_string(), 1);
    ptpn.pri.insert("t4".to_string(), 2);
    ptpn.pri.insert("t5".to_string(), 2);
    ptpn.pri.insert("t6".to_string(), 2);
    ptpn.pri.insert("t9".to_string(), 3);
    ptpn.pri.insert("t10".to_string(), 3);
    ptpn.pri.insert("t11".to_string(), 3);

    ptpn.periodic_transitions.insert("t1".to_string());
    ptpn.periodic_transitions.insert("t4".to_string());

    ptpn.task_places.insert(
        1,
        TaskPlaces {
            ready: "p1".to_string(),
            running: "p2".to_string(),
            exit: "p3".to_string(),
        },
    );
    ptpn.task_places.insert(
        2,
        TaskPlaces {
            ready: "p4".to_string(),
            running: "p5".to_string(),
            exit: "p6".to_string(),
        },
    );
    ptpn.task_places.insert(
        3,
        TaskPlaces {
            ready: "p_period_c".to_string(),
            running: "p2".to_string(),
            exit: "p_period_c".to_string(),
        },
    );

    ptpn
}
