use vergen_gitcl::{CargoBuilder, Emitter, GitclBuilder, RustcBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Emitter::default()
        .add_instructions(
            &CargoBuilder::default()
                .target_triple(true)
                .debug(true)
                .features(true)
                .opt_level(true)
                .build()?
        )?
        .add_instructions(
            &RustcBuilder::default()
                .semver(true)
                .host_triple(true)
                .build()?
        )?
        .add_instructions(
            &GitclBuilder::default()
                .branch(true)
                .describe(true, true, None)
                .sha(false)
                .build()?
        )?
        .fail_on_error()
        .emit()?;

    Ok(())
}
