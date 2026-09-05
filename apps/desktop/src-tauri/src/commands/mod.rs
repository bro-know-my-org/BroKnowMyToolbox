pub(crate) mod config;
pub(crate) mod diagnostics;
pub(crate) mod file_generator;
pub(crate) mod locale;
pub(crate) mod spark_preferences;

pub use config::{load_app_config, save_app_config};
pub use diagnostics::{RuntimeDiagnostics, runtime_diagnostics};

pub use file_generator::{
    CommandError, ExecutedFileOutput, FileGenerationPlanOutput, FileGenerationReportOutput,
    FileGenerationRequest, FileTemplateCatalog, FileTemplateEntry, PlannedFileOutput,
    ReviewedFileInput, execute_file_generation, list_file_templates, plan_file_generation,
    save_user_template, set_file_generation_consent,
};
pub use locale::load_locale_override;
pub use spark_preferences::{
    SparkPreferences, SparkPreferencesError, load_spark_preferences, save_spark_preferences,
};
