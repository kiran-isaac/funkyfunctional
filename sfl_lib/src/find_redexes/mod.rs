use crate::{ASTNode, ASTNodeType};
mod replace;
mod reduce;

pub enum ProgressType {
    None, 
    Replace,
    Reduce
}

pub struct Progress {
    pub node : usize,
    pub progress_type : ProgressType
}

pub fn find_redexes(root : &ASTNode) -> Result<Vec<Progress>, ()> {
    let mut progress = vec![];

    let replacer = replace::ReplacementFinder::new();

    for redex in replacer.find(root)? {
        progress.push(Progress {
            node : redex,
            progress_type : ProgressType::Replace
        });
    }

    Ok(progress)
}