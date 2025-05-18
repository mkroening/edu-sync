use edu_sync::config::Config;

pub fn check_accounts(config: &Config) -> bool {
    let sucess = config.has_accounts();
    if !sucess {
        eprintln!("No accounts configured. To add an account, use the add subcommand.");
    }
    sucess
}

pub fn check_active_courses(config: &Config) -> bool {
    if !check_accounts(config) {
        false
    } else if !config.has_courses() {
        eprintln!("No courses known. To fetch available courses, use the fetch subcommand.");
        false
    } else if !config.has_active_courses() {
        eprintln!(
            "No courses activated. To activate synchronization for courses, edit the config at\n{}",
            Config::path().display()
        );
        false
    } else {
        true
    }
}
