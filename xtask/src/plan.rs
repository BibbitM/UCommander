//! The `plan-status` task: checks the status in `docs/PLAN.md`.

const DONE: &str = "Done";
const PLANNED: &str = "Planned";

/// Returns the plan point ID of a `pt/<id>-short-name` branch, as it is in
/// `docs/PLAN.md`: `pt/p3-x` gives `P3`, and `pt/03-x` gives `3`.
fn plan_point_id(branch: &str) -> Option<String> {
    let id = branch.strip_prefix("pt/")?.split('-').next()?;
    Some(match id.strip_prefix('p') {
        Some(number) => format!("P{number}"),
        None => id.trim_start_matches('0').to_string(),
    })
}

/// Splits a table row of a plan point into its ID and its status.
fn plan_point_row(line: &str) -> Option<(&str, &str)> {
    let cells: Vec<&str> = line.trim().split('|').map(str::trim).collect();
    let id = *cells.get(1)?;
    let digits = id.strip_prefix('P').unwrap_or(id);
    let is_plan_point = !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit());
    let status = *cells.iter().rev().nth(1)?;
    is_plan_point.then_some((id, status))
}

/// Checks that the plan point of `branch` is `Done` in the text of
/// `docs/PLAN.md`. A branch that does not start with `pt/` passes.
pub fn check_plan_status(branch: &str, plan: &str) -> Result<(), String> {
    let Some(id) = plan_point_id(branch) else {
        return Ok(());
    };
    let status = plan
        .lines()
        .filter_map(plan_point_row)
        .find(|(row_id, _)| *row_id == id)
        .map(|(_, status)| status)
        .ok_or(format!("docs/PLAN.md has no plan point {id}"))?;
    if status == DONE {
        Ok(())
    } else {
        Err(format!(
            "plan point {id} has the status {status} in docs/PLAN.md. \
             Set it to {DONE} in the last commit of the branch."
        ))
    }
}

/// Returns the IDs of the plan points with a status that is not valid.
pub fn invalid_statuses(plan: &str) -> Vec<String> {
    plan.lines()
        .filter_map(plan_point_row)
        .filter(|(_, status)| *status != DONE && *status != PLANNED)
        .map(|(id, _)| id.to_string())
        .collect()
}
