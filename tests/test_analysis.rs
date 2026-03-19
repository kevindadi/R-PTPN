use ptpn::analysis;
use ptpn::examples::three_task;
use ptpn::rational::Q;
use ptpn::scg;
use std::collections::HashMap;

#[test]
fn test_three_task_wcet() {
    let ptpn = three_task::build_three_task_ptpn();
    let scg = scg::build_scg(&ptpn);
    let wcet1 = analysis::compute_wcet(&ptpn, &scg, 1);
    assert!(wcet1 >= Q::from(0));
}

#[test]
fn test_three_task_wcrt() {
    let ptpn = three_task::build_three_task_ptpn();
    let scg = scg::build_scg(&ptpn);
    let wcrt1 = analysis::compute_wcrt(&ptpn, &scg, 1);
    assert!(wcrt1 >= Q::from(0));
}

#[test]
fn test_schedulability_check() {
    let ptpn = three_task::build_three_task_ptpn();
    let scg = scg::build_scg(&ptpn);
    let mut deadlines = HashMap::new();
    deadlines.insert(1u32, Q::from(20));
    deadlines.insert(2, Q::from(20));
    deadlines.insert(3, Q::from(20));
    let sched = analysis::check_schedulability(&ptpn, &scg, &deadlines);
    assert!(sched.len() >= 1);
}
