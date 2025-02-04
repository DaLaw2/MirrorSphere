use crate::model::hash_type::HashType;

pub enum ComparisonMode {
    // Compare size and modify time
    Quick,
    // Quick + compare regular file attr
    Standard,
    // Standard + compare file checksum
    Thorough(HashType),
}
