use std::ffi::CStr;

use crate::{unwrap_or_bail, munge::TreeNode, AnyResult};
use super::{Resources, Resource, ScriptResource};

impl Resources {
    /// Parses a given resource. Returns None if it's already loaded, or unsupported
    pub fn parse_resource(&mut self, node: &TreeNode) -> AnyResult<Option<(String, Resource)>> {
        Ok(match node.name() {
            ids::SCR_ => {
                let children = node.parse_children()?;
                let mut name = None;
                let mut info = None;
                let mut body = None;

                for child in &children {
                    match child.name() {
                        ids::NAME => {
                            name = CStr::from_bytes_with_nul(child.data())?
                                .to_str()?
                                .to_string()
                                .into();
                        }
                        ids::INFO => {
                            info = Some(child.data()[0]);
                        }
                        ids::BODY => {
                            body = Some(child.data());
                        }
                        _ => {}
                    }
                }

                let name = unwrap_or_bail!(name, "Name not present in `scr_` resource");

                Some((
                    name.clone(),
                    Resource::Script(ScriptResource {
                        name,
                        info: unwrap_or_bail!(info, "Info not present in `scr_` resource"),
                        chunk: Vec::from(unwrap_or_bail!(
                            body,
                            "Chunk not present in `scr_` resource"
                        )),
                    }),
                ))
            }
            _ => None,
        })
    }
}

mod ids {
    use crate::munge::NodeName;

    pub const SCR_: NodeName = NodeName::from_literal("scr_");
    pub const NAME: NodeName = NodeName::from_literal("NAME");
    pub const INFO: NodeName = NodeName::from_literal("INFO");
    pub const BODY: NodeName = NodeName::from_literal("BODY");
}
