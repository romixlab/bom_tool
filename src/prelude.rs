pub use crate::context::Context;
pub use crate::tab_viewer::TabUi;
pub use egui::{Ui, WidgetText};
pub use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
pub use log::{debug, error, info, trace, warn};

#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_imports)]
pub use tracing::{debug, error, info, trace, warn};
