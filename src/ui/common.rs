#[derive(Debug, Clone, PartialEq)]
pub enum FolderSelectionMode {
    Source,
    Destination,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonModeSelection {
    Standard,
    Advanced,
    Thorough,
}
