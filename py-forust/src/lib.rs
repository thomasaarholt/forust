use forust::binning::bin_matrix;
use forust::data::Matrix;
use forust::gradientbooster::GradientBooster as CrateGradienBooster;
use forust::objective::ObjectiveType;
use forust::utils::percentiles as crate_percentiles;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

#[pyclass(subclass)]
struct GradientBooster {
    booster: CrateGradienBooster<f64>,
}

#[pymethods]
impl GradientBooster {
    #[new]
    pub fn new(
        objective_type: Option<&str>,
        iterations: Option<usize>,
        learning_rate: Option<f64>,
        max_depth: Option<usize>,
        max_leaves: Option<usize>,
        l2: Option<f64>,
        gamma: Option<f64>,
        min_leaf_weight: Option<f64>,
        base_score: Option<f64>,
        nbins: Option<u16>,
        parallel: Option<bool>,
    ) -> Self {
        let mut booster = CrateGradienBooster::<f64>::default();
        match objective_type {
            None => booster.objective_type = ObjectiveType::LogLoss,
            Some(s) => {
                if s == "LogLoss" {
                    booster.objective_type = ObjectiveType::LogLoss
                } else if s == "SquaredLoss" {
                    booster.objective_type = ObjectiveType::SquaredLoss
                } else {
                    panic!("Not a valid objective type provided.")
                }
            }
        }
        if let Some(x) = iterations {
            booster.iterations = x;
        }
        if let Some(x) = learning_rate {
            booster.learning_rate = x;
        }
        if let Some(x) = max_depth {
            booster.max_depth = x;
        }
        if let Some(x) = max_leaves {
            booster.max_leaves = x;
        }
        if let Some(x) = l2 {
            booster.l2 = x;
        }
        if let Some(x) = gamma {
            booster.gamma = x;
        }
        if let Some(x) = min_leaf_weight {
            booster.min_leaf_weight = x;
        }
        if let Some(x) = base_score {
            booster.base_score = x;
        }
        if let Some(x) = nbins {
            booster.nbins = x;
        }
        if let Some(x) = parallel {
            booster.parallel = x;
        }
        GradientBooster { booster }
    }

    pub fn fit(
        &mut self,
        flat_data: PyReadonlyArray1<f64>,
        rows: usize,
        cols: usize,
        y: PyReadonlyArray1<f64>,
        sample_weight: PyReadonlyArray1<f64>,
        parallel: Option<bool>,
    ) -> PyResult<()> {
        let flat_data = flat_data.as_slice()?;
        let data = Matrix::new(flat_data, rows, cols);
        let y = y.as_slice()?;
        let sample_weight = sample_weight.as_slice()?;
        let parallel = match parallel {
            None => true,
            Some(v) => v,
        };
        self.booster.fit(&data, &y, &sample_weight, parallel).unwrap();
        Ok(())
    }
    pub fn predict<'py>(
        &self,
        py: Python<'py>,
        flat_data: PyReadonlyArray1<f64>,
        rows: usize,
        cols: usize,
        parallel: Option<bool>,
    ) -> PyResult<&'py PyArray1<f64>> {
        let flat_data = flat_data.as_slice()?;
        let data = Matrix::new(flat_data, rows, cols);
        let parallel = match parallel {
            None => true,
            Some(v) => v,
        };
        Ok(self.booster.predict(&data, parallel).into_pyarray(py))
    }

    pub fn text_dump(
        &self,
    ) -> PyResult<Vec<String>> {
        let mut trees = Vec::new();
        for t in &self.booster.trees {
            trees.push(format!("{}", t));
        }
        return Ok(trees)
    }
}

#[pyfunction]
fn rust_bin_matrix<'py>(
    py: Python<'py>,
    flat_data: PyReadonlyArray1<f64>,
    rows: usize,
    cols: usize,
    sample_weight: PyReadonlyArray1<f64>,
    nbins: u16,
) -> PyResult<(&'py PyArray1<u16>, Vec<Vec<f64>>, Vec<usize>)> {
    let flat_data = flat_data.as_slice()?;
    let sample_weight = sample_weight.as_slice()?;
    let data = Matrix::new(flat_data, rows, cols);
    let r = bin_matrix(&data, sample_weight, nbins).unwrap();
    Ok((r.binned_data.into_pyarray(py), r.cuts, r.nunique))
}

#[pyfunction]
fn print_matrix(x: PyReadonlyArray1<f64>, rows: usize, cols: usize) -> PyResult<()> {
    let m = Matrix::new(x.as_slice()?, rows, cols);
    println!("{}", m);
    Ok(())
}

#[pyfunction]
fn percentiles<'py>(
    py: Python<'py>,
    v: PyReadonlyArray1<f64>,
    sample_weight: PyReadonlyArray1<f64>,
    percentiles: PyReadonlyArray1<f64>,
) -> PyResult<&'py PyArray1<f64>> {
    let v_ = v.as_slice()?;
    let sample_weight_ = sample_weight.as_slice()?;
    let percentiles_ = percentiles.as_slice()?;
    let p = crate_percentiles(v_, sample_weight_, percentiles_);
    Ok(p.into_pyarray(py))
}

/// A Python module implemented in Rust.
#[pymodule]
fn forust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(print_matrix, m)?)?;
    m.add_function(wrap_pyfunction!(percentiles, m)?)?;
    m.add_function(wrap_pyfunction!(rust_bin_matrix, m)?)?;
    m.add_class::<GradientBooster>()?;
    Ok(())
}
