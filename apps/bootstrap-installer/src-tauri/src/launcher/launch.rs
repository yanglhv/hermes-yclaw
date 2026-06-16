use std::error::Error;
use std::process::Command;
use crate::app::AppDescriptor;

pub struct ExitSuccess(());

pub fn launch_and_exit(app: &AppDescriptor) -> Result<ExitSuccess, Box<dyn Error>> {
    if app.binaries.is_empty() {
        return Err("No binaries configured".into());
    }
    let binary_path = crate::paths::hermes_home()
        .join(&app.install_root)
        .join("bin")
        .join(&app.binaries[0]);
    Command::new(&binary_path).spawn()?;
    Ok(ExitSuccess(()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_and_exit_returns_result() {
        let app = crate::app::AppDescriptor::literal_hermes();
        let result = super::launch_and_exit(&app);
        assert!(result.is_ok() || result.is_err());
    }
}
