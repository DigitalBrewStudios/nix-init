use cargo::core::{PackageId, Resolve};
use semver::Version;

use crate::{inputs::AllInputs, macros::input_macros};

pub(super) fn load_js_dependency(inputs: &mut AllInputs, resolve: &Resolve, pkg: PackageId) {
    input_macros!(inputs);

    match &*pkg.name() {
        // keep-sorted start block=yes
        // TODO: List some deps that we need to add.
        // keep-sorted end
        _ => {}
    }
}
