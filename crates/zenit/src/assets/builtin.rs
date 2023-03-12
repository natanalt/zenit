//! Resources imported into the executable
//! 
//! Basically, all files from `zenit/assets/builtin` are imported here. This module provides some
//! helper functions for parsing files from here.

use include_dir::{include_dir, Dir};

pub const CONTENTS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/builtin");
