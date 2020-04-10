use lsp_types::{Diagnostic, DiagnosticRelatedInformation, Location, Range, Url};
use move_ir_types::location::Loc;
use move_lang::errors::{Error, FilesSourceText};
use move_lang::shared::Address;

use crate::utils::location::File;

pub type FilePath = &'static str;

#[derive(Debug, Default, Clone)]
pub struct RootDatabase {
    pub sender_address: Address,
    pub project_files_mapping: FilesSourceText,
}

impl RootDatabase {
    pub fn apply_change(&mut self, change: AnalysisChange) {
        if let Some(address) = change.address_changed {
            self.sender_address = address;
        }
        for (fpath, text) in change.files_updated {
            self.project_files_mapping.insert(fpath, text);
        }
    }

    pub fn libra_error_into_diagnostic(&self, error: Error) -> Diagnostic {
        assert!(!error.is_empty(), "Libra's Error is an empty Vec");
        let (primary_loc, primary_message) = error.get(0).unwrap().to_owned();
        let mut diagnostic = {
            let range = self.loc_to_range(primary_loc);
            Diagnostic::new_simple(range, primary_message)
        };
        // first error is an actual one, others are related info
        if error.len() > 1 {
            let mut related_info = vec![];
            for (related_loc, related_message) in error[1..].iter() {
                let range = self.loc_to_range(*related_loc);
                let related_fpath = related_loc.file();
                let file_uri = Url::from_file_path(related_fpath)
                    .unwrap_or_else(|_| panic!("Cannot build Url from path {:?}", related_fpath));

                let related_info_item = DiagnosticRelatedInformation {
                    location: Location::new(file_uri, range),
                    message: related_message.to_string(),
                };
                related_info.push(related_info_item);
            }
            diagnostic.related_information = Some(related_info)
        }
        diagnostic
    }

    fn loc_to_range(&self, loc: Loc) -> Range {
        let text = self
            .project_files_mapping
            .get(loc.file())
            .unwrap()
            .to_owned();
        let file = File::new(text);
        let start_pos = file.position(loc.span().start().to_usize()).unwrap();
        let end_pos = file.position(loc.span().end().to_usize()).unwrap();
        Range::new(start_pos, end_pos)
    }
}

#[derive(Default, Debug)]
pub struct AnalysisChange {
    address_changed: Option<Address>,
    files_updated: Vec<(FilePath, String)>,
}

impl AnalysisChange {
    pub fn new() -> Self {
        AnalysisChange::default()
    }

    pub fn update_file(&mut self, fname: FilePath, text: String) {
        self.files_updated.push((fname, text));
    }

    pub fn change_sender_address(&mut self, new_address: Address) {
        self.address_changed = Some(new_address);
    }
}
