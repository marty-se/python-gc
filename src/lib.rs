mod structs;

#[cfg(test)]
mod tests {
    use crate::structs::pyobject::PyObjectWrapper;
    use crate::structs::refcounter::RefCounter;
    use pyo3::prelude::*;
    use pyo3::types::PyList;
    use scc::SCCCollector;

    #[test]
    fn test_collect_scc_and_count_internal_refs() -> Result<(), PyErr> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let empty_pylist_slice: &[PyList] = &[];

        let list = PyList::new(py, empty_pylist_slice);
        let list2 = PyList::new(py, &[list, list]);
        list.append(list2)?;

        let list_obj = list.to_object(py);
        let list2_obj = list2.to_object(py);

        // Drop GIL to get rid of outstanding refs from pyo3's release pool.
        drop(py);
        drop(gil);

        let gil2 = Python::acquire_gil();
        let py2 = gil2.python();

        assert_eq!(list_obj.get_refcnt(), 3);
        assert_eq!(list2_obj.get_refcnt(), 2);

        let list2_wrapper = PyObjectWrapper::new(py2, list2_obj);

        let scc_collector = SCCCollector::new(list2_wrapper);
        let result: Vec<PyObjectWrapper> = scc_collector.iter().flatten().collect();

        let counted_refs = RefCounter::count_internal_refs(result);
        assert_eq!(counted_refs, vec![2, 1]);
        Ok(())
    }
}
