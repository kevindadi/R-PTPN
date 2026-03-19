use ptpn::examples::three_task;

#[test]
fn test_enabled_transitions() {
    let ptpn = three_task::build_three_task_ptpn();
    let e = ptpn.enabled(&ptpn.m0);
    assert!(!e.is_empty());
}

#[test]
fn test_fire_marking() {
    let ptpn = three_task::build_three_task_ptpn();
    let t1 = "t1".to_string();
    let m = ptpn.fire_marking(&ptpn.m0, &t1);
    assert!(m.get("p2").copied().unwrap_or(0) >= 1);
}

#[test]
fn test_newly_enabled() {
    let ptpn = three_task::build_three_task_ptpn();
    let m = ptpn.m0.clone();
    let t1 = "t1".to_string();
    let m_prime = ptpn.fire_marking(&m, &t1);
    let new_ = ptpn.newly_enabled(&m, &m_prime, &t1);
    assert!(new_.contains("t2"));
}
