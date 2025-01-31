use crate::prelude::*;
use egui_tabular::rvariant::VariantTy;
use egui_tabular::{RequiredColumn, RequiredColumns, TabularImporter};

#[derive(Default, Serialize, Deserialize)]
pub struct TabBomImporter {
    bom_instance: char,
    #[serde(skip)]
    inner: Inner,
}

struct Inner {
    tabular_importer: TabularImporter,
}

impl Default for Inner {
    fn default() -> Self {
        let required_columns = RequiredColumns::new([
            RequiredColumn::new("key", VariantTy::Str).synonyms(["parameter", "parameter_name"]),
            RequiredColumn::new("value", VariantTy::U32),
        ]);
        Self {
            tabular_importer: TabularImporter::new(required_columns),
        }
    }
}

impl TabBomImporter {
    pub fn new(bom_instance: char) -> Self {
        Self {
            bom_instance,
            inner: Inner::default(),
        }
    }
}

impl TabUi for TabBomImporter {
    fn title(&self) -> WidgetText {
        format!("BOM {}", self.bom_instance).into()
    }

    fn ui(&mut self, ui: &mut Ui, _cx: &mut Context) {
        self.inner.tabular_importer.show(ui);
    }

    fn is_closeable(&self) -> bool {
        false
    }
}
