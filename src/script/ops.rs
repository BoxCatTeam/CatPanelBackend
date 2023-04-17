use deno_core::{extension, op, OpState};
use serde::Deserialize;

extension!(
    cat_panel_component,
    ops = [op_reg_info],
    customizer = |ext: &mut deno_core::ExtensionBuilder| {
        ext.force_op_registration();
    },
);

#[derive(Debug, Deserialize)]
pub struct Info {
    name: String,
    available_version: Vec<String>,
}

#[op]
fn op_reg_info(state: &mut OpState, info: Info) {
    state.put(info);
}
