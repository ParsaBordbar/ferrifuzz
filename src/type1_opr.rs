use pyo3::prelude::*;

// Internal Operators

fn max(&meu1, &meu2) {
    if meu1 >= meu2 {
        return meu1
    } else 
    return meu2
}

fn min(&meu1, &meu2) {
    if meu1 >= meu2 {
        return meu2
    } else 
        return meu1
}

fn product(&meu1, &meu2) {
    return meu1 * meu2
}

#[pyclass]
#[derive(Clone)]
pub struct Operator {
    // #[pyo3(get)]
    // pub name: String,
    // #[pyo3(get)]
    // pub meu1: f64,
    // #[pyo3(get)]
    // pub meu2: f64,
}


impl Operator {

    #[new]
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            rules: Vec::new(),
        }
    }

    #[pymethods]
    pub fn t_norm(meu1: f32, meu2: f32, kind: String) -> {
        match kind {
            "min" => return min(meu1, meu2)
            "product" => return product(meu1, meu2)
            _ => { // Clean This Block When the Documentation is available!
                return min(meu1, meu2)
        }
    }


    #[pymethods]
    pub fn t_conorm(meu1: f32, meu2: f32, kind: String) -> {
        match kind {
            "max" => {

            }
            _ => { // Clen This Block When the Documentation is available!
                if meu1 >= meu2 {
                    return meu1
                } else 
                return meu2
        }
    }
    // Todo Add Composite and Other types of Operators!
}


