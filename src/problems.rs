use core::fmt;

use rand::random_range;

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
        static IMPLEMENTED_PROBLEMS:[ImplementedProblem; 6] = [ImplementedProblem::LargeFormatFourOp(BasicOperation::Undecided),ImplementedProblem::MissingOperand(BasicOperation::Undecided),ImplementedProblem::FourOp(BasicOperation::Undecided),/*ImplementedProblem::LongDiv,*/ImplementedProblem::ShortDiv,ImplementedProblem::Proportions,ImplementedProblem::DirectPercent];
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
            ImplementedProblem::DirectPercent => 2,
            _ => 0,
        }
    }

    pub fn fields(&self) -> Vec<String> {
        match *self {
            ImplementedProblem::MissingOperand(_) => vec![String::from("Operand 1"), String::from("Result")],
            ImplementedProblem::LongDiv | ImplementedProblem::ShortDiv => vec![String::from("Dividend"), String::from("Divisor")],
            ImplementedProblem::Proportions => vec![String::from("Numerator 1"),String::from("Denominator 1"), String::from("Numerator 2"), String::from("Denominator 2")],
            ImplementedProblem::DirectPercent => vec![String::from("Percentage"), String::from("Whole")],
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
pub struct Bound {
    pub main_lower: i32,
    pub main_upper: i32,
    pub aux_lower: i32,
    pub aux_upper:i32,
    pub denom_upper:u8,
}

impl Bound {
    fn new() -> Bound {
        Bound {
            main_lower: 0,
            main_upper: 100,
            aux_lower: 0,
            aux_upper: 10,
            denom_upper: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Term {
    pub whole: String,
    pub numerator: u32,
    pub denominator: u32,
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.denominator == 0 {
            write!(f, "{}", self.whole)
        } else {
            write!(f, "{} ({}) / ({})", self.whole, self.numerator, self.denominator)
        }
    }
}

impl Term {
    pub fn new(whole:String) -> Term {
        Term {
            whole,
            numerator: 0,
            denominator: 0,
        }
    }

    pub fn default() -> Term {
        Term {
            whole: String::new(),
            numerator: 0,
            denominator: 0,
        }
    }
}

#[derive(Clone)]
pub struct MathProblem {
    pub randomized: bool,
    pub auxillary: bool,
    pub allow_fractions: bool,
    pub allow_decimals: bool,
    pub digits_after_decimal: u8,
    pub main_bound: Bound,
    pub lower_bound: i32,
    pub upper_bound: i32,
    pub problem_type: ImplementedProblem,
    pub terms: Vec<Term>,
}

impl MathProblem {
    pub fn new() -> MathProblem {
        MathProblem {
            randomized: true,
            auxillary: false,
            allow_fractions: false,
            allow_decimals: false,
            digits_after_decimal: 0,
            main_bound: Bound::new(),
            lower_bound: 0,
            upper_bound: 100,
            problem_type: ImplementedProblem::MoreToCome,
            terms: Vec::new(),
        }
    }

    fn gen_term(&self) -> String {
        return String::new();
    }

    fn gen_aux_term(&self) -> String {
        return String::new();
    }

    pub fn generate(&mut self) {
        if !self.randomized {return;}

        self.terms = match self.problem_type {
            ImplementedProblem::LargeFormatFourOp(BasicOperation::Subtraction)
            | ImplementedProblem::MissingOperand(BasicOperation::Subtraction)
            | ImplementedProblem::FourOp(BasicOperation::Subtraction) => {if !self.auxillary {
                    let operand = random_range(self.lower_bound..self.upper_bound);
                    vec![Term::new(operand.to_string()), Term::new(random_range(self.lower_bound..operand).to_string())]
                } else {vec![Term::new(random_range(self.lower_bound..self.upper_bound).to_string()),Term::new(random_range(self.lower_bound..self.upper_bound).to_string())]}
            },
            ImplementedProblem::ShortDiv | ImplementedProblem::LongDiv => {
                let divisor = random_range(self.lower_bound..self.upper_bound);
                if self.auxillary {
                    vec![Term::new(random_range(divisor..self.upper_bound).to_string()), Term::new(divisor.to_string())]
                } else {vec![Term::new((divisor * random_range(self.lower_bound..self.upper_bound)).to_string()), Term::new(divisor.to_string())]}
            },
            ImplementedProblem::Proportions => {
                let mut terms = (0..4).map(|_| Term::new(random_range(self.lower_bound..self.upper_bound).to_string())).collect::<Vec<Term>>();
                terms[random_range(0..4)] = Term::new(String::from("x"));
                terms
            },
            // ImplementedProblem::DirectPercent => {

            // },
            _ => (0..self.problem_type.required_operands()).map(|_| Term::new(random_range(self.lower_bound..self.upper_bound).to_string())).collect::<Vec<Term>>()
        }

    }

    pub fn display(&self) -> String {
        if self.problem_type.required_operands() > self.terms.len() {return String::from("Error Occurred");}

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