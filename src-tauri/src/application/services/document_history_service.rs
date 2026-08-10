use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    application::ports::{DocumentHistoryRepository, DocumentPathInspector},
    domain::{
        document::DocumentKind,
        document_history::{DocumentAvailability, DocumentHistory, DocumentHistoryEntry},
        storage::StoredDocumentEntry,
    },
    error::{AppError, AppResult},
};

const MAXIMUM_HISTORY_ENTRIES: usize = 12;

pub(crate) struct DocumentHistoryService {
    repository: Arc<dyn DocumentHistoryRepository>,
    path_inspector: Arc<dyn DocumentPathInspector>,
    operation_lock: Mutex<()>,
}

impl DocumentHistoryService {
    pub(crate) fn new(
        repository: Arc<dyn DocumentHistoryRepository>,
        path_inspector: Arc<dyn DocumentPathInspector>,
    ) -> Self {
        Self {
            repository,
            path_inspector,
            operation_lock: Mutex::new(()),
        }
    }

    pub(crate) fn load_with_library(&self, library: &[StoredDocumentEntry]) -> AppResult<DocumentHistory> {
        let _guard = self.lock()?;
        let mut recent = self.repository.load_recent()?;
        let mut closed = self.repository.load_closed()?;
        attach_document_ids(&mut recent, library);
        attach_document_ids(&mut closed, library);
        recent = normalize(recent);
        closed = normalize(closed);
        self.repository.save_recent(&recent)?;
        self.repository.save_closed(&closed)?;
        Ok(self.refresh_availability(DocumentHistory { recent, closed }))
    }

    fn load_unlocked(&self) -> AppResult<DocumentHistory> {
        Ok(DocumentHistory {
            recent: normalize(self.repository.load_recent()?),
            closed: normalize(self.repository.load_closed()?),
        })
    }

    pub(crate) fn record_closed(
        &self,
        document_id: Option<String>,
        path: String,
        title: String,
        kind: DocumentKind,
    ) -> AppResult<DocumentHistory> {
        let _guard = self.lock()?;
        let entry = DocumentHistoryEntry {
            document_id,
            path,
            title,
            kind,
            last_closed_at: now_millis(),
            availability: DocumentAvailability::Unknown,
        };
        let mut history = self.load_unlocked()?;
        history.recent = insert_at_front(history.recent, entry.clone());
        history.closed = insert_at_front(history.closed, entry);
        self.repository.save_recent(&history.recent)?;
        self.repository.save_closed(&history.closed)?;
        Ok(self.refresh_availability(history))
    }

    pub(crate) fn record_accessed(
        &self,
        document_id: Option<String>,
        path: String,
        title: String,
        kind: DocumentKind,
    ) -> AppResult<DocumentHistory> {
        let _guard = self.lock()?;
        let entry = DocumentHistoryEntry {
            document_id,
            path,
            title,
            kind,
            last_closed_at: now_millis(),
            availability: DocumentAvailability::Unknown,
        };
        let mut history = self.load_unlocked()?;
        history.recent = insert_at_front(history.recent, entry);
        self.repository.save_recent(&history.recent)?;
        Ok(self.refresh_availability(history))
    }

    pub(crate) fn confirm_reopened(&self, path: &str) -> AppResult<DocumentHistory> {
        let _guard = self.lock()?;
        let mut history = self.load_unlocked()?;
        if history
            .closed
            .first()
            .is_some_and(|entry| same_path(&entry.path, path))
        {
            history.closed.remove(0);
            self.repository.save_closed(&history.closed)?;
        }
        Ok(self.refresh_availability(history))
    }

    pub(crate) fn clear(&self) -> AppResult<DocumentHistory> {
        let _guard = self.lock()?;
        self.repository.save_recent(&[])?;
        self.repository.save_closed(&[])?;
        Ok(DocumentHistory::default())
    }

    pub(crate) fn import_legacy(
        &self,
        recent: Vec<DocumentHistoryEntry>,
        closed: Vec<DocumentHistoryEntry>,
    ) -> AppResult<DocumentHistory> {
        let _guard = self.lock()?;
        let current = self.load_unlocked()?;
        let recent = normalize([current.recent, recent].concat());
        let closed = normalize([current.closed, closed].concat());
        self.repository.save_recent(&recent)?;
        self.repository.save_closed(&closed)?;
        Ok(self.refresh_availability(DocumentHistory { recent, closed }))
    }

