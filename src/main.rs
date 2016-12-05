use std::result;
use std::fmt;
use std::error::Error;
use std::collections::VecDeque;

#[derive(Debug)]
enum CPCError {
    NonExistingCell,
    NullColumn
}

impl fmt::Display for CPCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CPCError::NonExistingCell => write!(f, "Equilibration attempt of a non-existing cell"),
            CPCError::NullColumn => write!(f, "The column has no cells"),
        }
    }
}

impl Error for CPCError {
    fn description(&self) -> &str {
        match *self {
            CPCError::NonExistingCell => "non-existing cell",
            CPCError::NullColumn => "The column has no cells",
        }
    }
}

type Result<T> = result::Result<T, CPCError>;

struct Column {
    upper: VecDeque<f32>,
    lower: VecDeque<f32>,
    output: VecDeque<f32>,
    kval: f32
}

impl Column {
    fn grow(&mut self) {
        self.upper.push_back(0.0);
        self.lower.push_back(0.0);
    }

    fn equilibrate_cell(&mut self, index: usize) -> Result<()> {
        if index>=self.upper.len() { return Err(CPCError::NonExistingCell)}
        let tot = self.upper[index] + self.lower[index];
        self.upper[index] = self.kval * tot / (1.0+self.kval);
        self.lower[index] = tot - self.upper[index];
        return Ok(())
    }

    fn equilibrate(&mut self) {
        for index in 0..self.upper.len() {
            // we can unwrap here we know the size of upper…
            self.equilibrate_cell(index).unwrap();
        }
    }

    fn push_equilibrate_upper(&mut self) -> Result<f32> {
        // Get last value of upper
        let out;
        match self.upper.back_mut() {
            Some(x) => {
                out = *x;
                self.output.push_front(*x);
                *x = 0.0;
            },
            None => return Err(CPCError::NullColumn),
        }

        self.upper.push_front(0.0);
        self.upper.pop_back();
        self.equilibrate();
        Ok(out)
    }

    fn pretty_print(&mut self) {
        for index in 0..self.upper.len() {
            print!("{:.2} | ", self.upper[index])
        }
        println!("");
        for index in 0..self.lower.len() {
            print!("{:.2} | ", self.lower[index])
        }
        println!("");
    }
}


fn main() {
    let mut column = Column { upper: VecDeque::new(),
                              lower: VecDeque::new(),
                              output: VecDeque::new(),
                              kval: 1.0};
    for i in 0..1000 {
        column.grow();
    }
    
    column.upper[0] = 1.0;
    //column.pretty_print(); // that's just for small columns and debug
    match column.equilibrate_cell(0) {
        Ok(()) => {},
        Err(err) => println!("ERROR: {}", err)
    }
    //column.pretty_print(); // that's just for small columns and debug
    for i in 0..10000 {
        column.push_equilibrate_upper();
//        column.pretty_print();
//        println!("");
    }

    for i in 0..column.output.len() {
        println!("{}", column.output[i])
    }
}
