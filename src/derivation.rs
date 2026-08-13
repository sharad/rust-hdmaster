





use crate::{
    algorithm::Algorithm,
    error::Result,
    node::HdNode,
    path::{ChildIndex, DerivationPath},
    provider::ProviderRegistry,
    seed::MasterSeed,
    serialization::load_node,
};
use std::path::Path;





pub struct NodeDeriver {
    registry: ProviderRegistry,
}


impl Default for NodeDeriver {
    fn default() -> Self {
        Self {
            registry: ProviderRegistry::standard(),
        }
    }
}



impl NodeDeriver {

    pub fn new(registry: ProviderRegistry) -> Self {
        Self { registry }
    }

    pub fn derive_from_seed(
        &self,
        seed: &MasterSeed,
        algorithm: Algorithm,
        application: &str,
        path: &DerivationPath,
    ) -> Result<HdNode> {
        let p = self.registry.get(algorithm)?;
        let mut n = p.master(application, &seed.0)?;
        for i in &path.0 {
            n = p.child(&n, *i)?
        }
        Ok(n)
    }

    pub fn derive_child_from_node(&self, parent: &HdNode, path: &DerivationPath) -> Result<HdNode> {
        let p = self.registry.get(parent.algorithm)?;
        let mut n = parent.clone();
        for i in &path.0 {
            n = p.child(&n, *i)?
        }
        Ok(n)
    }

    pub fn derive_child_from_node_file(
        &self,
        file: &Path,
        path: &DerivationPath,
    ) -> Result<HdNode> {
        let n = load_node(file)?;
        self.derive_child_from_node(&n, path)
    }

    pub fn derive_child(&self, parent: &HdNode, index: ChildIndex) -> Result<HdNode> {
        self.derive_child_from_node(parent, &DerivationPath(vec![index]))
    }

    pub fn derive_child_from_file(&self, file: &Path, index: ChildIndex) -> Result<HdNode> {
        self.derive_child_from_node_file(file, &DerivationPath(vec![index]))
    }
}