    fn refresh_availability(&self, mut history: DocumentHistory) -> DocumentHistory {
        let mut statuses = HashMap::new();
        for entry in history.recent.iter_mut().chain(history.closed.iter_mut()) {
            let key = path_key(&entry.path);
            entry.availability = *statuses
                .entry(key)
                .or_insert_with(|| self.path_inspector.inspect(&entry.path));
        }
        history
    }

    fn lock(&self) -> AppResult<std::sync::MutexGuard<'_, ()>> {
        self.operation_lock
            .lock()
            .map_err(|_| AppError::StateUnavailable)
    }
}

fn insert_at_front(
    entries: Vec<DocumentHistoryEntry>,
    entry: DocumentHistoryEntry,
) -> Vec<DocumentHistoryEntry> {
    normalize(std::iter::once(entry).chain(entries).collect())
}

fn normalize(mut entries: Vec<DocumentHistoryEntry>) -> Vec<DocumentHistoryEntry> {
    entries.retain(|entry| !entry.path.trim().is_empty());
    entries.sort_by(|left, right| right.last_closed_at.cmp(&left.last_closed_at));
    let mut unique = Vec::with_capacity(entries.len().min(MAXIMUM_HISTORY_ENTRIES));
    for entry in entries {
        if unique
            .iter()
            .any(|existing: &DocumentHistoryEntry| same_document(existing, &entry))
        {
            continue;
        }
        unique.push(entry);
        if unique.len() == MAXIMUM_HISTORY_ENTRIES {
            break;
        }
    }
    unique
}

fn same_document(left: &DocumentHistoryEntry, right: &DocumentHistoryEntry) -> bool {
    if same_path(&left.path, &right.path) {
        return true;
    }
    match (&left.document_id, &right.document_id) {
        (Some(left_id), Some(right_id)) => left_id == right_id,
        (Some(_), None) | (None, Some(_)) => {
            left.kind == right.kind && left.title.eq_ignore_ascii_case(&right.title)
        }
        (None, None) => false,
    }
}

fn attach_document_ids(history: &mut [DocumentHistoryEntry], library: &[StoredDocumentEntry]) {
    for entry in history.iter_mut().filter(|entry| entry.document_id.is_none()) {
        let exact = library.iter().find(|stored| stored.kind == entry.kind
            && (same_path(&stored.path, &entry.path) || same_path(&stored.source_path, &entry.path)));
        let matched = exact.or_else(|| {
            let mut candidates = library.iter().filter(|stored| stored.kind == entry.kind
                && stored.title.eq_ignore_ascii_case(&entry.title));
            let first = candidates.next()?;
            candidates.next().is_none().then_some(first)
        });
        entry.document_id = matched.map(|stored| stored.id.clone());
    }
}

fn same_path(left: &str, right: &str) -> bool {
    path_key(left) == path_key(right)
}

