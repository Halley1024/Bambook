use crate::features::pdf::PdfDocumentModel;
use crate::features::pdf::PdfPageStructure;
use crate::features::pdf::RenderedPdfPage;
use crate::shared::error::AppResult;

/**
 * 定义接入引擎的所需要实现的接口，之后换引擎就只需要实现这个接口即可
 * open 打开文件
 * authenticate 验证文件
 * page_structure 获取页面结构
 * render_page 渲染页面
 * close 关闭文件
 */
pub(crate) trait PdfEngine: Send + Sync {
    fn open(&self, path: &str, password: Option<&str>) -> AppResult<PdfDocumentModel>;
    fn authenticate(&self, session_id: &str, password: &str) -> AppResult<PdfDocumentModel>;
    fn page_structure(&self, session_id: &str, page_index: u32) -> AppResult<PdfPageStructure>;
    fn render_page(
        &self,
        session_id: &str,
        page_index: u32,
        scale: f32,
    ) -> AppResult<RenderedPdfPage>;
    fn close(&self, session_id: &str) -> AppResult<()>;
}
