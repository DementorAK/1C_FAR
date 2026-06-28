use crate::v8::styles::PresentationStyle;
use crate::v8::vfs_builder::{BuildVfsError, VfsEntry};
use std::collections::HashMap;

pub struct EdtStyle;

impl PresentationStyle for EdtStyle {
    fn build_vfs(
        &self,
        _rows_map: &HashMap<String, Vec<u8>>,
    ) -> Result<Vec<VfsEntry>, BuildVfsError> {
        // Заглушка
        Ok(Vec::new())
    }

    fn sync_vfs_to_rows(&self, _vfs: &[VfsEntry], _updates: &mut HashMap<String, Vec<u8>>) {
        // Заглушка
    }
}
