mod commands;
mod spark_authorization;
mod state;
mod update;

pub use commands::{
    CommandError, ExecutedFileOutput, FileGenerationPlanOutput, FileGenerationReportOutput,
    FileGenerationRequest, FileTemplateEntry, PlannedFileOutput, ReviewedFileInput,
    RuntimeDiagnostics, SparkPreferences, SparkPreferencesError, execute_file_generation,
    list_file_templates, load_app_config, load_locale_override, load_spark_preferences,
    plan_file_generation, runtime_diagnostics, save_app_config, save_spark_preferences,
    save_user_template, set_file_generation_consent,
};
pub use spark_authorization::{SparkHostAuthorizer, set_spark_consent};
pub use state::DesktopState;
pub use update::{UpdateCheckResult, check_for_update};

pub fn run() -> Result<(), String> {
    let state = DesktopState::discover()
        .map_err(|error| format!("failed to resolve desktop data directory: {error}"))?;
    let spark_authorizer = SparkHostAuthorizer::new(&state.data_root);
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(bkmsa_tauri::init_with_authorizer(spark_authorizer))
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::file_generator::plan_file_generation_command,
            commands::file_generator::execute_file_generation_command,
            commands::file_generator::set_file_generation_consent_command,
            commands::file_generator::list_file_templates_command,
            commands::file_generator::save_user_template_command,
            commands::config::load_app_config_command,
            commands::config::save_app_config_command,
            commands::locale::load_locale_override_command,
            commands::diagnostics::runtime_diagnostics_command,
            spark_authorization::set_spark_consent_command,
            commands::spark_preferences::load_spark_preferences_command,
            commands::spark_preferences::save_spark_preferences_command,
            update::check_for_update_command,
        ])
        .run(tauri::generate_context!())
        .map_err(|error| format!("failed to run Bro Know My Toolbox: {error}"))
}
