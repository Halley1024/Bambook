mod json_annotation_repository;
mod json_document_history_repository;
mod json_settings_repository;
mod json_workspace_state_repository;
mod local_storage_repository;

pub(crate) use json_annotation_repository::JsonAnnotationRepository;
pub(crate) use json_document_history_repository::JsonDocumentHistoryRepository;
pub(crate) use json_settings_repository::JsonSettingsRepository;
pub(crate) use json_workspace_state_repository::JsonWorkspaceStateRepository;
pub(crate) use local_storage_repository::LocalStorageRepository;
