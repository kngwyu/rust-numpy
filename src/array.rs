//! Untyped safe interface for NumPy ndarray

use ndarray::*;
use npyffi;
use pyo3::*;

use std::os::raw::c_void;
use std::ptr::null_mut;

use super::error::ArrayCastError;
use super::*;

/// Untyped safe interface for NumPy ndarray.
pub struct PyArray(PyObject);
pyobject_native_type!(PyArray, *npyffi::PyArray_Type_Ptr, npyffi::PyArray_Check);

impl IntoPyObject for PyArray {
    fn into_object(self, _py: Python) -> PyObject {
        self.0
    }
}

impl PyArray {
    /// Get raw pointer for PyArrayObject
    pub fn as_array_ptr(&self) -> *mut npyffi::PyArrayObject {
        self.as_ptr() as _
    }

    /// Construct one-dimension PyArray from boxed slice.
    ///
    /// # Example
    /// ```
    /// # extern crate pyo3; extern crate numpy; fn main() {
    /// use numpy::{PyArray, PyArrayModule};
    /// let gil = pyo3::Python::acquire_gil();
    /// let np = PyArrayModule::import(gil.python()).unwrap();
    /// let slice = vec![1, 2, 3, 4, 5].into_boxed_slice();
    /// let pyarray = PyArray::from_boxed_slice::<u32>(gil.python(), &np, slice);
    /// assert_eq!(pyarray.as_slice::<u32>().unwrap(), &[1, 2, 3, 4, 5]);
    /// # }
    /// ```
    pub fn from_boxed_slice<T: TypeNum>(py: Python, np: &PyArrayModule, v: Box<[T]>) -> PyArray {
        IntoPyArray::into_pyarray(v, py, np)
    }

    /// Construct one-dimension PyArray from Vec.
    ///
    /// # Example
    /// ```
    /// # extern crate pyo3; extern crate numpy; fn main() {
    /// use numpy::{PyArray, PyArrayModule};
    /// let gil = pyo3::Python::acquire_gil();
    /// let np = PyArrayModule::import(gil.python()).unwrap();
    /// let pyarray = PyArray::from_vec::<u32>(gil.python(), &np, vec![1, 2, 3, 4, 5]);
    /// assert_eq!(pyarray.as_slice::<u32>().unwrap(), &[1, 2, 3, 4, 5]);
    /// # }
    /// ```
    pub fn from_vec<T: TypeNum>(py: Python, np: &PyArrayModule, v: Vec<T>) -> PyArray {
        IntoPyArray::into_pyarray(v, py, np)
    }

    /// Construct a two-dimension PyArray from `Vec<Vec<T>>`.
    ///
    /// This function checks all dimension of inner vec, and if there's any vec
    /// where its dimension differs from others, it returns `ArrayCastError`.
    ///
    /// # Example
    /// ```
    /// # extern crate pyo3; extern crate numpy; #[macro_use] extern crate ndarray; fn main() {
    /// use numpy::{PyArray, PyArrayModule};
    /// let gil = pyo3::Python::acquire_gil();
    /// let np = PyArrayModule::import(gil.python()).unwrap();
    /// let vec2 = vec![vec![1, 2, 3]; 2];
    /// let pyarray = PyArray::from_vec2::<u32>(gil.python(), &np, &vec2).unwrap();
    /// assert_eq!(pyarray.as_array::<u32>().unwrap(), array![[1, 2, 3], [1, 2, 3]].into_dyn());
    /// assert!(PyArray::from_vec2::<u32>(gil.python(), &np, &vec![vec![1], vec![2, 3]]).is_err());
    /// # }
    /// ```
    pub fn from_vec2<T: TypeNum>(
        py: Python,
        np: &PyArrayModule,
        v: &Vec<Vec<T>>,
    ) -> Result<PyArray, ArrayCastError> {
        let last_len = v.last().map_or(0, |v| v.len());
        if v.iter().any(|v| v.len() != last_len) {
            return Err(ArrayCastError::FromVec);
        }
        let dims = [v.len(), last_len];
        let flattend: Vec<_> = v.iter().cloned().flatten().collect();
        unsafe {
            let data = convert::into_raw(flattend);
            Ok(PyArray::new_::<T>(py, np, &dims, null_mut(), data))
        }
    }

