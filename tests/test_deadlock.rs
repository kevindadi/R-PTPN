use ptpn::deadlock;
use ptpn::examples::three_task;
use ptpn::scg;

#[test]
fn test_no_global_deadlock() {
    let ptpn = three_task::build_three_task_ptpn();
    let scg = scg::build_scg(&ptpn);
    let _deadlocks = deadlock::detect_global_deadlocks(&ptpn, &scg);
}

#[test]
fn test_starvation_detection() {
    let ptpn = three_task::build_three_task_ptpn();
    let scg = scg::build_scg(&ptpn);
    let _starvations = deadlock::detect_starvation_sccs(&ptpn, &scg);
}
