use crate::app::state::AppState;
use crate::features::pdf::PdfDocumentModel;
use crate::shared::error::AppResult;

// 打开文件操作
pub(crate) fn open(
    state: &AppState,
    path: &str,
    password: Option<&str>,
) -> AppResult<PdfDocumentModel> {
    let settings = state.settings.load()?;  // app 的设置信息
    let document = state.pdf_documents.open(path, password, &settings)?;    // 通过AppState状态信息，打开pdf文件
    if let Err(error) = state
        .search
        .register_pdf(&document.session_id, &document.path, password)
    {
        let _ = state.pdf_documents.close(&document.session_id);
        return Err(error);
    }
    Ok(document)
}
pub(crate) fn authenticate(
    state: &AppState,
    session_id: &str,
    password: &str,
) -> AppResult<PdfDocumentModel> {
    let document = state.pdf_documents.authenticate(session_id, password)?;
    state.search.authenticate_pdf(session_id, password)?;
    Ok(document)
}
pub(crate) fn close(state: &AppState, session_id: &str) -> AppResult<()> {
    let search_result = state.search.close_pdf(session_id);
    let document_result = state.pdf_documents.close(session_id);
    document_result.and(search_result)
}
pub(crate) fn persist(state: &AppState, session_id: &str) -> AppResult<()> {
    let settings = state.settings.load()?;
    state
        .pdf_documents
        .persist(session_id, &settings)
        .map(|_| ())
}