    /// Construct a three-dimension PyArray from `Vec<Vec<Vec<T>>>`.
    ///
    /// This function checks all dimension of inner vec, and if there's any vec
    /// where its dimension differs from others, it returns `ArrayCastError`.
    ///
    /// # Example
    /// ```
    /// # extern crate pyo3; extern crate numpy; #[macro_use] extern crate ndarray; fn main() {
    /// use numpy::{PyArray, PyArrayModule};
    /// let gil = pyo3::Python::acquire_gil();
    /// let np = PyArrayModule::import(gil.python()).unwrap();
    /// let vec2 = vec![vec![vec![1, 2]; 2]; 2];
    /// let pyarray = PyArray::from_vec3::<u32>(gil.python(), &np, &vec2).unwrap();
    /// assert_eq!(
    ///     pyarray.as_array::<u32>().unwrap(),
    ///     array![[[1, 2], [1, 2]], [[1, 2], [1, 2]]].into_dyn()
    /// );
    /// assert!(PyArray::from_vec3::<u32>(gil.python(), &np, &vec![vec![vec![1], vec![]]]).is_err());
    /// # }
    /// ```
    pub fn from_vec3<T: TypeNum>(
        py: Python,
        np: &PyArrayModule,
        v: &Vec<Vec<Vec<T>>>,
    ) -> Result<PyArray, ArrayCastError> {
        let dim2 = v.last().map_or(0, |v| v.len());
        if v.iter().any(|v| v.len() != dim2) {
            return Err(ArrayCastError::FromVec);
        }
        let dim3 = v.last().map_or(0, |v| v.last().map_or(0, |v| v.len()));
        if v.iter().any(|v| v.iter().any(|v| v.len() != dim3)) {
            return Err(ArrayCastError::FromVec);
        }
        let dims = [v.len(), dim2, dim3];
        let flattend: Vec<_> = v.iter().flat_map(|v| v.iter().cloned().flatten()).collect();
        unsafe {
            let data = convert::into_raw(flattend);
            Ok(PyArray::new_::<T>(py, np, &dims, null_mut(), data))
        }
    }

    /// Construct PyArray from ndarray::Array.
    ///
    /// # Example
    /// ```
    /// # extern crate pyo3; extern crate numpy; #[macro_use] extern crate ndarray; fn main() {
    /// use numpy::{PyArray, PyArrayModule};
    /// let gil = pyo3::Python::acquire_gil();
    /// let np = PyArrayModule::import(gil.python()).unwrap();
    /// let pyarray = PyArray::from_ndarray::<u32, _>(gil.python(), &np, array![[1, 2], [3, 4]]);
    /// assert_eq!(pyarray.as_array::<u32>().unwrap(), array![[1, 2], [3, 4]].into_dyn());
    /// # }
    /// ```
    pub fn from_ndarray<A, D>(py: Python, np: &PyArrayModule, arr: Array<A, D>) -> PyArray
    where
        A: TypeNum,
        D: Dimension,
    {
        IntoPyArray::into_pyarray(arr, py, np)
    }

    pub unsafe fn from_owned_ptr(py: Python, ptr: *mut pyo3::ffi::PyObject) -> Self {
        let obj = PyObject::from_owned_ptr(py, ptr);
        PyArray(obj)
    }

    pub unsafe fn from_borrowed_ptr(py: Python, ptr: *mut pyo3::ffi::PyObject) -> Self {
        let obj = PyObject::from_borrowed_ptr(py, ptr);
        PyArray(obj)
    }

