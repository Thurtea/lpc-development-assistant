use std::env;
use std::path::PathBuf;
use lpc_dev_assistant::ContextManager;

fn main() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    // If running from the lpc-dev-assistant subdirectory, treat parent as workspace root
    let workspace_root: PathBuf = if cwd.ends_with("lpc-dev-assistant") {
        cwd.parent().unwrap_or(&cwd).to_path_buf()
    } else {
        cwd
    };

    let ctx = ContextManager::new(workspace_root);
    ctx.ensure_templates_exist()?;
    let txt = ctx.load_simul_efun_context()?;
    println!("{}", txt);
    Ok(())
}
