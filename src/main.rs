fn main() {
    let ptpn = ptpn::examples::three_task::build_three_task_ptpn();
    let scg = ptpn::scg::build_scg(&ptpn);
    println!("State classes: {}", scg.classes.len());
    println!("Edges: {}", scg.edges.len());

    for task in &ptpn.tasks {
        let wcet = ptpn::analysis::compute_wcet(&ptpn, &scg, *task);
        let wcrt = ptpn::analysis::compute_wcrt(&ptpn, &scg, *task);
        println!("Task {}: WCET={}, WCRT={}", task, wcet, wcrt);
    }

    let deadlocks = ptpn::deadlock::detect_global_deadlocks(&ptpn, &scg);
    let starvations = ptpn::deadlock::detect_starvation_sccs(&ptpn, &scg);
    println!("Deadlocks: {:?}", deadlocks);
    println!("Starvations: {} SCCs", starvations.len());
}
