use xtask::plan::{check_plan_status, invalid_statuses};

const PLAN: &str = "\
| Pt | Name | Content | Release | Status |
|---|---|---|---|---|
| P2 | Skeleton | Text. | — | Done |
| P3 | Package | Text. | — | Planned |

| Pt | Name | IDs | Release | Status |
|---|---|---|---|---|
| 3 | Asset class | D1 | v0.2.0 | Done |
| 4a | Technical debt | — | | Done |
| 5b | Other fixes | — | | Planned |
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
fn a_branch_with_a_letter_finds_its_row() {
    assert_eq!(check_plan_status("pt/04a-tech-debt", PLAN), Ok(()));
    assert_eq!(check_plan_status("pt/4a-tech-debt", PLAN), Ok(()));
}

#[test]
fn a_branch_with_a_letter_and_status_planned_fails() {
    let error = check_plan_status("pt/05b-other-fixes", PLAN).unwrap_err();

    assert!(
        error.contains("plan point 5b has the status Planned"),
        "{error}"
    );
}

#[test]
fn a_number_branch_does_not_match_a_row_with_a_letter() {
    let error = check_plan_status("pt/04-name", PLAN).unwrap_err();

    assert!(error.contains("no plan point 4"));
}

#[test]
fn a_row_with_a_letter_and_a_status_that_is_not_valid_is_reported() {
    assert_eq!(
        invalid_statuses("| 4a | Debt | — | | In review |\n"),
        ["4a"]
    );
}

#[test]
fn a_point_and_its_letter_point_find_their_own_rows() {
    let plan = "| 4 | Name | D2 | | Planned |\n| 4a | Debt | — | | Done |\n";

    assert!(check_plan_status("pt/04-name", plan).is_err());
    assert_eq!(check_plan_status("pt/04a-debt", plan), Ok(()));
}

#[test]
fn a_branch_with_two_letters_or_a_letter_after_p_has_no_row() {
    let plan = "| 4a | Debt | — | | Done |\n| P3 | Package | Text. | — | Done |\n";

    for branch in ["pt/04ab-x", "pt/p3a-x"] {
        let error = check_plan_status(branch, plan).unwrap_err();

        assert!(error.contains("has no plan point"), "{branch}: {error}");
    }
}

#[test]
fn a_row_with_two_letters_or_an_upper_case_letter_is_not_a_plan_point() {
    assert!(invalid_statuses("| 4ab | Debt | — | | In review |\n").is_empty());
    assert!(invalid_statuses("| 4A | Debt | — | | In review |\n").is_empty());
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