    /// Returns the number of dimensions in the array.
    ///
    /// Same as [numpy.ndarray.ndim](https://docs.scipy.org/doc/numpy/reference/generated/numpy.ndarray.ndim.html)
    ///
    /// # Example
    /// ```
    /// # extern crate pyo3; extern crate numpy; fn main() {
    /// use numpy::{PyArray, PyArrayModule};
    /// let gil = pyo3::Python::acquire_gil();
    /// let np = PyArrayModule::import(gil.python()).unwrap();
    /// let arr = PyArray::new::<f64>(gil.python(), &np, &[4, 5, 6]);
    /// assert_eq!(arr.ndim(), 3);
    /// # }
    /// ```
    // C API: https://docs.scipy.org/doc/numpy/reference/c-api.array.html#c.PyArray_NDIM
    pub fn ndim(&self) -> usize {
        let ptr = self.as_array_ptr();
        unsafe { (*ptr).nd as usize }
    }

    /// Same as [shape](./struct.PyArray.html#method.shape)
    ///
    /// Reserved for backward compatibility.
    #[inline]
    pub fn dims(&self) -> &[usize] {
        self.shape()
    }

    pub fn len(&self) -> usize {
        self.shape().iter().fold(1, |a, b| a * b)
    }

    /// Returns a slice which contains dimmensions of the array.
    ///
    /// Same as [numpy.ndarray.shape](https://docs.scipy.org/doc/numpy/reference/generated/numpy.ndarray.shape.html)
    /// # Example
    /// ```
    /// # extern crate pyo3; extern crate numpy; fn main() {
    /// use numpy::{PyArray, PyArrayModule};
    /// let gil = pyo3::Python::acquire_gil();
    /// let np = PyArrayModule::import(gil.python()).unwrap();
    /// let arr = PyArray::new::<f64>(gil.python(), &np, &[4, 5, 6]);
    /// assert_eq!(arr.shape(), &[4, 5, 6]);
    /// # }
    /// ```
    // C API: https://docs.scipy.org/doc/numpy/reference/c-api.array.html#c.PyArray_DIMS
    pub fn shape(&self) -> &[usize] {
        let n = self.ndim();
        let ptr = self.as_array_ptr();
        unsafe {
            let p = (*ptr).dimensions as *mut usize;
            ::std::slice::from_raw_parts(p, n)
        }
    }

    /// Returns a slice which contains how many bytes you need to jump to the next row.
    ///
    /// Same as [numpy.ndarray.strides](https://docs.scipy.org/doc/numpy/reference/generated/numpy.ndarray.strides.html)
    /// # Example
    /// ```
    /// # extern crate pyo3; extern crate numpy; fn main() {
    /// use numpy::{PyArray, PyArrayModule};
    /// let gil = pyo3::Python::acquire_gil();
    /// let np = PyArrayModule::import(gil.python()).unwrap();
    /// let arr = PyArray::new::<f64>(gil.python(), &np, &[4, 5, 6]);
    /// assert_eq!(arr.strides(), &[240, 48, 8]);
    /// # }
    /// ```
    // C API: https://docs.scipy.org/doc/numpy/reference/c-api.array.html#c.PyArray_STRIDES
    pub fn strides(&self) -> &[isize] {
        let n = self.ndim();
        let ptr = self.as_array_ptr();
        unsafe {
            let p = (*ptr).strides;
            ::std::slice::from_raw_parts(p, n)
        }
    }

    unsafe fn data<T>(&self) -> *mut T {
        let ptr = self.as_array_ptr();
        (*ptr).data as *mut T
    }

    fn ndarray_shape<A>(&self) -> StrideShape<IxDyn> {
        // FIXME may be done more simply
        let shape: Shape<_> = Dim(self.shape()).into();
        let st: Vec<usize> = self
            .strides()
            .iter()
            .map(|&x| x as usize / ::std::mem::size_of::<A>())
            .collect();
        shape.strides(Dim(st))
    }

    pub fn typenum(&self) -> i32 {
        unsafe {
            let descr = (*self.as_array_ptr()).descr;
            (*descr).type_num
        }
    }

