use pyo3::AsPyPointer;
use pyo3::PyObject;
use pyo3::Python;
use std::hash::Hash;
use std::os::raw::{c_int, c_void};

use crate::traits::node::*;

unsafe extern "C" fn visit(object: *mut pyo3::ffi::PyObject, arg: *mut c_void) -> c_int {
    let py = pyo3::Python::assume_gil_acquired();
    let pyobj = PyObject::from_borrowed_ptr(py, object);

    let result: &mut Vec<PyObject> = &mut *(arg as *mut Vec<PyObject>);
    result.push(pyobj);

    0
}

impl FindAdjacent for PyObject {
    fn find_adjacent(&self) -> Vec<PyObject> {
        let obj_ptr = self.as_ptr();
        let type_obj = unsafe { *(*obj_ptr).ob_type };
        let mut result = Vec::<PyObject>::new();
        if let Some(traverseproc) = type_obj.tp_traverse {
            let result_ptr: *mut c_void = &mut result as *mut _ as *mut c_void;
            unsafe {
                traverseproc(obj_ptr, visit, result_ptr);
            }
        }

        result
    }
}

pub struct PyObjectWrapper<'p> {
    obj: PyObject,
    py: Python<'p>,
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

impl<'p> FindAdjacent for PyObjectWrapper<'p> {
    fn find_adjacent(&self) -> Vec<PyObjectWrapper<'p>> {
        self.obj
            .find_adjacent()
            .into_iter()
            .map(|pyobject| PyObjectWrapper {
                obj: pyobject,
                py: self.py,
            })
            .collect()
    }
}
