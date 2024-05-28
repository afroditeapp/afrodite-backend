use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    EmitBuilder::builder()
        .fail_on_error()
        .cargo_target_triple()
        .cargo_debug()
        .cargo_features()
        .cargo_opt_level()
        .rustc_semver()
        .rustc_host_triple()
        .git_branch()
        .git_describe(true, true, None)
        .git_sha(false)
        .emit()?;

    Ok(())
}
