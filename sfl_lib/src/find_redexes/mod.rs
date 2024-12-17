use crate::{ASTNode, ASTNodeType};
mod replace;
mod reduce;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum ProgressType {
    None, 
    Replace,
    Reduce
}

#[derive(Debug)]
pub struct Progress {
    pub redex : ASTNode,
    pub contraction : Option<ASTNode>,
    pub progress_type : ProgressType
}

pub fn find_redexes(root : &ASTNode) -> Result<Vec<Progress>, ()> {
    let mut progress = vec![];

    for redex in replace::find(root)? {
        progress.push(Progress {
            redex,
            contraction : None,
            progress_type : ProgressType::Replace
        });
    }

    Ok(progress)
}