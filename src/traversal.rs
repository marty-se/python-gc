use pyo3::AsPyPointer;
use pyo3::PyObject;
use pyo3::Python;
use std::os::raw::{c_int, c_void};
use std::vec::Vec;

pub trait FindAdjacent {
    fn find_adjacent(_py: Python, obj: Self) -> Vec<PyObject>;
}

unsafe extern "C" fn visit(object: *mut pyo3::ffi::PyObject, arg: *mut c_void) -> c_int {
    let py = pyo3::Python::assume_gil_acquired();
    let pyobj = PyObject::from_borrowed_ptr(py, object);

    let result: &mut Vec<PyObject> = &mut *(arg as *mut Vec<PyObject>);
    result.push(pyobj);

    0
}

impl FindAdjacent for PyObject {
    fn find_adjacent(_py: Python, obj: Self) -> Vec<PyObject> {
        let obj_ptr = obj.as_ptr();
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