fn path_key(path: &str) -> String {
    let normalized = path.trim().replace('/', "\\").to_lowercase();
    let normalized = if let Some(remainder) = normalized.strip_prefix(r"\\?\unc\") {
        format!(r"\\{remainder}")
    } else {
        normalized.strip_prefix(r"\\?\").unwrap_or(&normalized).to_owned()
    };
    normalized.trim_end_matches('\\').to_owned()
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        application::ports::{DocumentHistoryRepository, DocumentPathInspector},
        domain::{
            document::DocumentKind,
            document_history::{DocumentAvailability, DocumentHistoryEntry},
        },
        error::AppResult,
    };

    use super::DocumentHistoryService;

    #[derive(Default)]
    struct FakeRepository {
        recent: Mutex<Vec<DocumentHistoryEntry>>,
        closed: Mutex<Vec<DocumentHistoryEntry>>,
    }

    struct FakePathInspector;

    impl DocumentPathInspector for FakePathInspector {
        fn inspect(&self, path: &str) -> DocumentAvailability {
            if path.contains("missing") {
                DocumentAvailability::Missing
            } else {
                DocumentAvailability::Available
            }
        }
    }

    impl DocumentHistoryRepository for FakeRepository {
        fn load_recent(&self) -> AppResult<Vec<DocumentHistoryEntry>> {
            Ok(self.recent.lock().unwrap().clone())
        }
        fn save_recent(&self, entries: &[DocumentHistoryEntry]) -> AppResult<()> {
            *self.recent.lock().unwrap() = entries.to_vec();
            Ok(())
        }
        fn load_closed(&self) -> AppResult<Vec<DocumentHistoryEntry>> {
            Ok(self.closed.lock().unwrap().clone())
        }
        fn save_closed(&self, entries: &[DocumentHistoryEntry]) -> AppResult<()> {
            *self.closed.lock().unwrap() = entries.to_vec();
            Ok(())
        }
    }

    #[test]
    fn legacy_import_keeps_the_newest_path_and_sorts_by_close_time() {
        let service = service();
        let history = service
            .import_legacy(
                vec![
                    entry("C:/docs/a.pdf", 10),
                    entry("C:\\docs\\a.pdf", 30),
                    entry("b.md", 20),
                ],
                vec![],
            )
            .unwrap();

        assert_eq!(history.recent.len(), 2);
        assert_eq!(history.recent[0].last_closed_at, 30);
        assert_eq!(history.recent[1].path, "b.md");
    }

    #[test]
    fn confirmation_only_pops_the_closed_stack_top() {
        let service = service();
        service
            .import_legacy(vec![], vec![entry("older.md", 10), entry("latest.pdf", 20)])
            .unwrap();

        let unchanged = service.confirm_reopened("older.md").unwrap();
        assert_eq!(unchanged.closed.len(), 2);
        let popped = service.confirm_reopened("latest.pdf").unwrap();
        assert_eq!(popped.closed.len(), 1);
        assert_eq!(popped.closed[0].path, "older.md");
    }

    #[test]
    fn recording_access_updates_recent_without_adding_a_closed_document() {
        let service = service();
        let history = service
            .record_accessed(Some("pdf-guide".into()), "guide.pdf".into(), "Guide".into(), DocumentKind::Pdf)
            .unwrap();

        assert_eq!(history.recent.len(), 1);
        assert!(history.closed.is_empty());
    }

    #[test]
    fn refreshes_file_availability_without_persisting_it() {
        let service = service();
        let history = service
            .import_legacy(
                vec![entry("missing.pdf", 10), entry("available.md", 20)],
                vec![],
            )
            .unwrap();

        assert_eq!(
            history.recent[0].availability,
            DocumentAvailability::Available
        );
        assert_eq!(
            history.recent[1].availability,
            DocumentAvailability::Missing
        );
    }

    #[test]
    fn loading_rewrites_duplicate_recent_paths_in_normalized_form() {
        let repository = Arc::new(FakeRepository::default());
        *repository.recent.lock().unwrap() = vec![
            entry(r"\\?\C:\docs\guide.pdf", 30),
            entry("c:/docs/guide.pdf", 20),
            entry("C:\\docs\\notes.md", 10),
        ];
        let service = DocumentHistoryService::new(repository.clone(), Arc::new(FakePathInspector));

        let history = service.load_with_library(&[]).unwrap();

        assert_eq!(history.recent.len(), 2);
        assert_eq!(history.recent[0].last_closed_at, 30);
        assert_eq!(repository.recent.lock().unwrap().len(), 2);
    }

    #[test]
    fn recording_the_same_document_id_replaces_its_previous_path() {
        let service = service();
        service.record_accessed(Some("pdf-guide".into()), "C:/source/guide.pdf".into(), "guide.pdf".into(), DocumentKind::Pdf).unwrap();
        let history = service.record_accessed(Some("pdf-guide".into()), "C:/Bambook/pdf-guide/document.pdf".into(), "guide.pdf".into(), DocumentKind::Pdf).unwrap();

        assert_eq!(history.recent.len(), 1);
        assert_eq!(history.recent[0].path, "C:/Bambook/pdf-guide/document.pdf");
    }

    #[test]
    fn recording_a_package_replaces_it_in_both_recent_and_closed_indexes() {
        let service = service();
        service.record_closed(Some("pdf-guide-package".into()), "C:/exports/guide/guide.pdf".into(), "guide.pdf".into(), DocumentKind::Pdf).unwrap();
        let history = service.record_closed(Some("pdf-guide-package".into()), "D:/exports/guide/guide.pdf".into(), "guide.pdf".into(), DocumentKind::Pdf).unwrap();

        assert_eq!(history.recent.len(), 1);
        assert_eq!(history.closed.len(), 1);
        assert_eq!(history.recent[0].path, "D:/exports/guide/guide.pdf");
        assert_eq!(history.closed[0].path, "D:/exports/guide/guide.pdf");
    }

    fn service() -> DocumentHistoryService {
        DocumentHistoryService::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakePathInspector),
        )
    }

    fn entry(path: &str, last_closed_at: u64) -> DocumentHistoryEntry {
        DocumentHistoryEntry {
            document_id: None,
            path: path.into(),
            title: path.into(),
            kind: if path.ends_with(".pdf") {
                DocumentKind::Pdf
            } else {
                DocumentKind::Markdown
            },
            last_closed_at,
            availability: DocumentAvailability::Unknown,
        }
    }
}
