use ptpn::examples::three_task;
use ptpn::scg;

#[test]
fn test_three_task_scg_size() {
    let ptpn = three_task::build_three_task_ptpn();
    let scg = scg::build_scg(&ptpn);
    assert!(scg.classes.len() >= 10);
}

#[test]
fn test_scg_determinism() {
    let ptpn = three_task::build_three_task_ptpn();
    let scg1 = scg::build_scg(&ptpn);
    let scg2 = scg::build_scg(&ptpn);
    assert_eq!(scg1.classes.len(), scg2.classes.len());
}