    fn type_check<A: types::TypeNum>(&self) -> Result<(), ArrayCastError> {
        let test = A::typenum();
        let truth = self.typenum();
        if A::typenum() == self.typenum() {
            Ok(())
        } else {
            Err(ArrayCastError::to_rust(test, truth))
        }
    }

    /// Get data as a ndarray::ArrayView
    pub fn as_array<A: types::TypeNum>(&self) -> Result<ArrayViewD<A>, ArrayCastError> {
        self.type_check::<A>()?;
        unsafe {
            Ok(ArrayView::from_shape_ptr(
                self.ndarray_shape::<A>(),
                self.data(),
            ))
        }
    }

    /// Get data as a ndarray::ArrayViewMut
    pub fn as_array_mut<A: types::TypeNum>(&self) -> Result<ArrayViewMutD<A>, ArrayCastError> {
        self.type_check::<A>()?;
        unsafe {
            Ok(ArrayViewMut::from_shape_ptr(
                self.ndarray_shape::<A>(),
                self.data(),
            ))
        }
    }

    /// Get data as a Rust immutable slice
    pub fn as_slice<A: types::TypeNum>(&self) -> Result<&[A], ArrayCastError> {
        self.type_check::<A>()?;
        unsafe { Ok(::std::slice::from_raw_parts(self.data(), self.len())) }
    }

    /// Get data as a Rust mutable slice
    pub fn as_slice_mut<A: types::TypeNum>(&self) -> Result<&mut [A], ArrayCastError> {
        self.type_check::<A>()?;
        unsafe { Ok(::std::slice::from_raw_parts_mut(self.data(), self.len())) }
    }

    pub unsafe fn new_<T: types::TypeNum>(
        py: Python,
        np: &PyArrayModule,
        dims: &[usize],
        strides: *mut npy_intp,
        data: *mut c_void,
    ) -> Self {
        let dims: Vec<_> = dims.iter().map(|d| *d as npy_intp).collect();
        let ptr = np.PyArray_New(
            np.get_type_object(npyffi::ArrayType::PyArray_Type),
            dims.len() as i32,
            dims.as_ptr() as *mut npy_intp,
            T::typenum(),
            strides,
            data,
            0,                      // itemsize
            0,                      // flag
            ::std::ptr::null_mut(), //obj
        );
        Self::from_owned_ptr(py, ptr)
    }

    /// a wrapper of [PyArray_SimpleNew](https://docs.scipy.org/doc/numpy/reference/c-api.array.html#c.PyArray_SimpleNew)
    pub fn new<T: TypeNum>(py: Python, np: &PyArrayModule, dims: &[usize]) -> Self {
        unsafe { Self::new_::<T>(py, np, dims, null_mut(), null_mut()) }
    }

    /// a wrapper of [PyArray_ZEROS](https://docs.scipy.org/doc/numpy/reference/c-api.array.html#c.PyArray_ZEROS)
    pub fn zeros<T: TypeNum>(
        py: Python,
        np: &PyArrayModule,
        dims: &[usize],
        order: NPY_ORDER,
    ) -> Self {
        let dims: Vec<npy_intp> = dims.iter().map(|d| *d as npy_intp).collect();
        unsafe {
            let descr = np.PyArray_DescrFromType(T::typenum());
            let ptr = np.PyArray_Zeros(
                dims.len() as i32,
                dims.as_ptr() as *mut npy_intp,
                descr,
                order as i32,
            );
            Self::from_owned_ptr(py, ptr)
        }
    }

    /// a wrapper of [PyArray_Arange](https://docs.scipy.org/doc/numpy/reference/c-api.array.html#c.PyArray_Arange)
    pub fn arange<T: TypeNum>(
        py: Python,
        np: &PyArrayModule,
        start: f64,
        stop: f64,
        step: f64,
    ) -> Self {
        unsafe {
            let ptr = np.PyArray_Arange(start, stop, step, T::typenum());
            Self::from_owned_ptr(py, ptr)
        }
    }
}
