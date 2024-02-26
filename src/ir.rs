use std::collections::HashSet;

use crate::position::Located;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum IR {
    #[default]
    None,

    Jump {
        addr: usize,
    },
    JumpIf {
        negative: bool,
        cond: usize,
        addr: usize,
    },

    Call {
        dst: Option<usize>,
        func: usize,
        start: usize,
        amount: usize,
    },

    Move {
        dst: usize,
        src: usize,
    },
    Get {
        dst: usize,
        addr: usize,
    },
    Set {
        addr: usize,
        src: usize,
    },

    String {
        dst: usize,
        addr: usize,
    },
    Int {
        dst: usize,
        addr: usize,
    },
    Float {
        dst: usize,
        addr: usize,
    },

    List {
        dst: usize,
        length: usize,
    },
    Map {
        dst: usize,
    },

    Field {
        dst: usize,
        head: usize,
        field: usize,
    },
    FieldString {
        dst: usize,
        head: usize,
        addr: usize,
    },
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LabeledIR {
    pub ir: IR,
    pub label: Option<usize>,
}
impl LabeledIR {
    pub fn new(ir: IR) -> Self {
        Self { ir, label: None }
    }
    pub fn labeled(mut self, label: usize) -> Self {
        self.label = Some(label);
        self
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Closure {
    pub code: Vec<Located<LabeledIR>>,
    pub string: Vec<String>,
    pub int: Vec<i64>,
    pub float: Vec<f64>,
}

pub struct IRCompiler {
    pub closure_stack: Vec<Closure>,
    pub registers: Vec<HashSet<usize>>,
    pub labels: Vec<Vec<usize>>,
}
impl IRCompiler {
    pub fn new() -> Self {
        Self {
            closure_stack: vec![Closure::default()],
            registers: vec![HashSet::default()],
            labels: vec![vec![]],
        }
    }
    pub fn push_closure(&mut self) {
        self.closure_stack.push(Closure::default());
        self.registers.push(HashSet::default());
        self.labels.push(vec![]);
    }
    pub fn pop_closure(&mut self) -> Option<Closure> {
        self.registers.pop();
        self.labels.pop();
        self.closure_stack.pop()
    }
    pub fn closure(&self) -> Option<&Closure> {
        self.closure_stack.last()
    }
    pub fn closure_mut(&mut self) -> Option<&mut Closure> {
        self.closure_stack.last_mut()
    }
    pub fn registers(&self) -> Option<&HashSet<usize>> {
        self.registers.last()
    }
    pub fn cregisters_mut(&mut self) -> Option<&mut HashSet<usize>> {
        self.registers.last_mut()
    }
    pub fn labels(&self) -> Option<&Vec<usize>> {
        self.labels.last()
    }
    pub fn labels_mut(&mut self) -> Option<&mut Vec<usize>> {
        self.labels.last_mut()
    }
}
