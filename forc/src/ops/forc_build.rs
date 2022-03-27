use crate::cli::BuildCommand;
use anyhow::{anyhow, bail, Result};
use forc_pkg::{self as pkg, lock, Lock, Manifest};
use forc_util::{default_output_directory, find_manifest_dir, lock_path};
use std::{
    fs::{self, File},
    path::PathBuf,
};
use sway_utils::MANIFEST_FILE_NAME;

pub fn build(command: BuildCommand) -> Result<pkg::Compiled> {
    let BuildCommand {
        path,
        binary_outfile,
        use_ir,
        debug_outfile,
        print_finalized_asm,
        print_intermediate_asm,
        print_ir,
        offline_mode: offline,
        silent_mode,
        output_directory,
        minify_json_abi,
    } = command;

    let config = pkg::BuildConfig {
        use_ir,
        print_ir,
        print_finalized_asm,
        print_intermediate_asm,
        silent: silent_mode,
    };

    // find manifest directory, even if in subdirectory
    let this_dir = if let Some(ref path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };

    let manifest_dir = match find_manifest_dir(&this_dir) {
        Some(dir) => dir,
        None => {
            bail!(
                "could not find `{}` in `{}` or any parent directory",
                MANIFEST_FILE_NAME,
                this_dir.display(),
            );
        }
    };
    let manifest = Manifest::from_dir(&manifest_dir)?;
    let lock_path = lock_path(&manifest_dir);

    // Load the build plan from the lock file.
    let plan_result = pkg::BuildPlan::from_lock_file(&lock_path);

    // Retrieve the old lock file state so we can produce a diff.
    let old_lock = plan_result
        .as_ref()
        .ok()
        .map(|plan| Lock::from_graph(plan.graph()))
        .unwrap_or_default();

    // Validate the loaded build plan for the current manifest.
    let plan_result = plan_result.and_then(|plan| plan.validate(&manifest).map(|_| plan));

    // If necessary, construct a new build plan.
    let plan: pkg::BuildPlan = plan_result.or_else(|e| -> Result<pkg::BuildPlan> {
        println!("  Creating a new `Forc.lock` file");
        println!("    Cause: {}", e);
        let plan = pkg::BuildPlan::new(&manifest_dir, offline)?;
        let lock = Lock::from_graph(plan.graph());
        let diff = lock.diff(&old_lock);
        lock::print_diff(&manifest.project.name, &diff);
        let string = toml::ser::to_string_pretty(&lock)
            .map_err(|e| anyhow!("failed to serialize lock file: {}", e))?;
        fs::write(&lock_path, &string).map_err(|e| anyhow!("failed to write lock file: {}", e))?;
        println!("   Created new lock file at {}", lock_path.display());
        Ok(plan)
    })?;

    // Build it!
    let (compiled, source_map) = pkg::build(&plan, &config)?;

    if let Some(outfile) = binary_outfile {
        fs::write(&outfile, &compiled.bytecode)?;
    }

    if let Some(outfile) = debug_outfile {
        let source_map_json = serde_json::to_vec(&source_map).expect("JSON serialization failed");
        fs::write(outfile, &source_map_json)?;
    }

    // TODO: We may support custom build profiles in the future.
    let profile = "debug";

    // Create the output directory for build artifacts.
    let output_dir = output_directory
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output_directory(&manifest_dir).join(profile));
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
    }

    // Place build artifacts into the output directory.
    let bin_path = output_dir
        .join(&manifest.project.name)
        .with_extension("bin");
    fs::write(&bin_path, &compiled.bytecode)?;
    if !compiled.json_abi.is_empty() {
        let json_abi_stem = format!("{}-abi", manifest.project.name);
        let json_abi_path = output_dir.join(&json_abi_stem).with_extension("json");
        let file = File::create(json_abi_path)?;
        let res = if minify_json_abi {
            serde_json::to_writer(&file, &compiled.json_abi)
        } else {
            serde_json::to_writer_pretty(&file, &compiled.json_abi)
        };
        res?;
    }

    println!("  Bytecode size is {} bytes.", compiled.bytecode.len());

    Ok(compiled)
}