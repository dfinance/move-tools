use std::collections::HashMap;
use utils::FilePath;

pub type OffsetsMap = HashMap<(usize, usize), usize>;

#[derive(Default, Clone)]
pub struct ProjectOffsetsMap(pub HashMap<FilePath, OffsetsMap>);

impl ProjectOffsetsMap {
    pub fn with_file_map(fpath: FilePath, map: OffsetsMap) -> ProjectOffsetsMap {
        let mut project_map = ProjectOffsetsMap::default();
        project_map.0.insert(fpath, map);
        project_map
    }

    pub fn insert(&mut self, fpath: FilePath, map: OffsetsMap) {
        self.0.insert(fpath, map);
    }

    pub fn apply_offsets_to_error(&self, error: CompilerError) -> CompilerError {
        error
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub fpath: FilePath,
    pub span: (usize, usize),
}

#[derive(Debug, Clone)]
pub struct CompilerErrorPart {
    pub location: Location,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CompilerError {
    pub parts: Vec<CompilerErrorPart>,
}

#[derive(Default)]
pub struct CompilerErrors(pub Vec<CompilerError>, ProjectOffsetsMap);

impl CompilerErrors {
    pub fn new(
        compiler_errors: Vec<CompilerError>,
        offsets_map: ProjectOffsetsMap,
    ) -> CompilerErrors {
        CompilerErrors(compiler_errors, offsets_map)
    }

    pub fn apply_offsets(self) -> Vec<CompilerError> {
        let CompilerErrors(errors, offsets_map) = self;
        errors
            .into_iter()
            .map(|error| offsets_map.apply_offsets_to_error(error))
            .collect()
    }

    pub fn extend(&mut self, other: CompilerErrors) {
        let CompilerErrors(errors, proj_offsets_map) = other;
        self.0.extend(errors);
        for (fpath, offsets_map) in proj_offsets_map.0.into_iter() {
            self.1.insert(fpath, offsets_map);
        }
    }
}
