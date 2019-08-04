use pyo3::prelude::*;
use pyo3::AsPyPointer;
use std::fmt;
use std::hash::Hash;
use std::os::raw::{c_int, c_void};

use scc::FindAdjacent;

unsafe extern "C" fn visit(object: *mut pyo3::ffi::PyObject, arg: *mut c_void) -> c_int {
    let py = pyo3::Python::assume_gil_acquired();
    let pyobj = PyObject::from_borrowed_ptr(py, object);

    let result: &mut Vec<PyObject> = &mut *(arg as *mut Vec<PyObject>);
    result.push(pyobj);

    0
}

impl<'p> FindAdjacent for PyObjectWrapper<'p> {
    fn find_adjacent(&self) -> Vec<Self> {
        let obj_ptr = self.obj.as_ptr();
        let type_obj = unsafe { *(*obj_ptr).ob_type };
        let mut result = Vec::<PyObject>::new();
        if let Some(traverseproc) = type_obj.tp_traverse {
            let result_ptr: *mut c_void = &mut result as *mut _ as *mut c_void;
            unsafe {
                traverseproc(obj_ptr, visit, result_ptr);
            }
        }

        result
            .into_iter()
            .map(|pyobject| PyObjectWrapper {
                obj: pyobject,
                py: self.py,
            })
            .collect()
    }
}

pub struct PyObjectWrapper<'p> {
    obj: PyObject,
    py: Python<'p>,
}

impl<'p> fmt::Debug for PyObjectWrapper<'p> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PyObjectWrapper({:?})", self.obj)
    }
}

impl<'p> PyObjectWrapper<'p> {
    pub fn new(py: Python<'p>, obj: PyObject) -> Self {
        PyObjectWrapper { obj: obj, py: py }
    }
}

impl<'p> Clone for PyObjectWrapper<'p> {
    fn clone(&self) -> Self {
        Self {
            obj: self.obj.clone_ref(self.py),
            py: self.py,
        }
    }
}

impl<'p> PartialEq for PyObjectWrapper<'p> {
    fn eq(&self, other: &Self) -> bool {
        self.obj.as_ptr() == other.obj.as_ptr()
    }
}

impl<'p> Eq for PyObjectWrapper<'p> {}

impl<'p> Hash for PyObjectWrapper<'p> {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.obj.as_ptr().hash(state)
    }
}

#[cfg(test)]
mod tests {
    use super::FindAdjacent;
    use super::PyObjectWrapper;
    use pyo3::{prelude::*, types::PyList};

    #[test]
    fn test_find_adjacent() {
        // Fix environment first: $ export PYTHONHOME=/anaconda3
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = PyList::new(py, &[1, 2, 3]);
        let list2 = PyList::new(py, &[list]);
        let list2_obj = list2.to_object(py);
        let list2_wrapper = PyObjectWrapper::new(py, list2_obj);

        let adjacent: Vec<PyObject> = list2_wrapper
            .find_adjacent()
            .into_iter()
            .map(|wrapper| wrapper.obj)
            .collect();
        assert_eq!(adjacent, vec![list.to_object(py)]);
    }
}
