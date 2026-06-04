use cargo::core::{PackageId, Resolve};
use semver::Version;

use crate::{inputs::AllInputs, macros::input_macros};

pub(super) fn load_node_dependency(inputs: &mut AllInputs, resolve: &Resolve, pkg: PackageId) {
    input_macros!(inputs);

    match &*pkg.name() {
        // keep-sorted start block=yes

        // keep-sorted end
        _ => {}
    }
}
