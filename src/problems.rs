use core::fmt;

#[derive(Clone, Copy, PartialEq)]
pub enum BasicOperation {
    Undecided,
    Addition,
    Subtraction,
    Multiplication,
}

impl BasicOperation {
    pub fn iterator() -> std::slice::Iter<'static, BasicOperation> {
        static OPERATIONS: [BasicOperation; 3] = [BasicOperation::Addition, BasicOperation::Subtraction, BasicOperation::Multiplication];
        OPERATIONS.iter()
    }
}

impl fmt::Display for BasicOperation {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        match *self {
            BasicOperation::Addition => write!(f, "+"),
            BasicOperation::Subtraction => write!(f, "-"),
            BasicOperation::Multiplication => write!(f,"*"),
            BasicOperation::Undecided => write!(f,"◦"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ImplementedProblem {
    LargeFormatFourOp(BasicOperation),
    MissingOperand(BasicOperation),
    FourOp(BasicOperation),
    LongDiv,
    ShortDiv,
    Proportions,
    DirectPercent,
    MoreToCome,
}

impl fmt::Display for ImplementedProblem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ImplementedProblem::LargeFormatFourOp(o) => write!(f,"Large Format Basic Arithmetic ({})", o),
            ImplementedProblem::MissingOperand(o) => write!(f,"Missing Operand ({})", o),
            ImplementedProblem::FourOp(o) => write!(f,"Basic Arithmetic ({})", o),
            ImplementedProblem::LongDiv => write!(f,"Long Division"),
            ImplementedProblem::ShortDiv => write!(f,"Horizontal Division"),
            ImplementedProblem::Proportions => write!(f,"Proportions"),
            ImplementedProblem::DirectPercent => write!(f,"Direct Percent"),
            _ => write!(f,"Unimplemented"),
        }
    }
}

impl ImplementedProblem {
    pub fn iterator() -> std::slice::Iter<'static, ImplementedProblem> {
        static IMPLEMENTED_PROBLEMS:[ImplementedProblem; 7] = [ImplementedProblem::LargeFormatFourOp(BasicOperation::Undecided),ImplementedProblem::MissingOperand(BasicOperation::Undecided),ImplementedProblem::FourOp(BasicOperation::Undecided),ImplementedProblem::LongDiv,ImplementedProblem::ShortDiv,ImplementedProblem::Proportions,ImplementedProblem::DirectPercent];
        IMPLEMENTED_PROBLEMS.iter()
    }

    pub fn required_operands(&self) -> usize {
        match *self {
            ImplementedProblem::LargeFormatFourOp(_) => 2,
            ImplementedProblem::MissingOperand(_) => 2,
            ImplementedProblem::FourOp(_) => 2,
            ImplementedProblem::LongDiv => 2,
            ImplementedProblem::ShortDiv => 2,
            ImplementedProblem::Proportions => 4,
            ImplementedProblem::DirectPercent => 3,
            _ => 0,
        }
    }

    pub fn fields(&self) -> Vec<String> {
        match *self {
            ImplementedProblem::MissingOperand(_) => vec![String::from("Operand 1"), String::from("Result")],
            ImplementedProblem::LongDiv | ImplementedProblem::ShortDiv => vec![String::from("Dividend"), String::from("Divisor")],
            ImplementedProblem::Proportions => vec![String::from("Numerator 1"),String::from("Denominator 1"), String::from("Numerator 2"), String::from("Denominator 2")],
            ImplementedProblem::DirectPercent => vec![String::from("Percentage"), String::from("Whole"), String::from("Part")],
            _ => (0..self.required_operands()).into_iter().map(|i| format!("Operand {}", i + 1)).collect::<Vec<String>>(),
        }
    }

    pub fn requires_operation(&self) -> bool {
        matches!(
            self,
            ImplementedProblem::LargeFormatFourOp(_)
                | ImplementedProblem::MissingOperand(_)
                | ImplementedProblem::FourOp(_)
        )
    }

    pub fn set_operation(&mut self, new_op: BasicOperation) {
        match self {
            ImplementedProblem::LargeFormatFourOp(op)
            | ImplementedProblem::MissingOperand(op)
            | ImplementedProblem::FourOp(op) => {
                *op = new_op;
            }
            _ => {} // do nothing if variant has no payload
        }
    }
}

#[derive(Clone)]
pub struct MathProblem {
    pub randomized: bool,
    pub allow_fracs: bool,
    pub problem_type: ImplementedProblem,
    pub terms: Vec<String>,
}

impl MathProblem {
    pub fn new() -> MathProblem {
        MathProblem { randomized: false, allow_fracs: false, problem_type: ImplementedProblem::MoreToCome, terms: Vec::new() }
    }

    pub fn display(&self) -> String {
        if self.problem_type.required_operands() > self.terms.len() {return String::from("Insufficient Operands\n\n");}
        match self.problem_type {
            ImplementedProblem::FourOp(o) => format!("${} {} {} = \\_\\_\\_$", self.terms[0], o, self.terms[1]),
            ImplementedProblem::MissingOperand(o) => format!("${} {} \\_\\_\\_ = {}$", self.terms[0], o, self.terms[1]),
            // ImplementedProblem::LongDiv => format!("${}overline(|{})$", self.terms[1], self.terms[0]),
            ImplementedProblem::ShortDiv => format!("${} ÷ {} = \\_\\_\\_$", self.terms[0], self.terms[1]),
            ImplementedProblem::Proportions => format!("$({})/({}) = ({})/({})$", self.terms[0], self.terms[1], self.terms[2], self.terms[3]),
            _ => format!("Unimplemented problem type: {} with parameters {:?}", self.problem_type, self.terms)
        }
    }
}