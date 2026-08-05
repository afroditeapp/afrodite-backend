use vergen_gitcl::{Cargo, Emitter, Gitcl, Rustc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Emitter::default()
        .add_instructions(
            &Cargo::builder()
                .target_triple(true)
                .debug(true)
                .features(true)
                .opt_level(true)
                .build(),
        )?
        .add_instructions(&Rustc::builder().semver(true).host_triple(true).build())?
        .add_instructions(
            &Gitcl::builder()
                .branch(true)
                .describe(true, true, None)
                .sha(false)
                .build(),
        )?
        .fail_on_error()
        .emit()?;

    Ok(())
}
