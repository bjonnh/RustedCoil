#![feature(test)]
use std::time::Instant;
use std::result;
use std::fmt;
use std::error::Error;
use std::collections::VecDeque;
extern crate test;
extern crate scoped_threadpool;
use scoped_threadpool::Pool;


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

type ColumnElement<T> = VecDeque<T>;
struct SubColumn<T> {
    upper: ColumnElement<T>,
    lower: ColumnElement<T>,
    output: ColumnElement<T>,
    kval: T
}

struct Column {
    subcolumns: VecDeque<SubColumn<f32>>
}

impl SubColumn<f32> {
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
    fn push_equilibrate_upper(&mut self) -> Result<()> {
        // Get last value of upper
        {
            let  back = self.upper.back_mut().unwrap();

            self.output.push_front(*back);
            *back = 0.0;
        }
        self.upper.push_front(0.0);
        self.upper.pop_back();
        self.equilibrate();
        Ok(())
    }

}

impl Column {
    fn add_subcolumn(&mut self) {
        self.subcolumns.push_back(
            SubColumn{upper:ColumnElement::new(),
                      lower:ColumnElement::new(),
                      output:ColumnElement::new(),
                      kval:1.0});
    }


    fn grow(&mut self) {
        // Used to grow the column size (we should probably have a default size
        // to avoid allocation everytime)

        // This should be linked to the add subcolumn so the subcolumn
        // is already the right size, maybe we should keep length around

        for subcolumn in self.subcolumns.iter_mut() {
            subcolumn.grow();
        }
    }

    fn equilibrate(&mut self) {
        // Should be parallelized here
        // Need to handle errors
        for subcolumn in self.subcolumns.iter_mut() {
            subcolumn.equilibrate();
        }

    }

    fn push_equilibrate_upper(&mut self, loops: u32, threads: u32) {
        // Should be parallelized here
        // let (tx, rx) = mpsc::channel();
                    //self.subcolumns[index].push_equilibrate_upper().unwrap();
        //self.output[index].push_front(out);
/*        for e in self.subcolumns.iter_mut() {
            for _ in 0..loops {
                let out = e.push_equilibrate_upper().unwrap();
                e.output.push_front(out);
            }
        }*/
        
        let mut pool = Pool::new(threads);
        pool.scoped(|scoped| {
            for e in &mut self.subcolumns {
                scoped.execute(move || {
                    for _ in 0..loops {
                        e.push_equilibrate_upper().unwrap();
                    }
                });
            }
        });

    }


/*    fn pretty_print(&mut self) {
        for index in 0..self.upper.len() {
            print!("{:.2} | ", self.upper[index])
        }
        println!("");
        for index in 0..self.lower.len() {
            print!("{:.2} | ", self.lower[index])
        }
        println!("");
    }*/
}


fn main() {
    let start = Instant::now();

    let mut column = Column { subcolumns: VecDeque::new()};
    for _ in 0..10 {
        column.add_subcolumn();
    }

    for _ in 0..100 {
        column.grow();

    }

    for i in 0..10 {
        column.subcolumns[i].upper[0] = 1.0;
    }

    //column.upper[0] = 1.0;
    //column.pretty_print(); // that's just for small columns and debug
/*    match column.equilibrate() {
        Ok(()) => {},
        Err(err) => println!("ERROR: {}", err)
}*/
    column.equilibrate();
    //column.pretty_print(); // that's just for small columns and debug

    column.push_equilibrate_upper(100000, 4);
//        column.pretty_print();
//        println!("");
    println!("Worked here");
    // TODO Inverted here should be better
    // TODO That would be better with indices anyway
/*    for subcol in column.subcolumns.iter() {
        for j in 0..subcol.output.len() {
            print!("{},", subcol.output[j])
        }
        println!("")
    }
     */
    let elapsed = start.elapsed();
    println!("Elapsed: {} ms",
             (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64);
}


#[cfg(test)]
mod tests {
    use super::Column;
    use std::collections::VecDeque;
    use test::Bencher;

    fn bench(threads: u32) {
        
            let mut column = Column { subcolumns: VecDeque::new()};
            for _ in 0..10 {
                column.add_subcolumn();
            }

            for _ in 0..100 {
                column.grow();
            }

            for i in 0..10 {
                column.subcolumns[i].upper[0] = 1.0;
            }

            column.push_equilibrate_upper(1000, threads);
        
    }

    #[bench]
    fn benchmark_singlethread(b: &mut Bencher) {
        b.iter(|| bench(1));
    }

    #[bench]
    fn benchmark_twothread(b: &mut Bencher) {
        b.iter(|| bench(2));
    }

    #[bench]
    fn benchmark_threethread(b: &mut Bencher) {
        b.iter(|| bench(3));
    }

    #[bench]
    fn benchmark_fourthread(b: &mut Bencher) {
        b.iter(|| bench(4));
    }

}
