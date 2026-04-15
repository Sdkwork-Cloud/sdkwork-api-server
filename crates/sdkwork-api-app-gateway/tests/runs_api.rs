use sdkwork_api_app_gateway::{
    cancel_thread_run, create_thread_and_run, create_thread_run, get_thread_run,
    get_thread_run_step, list_thread_run_steps, list_thread_runs, submit_thread_run_tool_outputs,
    update_thread_run,
};

fn assert_error_contains<T: std::fmt::Debug, E: std::fmt::Display>(
    result: Result<T, E>,
    expected: &str,
) {
    let error = result.expect_err("expected error");
    assert!(
        error.to_string().contains(expected),
        "expected error containing `{expected}`, got `{error}`"
    );
}

#[test]
fn local_thread_run_creation_returns_placeholder_run() {
    let run = create_thread_run("tenant-1", "project-1", "thread_1", "asst_1", None)
        .expect("local fallback should synthesize a run");

    assert_eq!(run.id, "run_1");
    assert_eq!(run.object, "thread.run");
    assert_eq!(run.thread_id, "thread_1");
    assert_eq!(run.assistant_id.as_deref(), Some("asst_1"));
    assert_eq!(run.status, "queued");
    assert_eq!(run.model, "gpt-4.1");
}

#[test]
fn local_thread_run_queries_return_placeholder_state() {
    let list = list_thread_runs("tenant-1", "project-1", "thread_1")
        .expect("local fallback should list placeholder runs");
    assert_eq!(list.data.len(), 1);
    assert_eq!(list.data[0].id, "run_1");

    let retrieved = get_thread_run("tenant-1", "project-1", "thread_1", "run_1")
        .expect("local fallback should retrieve a placeholder run");
    assert_eq!(retrieved.id, "run_1");

    let updated = update_thread_run("tenant-1", "project-1", "thread_1", "run_1")
        .expect("local fallback should update a placeholder run");
    assert_eq!(updated.id, "run_1");

    let submitted = submit_thread_run_tool_outputs(
        "tenant-1",
        "project-1",
        "thread_1",
        "run_1",
        vec![("call_1", "{\"ok\":true}")],
    )
    .expect("local fallback should accept placeholder tool outputs");
    assert_eq!(submitted.id, "run_1");

    let cancelled = cancel_thread_run("tenant-1", "project-1", "thread_1", "run_1")
        .expect("local fallback should cancel a placeholder run");
    assert_eq!(cancelled.id, "run_1");
    assert_eq!(cancelled.status, "cancelled");
}

#[test]
fn local_thread_and_run_fallback_returns_placeholder_run() {
    let run = create_thread_and_run("tenant-1", "project-1", "asst_1", None)
        .expect("local fallback should synthesize a thread-and-run result");

    assert_eq!(run.id, "run_1");
    assert_eq!(run.thread_id, "thread_1");
    assert_eq!(run.assistant_id.as_deref(), Some("asst_1"));
}

#[test]
fn local_thread_run_step_fallback_returns_placeholder_state() {
    let list = list_thread_run_steps("tenant-1", "project-1", "thread_1", "run_1")
        .expect("local fallback should list placeholder run steps");
    assert_eq!(list.data.len(), 1);
    assert_eq!(list.data[0].id, "step_1");

    let step = get_thread_run_step("tenant-1", "project-1", "thread_1", "run_1", "step_1")
        .expect("local fallback should retrieve a placeholder run step");
    assert_eq!(step.id, "step_1");
    assert_eq!(step.run_id, "run_1");
}

#[test]
fn local_thread_run_fallback_returns_not_found_for_missing_ids() {
    assert_error_contains(
        create_thread_run("tenant-1", "project-1", "thread_missing", "asst_1", None),
        "thread not found",
    );
    assert_error_contains(
        list_thread_runs("tenant-1", "project-1", "thread_missing"),
        "thread not found",
    );
    assert_error_contains(
        get_thread_run("tenant-1", "project-1", "thread_1", "run_missing"),
        "run not found",
    );
    assert_error_contains(
        update_thread_run("tenant-1", "project-1", "thread_1", "run_missing"),
        "run not found",
    );
    assert_error_contains(
        cancel_thread_run("tenant-1", "project-1", "thread_1", "run_missing"),
        "run not found",
    );
    assert_error_contains(
        submit_thread_run_tool_outputs(
            "tenant-1",
            "project-1",
            "thread_1",
            "run_missing",
            vec![("call_1", "{\"ok\":true}")],
        ),
        "run not found",
    );
    assert_error_contains(
        list_thread_run_steps("tenant-1", "project-1", "thread_1", "run_missing"),
        "run not found",
    );
    assert_error_contains(
        get_thread_run_step("tenant-1", "project-1", "thread_1", "run_1", "step_missing"),
        "run step not found",
    );
}
