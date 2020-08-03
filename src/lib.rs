use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PyBytes};
use pyo3::PyObjectProtocol;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use pyo3::basic::CompareOp;

use edit_tree::{Apply, EditTree};
use bincode2::{deserialize, serialize};
use edit_tree::edit_tree::EditTree::MatchNode;

#[pymodule]
fn edit_tree(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEditTree>()?;
    Ok(())
}

#[pyclass(module = "edit_tree", name=PyEditTree)]
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct PyEditTree {
    inner: edit_tree::EditTree<char>,
}

#[pymethods]
impl PyEditTree {
    #[new]
    pub fn __new__(a: &str, b: &str) -> Self {
        let tree = match EditTree::create_tree(
            &a.chars().collect::<Vec<char>>(),
            &b.chars().collect::<Vec<char>>(),
        ) {
            Some(tree) => tree,
            None => MatchNode {
                pre: 0,
                suf: 0,
                left: None,
                right: None,
            }
        };
        PyEditTree { inner: tree }
    }

    fn apply(&self, a: &str) -> PyResult<String> {
        match self.inner.apply(&a.chars().collect::<Vec<char>>()) {
            Some(lem) => Ok(lem.iter().collect()),
            None => Err(exceptions::Exception::py_err(format!(
                "Couldnt apply {:?} to {}",
                &self.inner, &a
            ))),
        }
    }

    fn serialize_to_string(&self) -> PyResult<String> {
        Ok(serde_json::to_string(&self.inner).map_err(|_| {
            exceptions::Exception::py_err(format!(
                "Failed to serialize to string. {:?}",
                &self.inner
            ))
        })?)
    }

    #[staticmethod]
    fn deserialize_from_string(string: &str) -> PyResult<Self> {
        Ok(PyEditTree {
            inner: serde_json::from_str(string).map_err(|_| {
                exceptions::Exception::py_err(format!(
                    "Failed to deserialize edit tree from: {:?}",
                    string
                ))
            })?,
        })
    }


    pub fn __setstate__(&mut self, py: Python, state: PyObject) -> PyResult<()> {
        match state.extract::<&PyBytes>(py) {
            Ok(s) => {
                self.inner = deserialize(s.as_bytes()).unwrap();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        Ok(PyBytes::new(py, &serialize(&self.inner).unwrap()).to_object(py))
    }

    #[staticmethod]
    pub fn  __getnewargs__<'a>() -> (&'a str, &'a str) {
        ("","")
    }

}




#[pyproto]
impl PyObjectProtocol for PyEditTree {
    fn __hash__(&self) -> PyResult<isize> {
        let mut h = DefaultHasher::new();
        self.inner.hash(&mut h);
        let h: u64 = h.finish();
        let r: PyResult<isize> = Ok(h as isize);
        r
    }

    fn __str__(&self) -> PyResult<String> {
        self.serialize_to_string()
    }

    fn __richcmp__(&self, other: PyRef<'p, Self>, op: CompareOp) -> PyResult<bool> {

//        let other : &PyEditTree = if let Ok(other) = other.extract() {
//            other
//        } else {
//            return Ok(false);
//        };
        match op {
            CompareOp::Eq => Ok(self.inner.eq(&other.inner)),
            CompareOp::Ne => Ok(self.inner.ne(&other.inner)),
            _ => type_err("Can only use '==' and '!=' with EditTree.").into(),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.inner))
    }


}

pub(crate) fn type_err(msg: impl Into<String>) -> PyErr {
    exceptions::TypeError::py_err(msg.into())
}
