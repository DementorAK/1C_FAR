use crate::v8::styles::PresentationStyle;
use crate::v8::vfs_builder::{BuildVfsError, VfsEntry};
use std::collections::HashMap;

pub struct V8UnpackStyle;

impl PresentationStyle for V8UnpackStyle {
    fn build_vfs(
        &self,
        _rows_map: &HashMap<String, Vec<u8>>,
    ) -> Result<Vec<VfsEntry>, BuildVfsError> {
        // Заглушка: будет реализовано в 8.4
        Ok(Vec::new())
    }

    fn sync_vfs_to_rows(&self, _vfs: &[VfsEntry], _updates: &mut HashMap<String, Vec<u8>>) {
        // Заглушка
    }
}
