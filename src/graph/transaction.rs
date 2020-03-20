use std::collections::BTreeSet;
use super::quad::Delta;

pub struct Transaction {
    pub deltas: BTreeSet<Delta>
}

