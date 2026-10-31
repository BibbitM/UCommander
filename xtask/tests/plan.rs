use xtask::plan::{check_plan_status, invalid_statuses};

const PLAN: &str = "\
| Pt | Name | Content | Release | Status |
|---|---|---|---|---|
| P2 | Skeleton | Text. | — | Done |
| P3 | Package | Text. | — | Planned |

| Pt | Name | IDs | Release | Status |
|---|---|---|---|---|
| 3 | Asset class | D1 | v0.2.0 | Done |
| 13 | External actors | B7 | v0.6.0 | Planned |
";

#[test]
fn a_phase_0_branch_with_status_done_passes() {
    assert_eq!(check_plan_status("pt/p2-content-skeleton", PLAN), Ok(()));
}

#[test]
fn a_phase_0_branch_with_status_planned_fails() {
    let error = check_plan_status("pt/p3-package-install", PLAN).unwrap_err();

    assert!(error.contains("P3"));
    assert!(error.contains("Done"));
}

#[test]
fn a_number_branch_with_a_leading_zero_finds_its_row() {
    assert_eq!(check_plan_status("pt/03-content-asset-class", PLAN), Ok(()));
}

#[test]
fn a_number_branch_does_not_match_a_longer_number() {
    assert!(check_plan_status("pt/13-external-actors", PLAN).is_err());
    assert!(check_plan_status("pt/01-summary", PLAN).is_err());
}

#[test]
fn a_plan_point_branch_without_a_row_fails() {
    let error = check_plan_status("pt/99-unknown", PLAN).unwrap_err();

    assert!(error.contains("99"));
}

#[test]
fn other_branches_pass() {
    assert_eq!(
        check_plan_status("dependabot/github_actions/x", PLAN),
        Ok(())
    );
    assert_eq!(check_plan_status("ci-test/p1-fail", PLAN), Ok(()));
}

#[test]
fn a_status_that_is_not_planned_or_done_is_reported() {
    let plan = "| 3 | Asset class | D1 | v0.2.0 | In review |\n| P1 | CI | Text. | — | Done |\n";

    assert_eq!(invalid_statuses(plan), ["3"]);
}

#[test]
fn the_decision_table_is_not_a_plan_point() {
    assert!(invalid_statuses("| Language | Rust. |\n").is_empty());
}

#[test]
fn the_real_plan_has_only_valid_statuses() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let plan = std::fs::read_to_string(root.join("docs").join("PLAN.md")).unwrap();

    assert_eq!(invalid_statuses(&plan), Vec::<String>::new());
}
