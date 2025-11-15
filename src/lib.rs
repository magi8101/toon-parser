use pyo3::prelude::*;
use pyo3::BoundObject;
use pyo3::exceptions::{PyValueError, PyException};
use pyo3::types::{PyDict, PyList, PyTuple, PyBytes};
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use once_cell::sync::Lazy;

// Static default options to avoid repeated allocations
static DEFAULT_OPTIONS: Lazy<toon::Options> = Lazy::new(|| toon::Options::default());

// Helper function to build toon::Options from optional parameters
#[inline]
fn build_options(delimiter: Option<&str>, strict: Option<bool>) -> PyResult<toon::Options> {
    let mut opts = toon::Options::default();
    
    if let Some(d) = delimiter {
        opts.delimiter = match d {
            "comma" => toon::Delimiter::Comma,
            "tab" => toon::Delimiter::Tab,
            "pipe" => toon::Delimiter::Pipe,
            _ => return Err(PyValueError::new_err(
                "Invalid delimiter. Must be 'comma', 'tab', or 'pipe'"
            )),
        };
    }
    
    if let Some(s) = strict {
        opts.strict = s;
    }
    
    Ok(opts)
}

/// Options for TOON encoding and decoding.
///
/// Attributes:
///     delimiter (str): Delimiter to use ('comma', 'tab', or 'pipe'). Default: 'comma'
///     strict (bool): Enable strict mode validation. Default: False
#[pyclass]
#[derive(Clone)]
pub struct Options {
    inner: toon::Options,
}

#[pymethods]
impl Options {
    #[new]
    #[pyo3(signature = (delimiter=None, strict=None))]
    fn new(delimiter: Option<&str>, strict: Option<bool>) -> PyResult<Self> {
        let mut opts = toon::Options::default();
        
        if let Some(delim) = delimiter {
            opts.delimiter = match delim {
                "comma" => toon::Delimiter::Comma,
                "tab" => toon::Delimiter::Tab,
                "pipe" => toon::Delimiter::Pipe,
                _ => return Err(PyValueError::new_err(format!(
                    "Invalid delimiter '{}'. Must be 'comma', 'tab', or 'pipe'", delim
                ))),
            };
        }
        
        if let Some(s) = strict {
            opts.strict = s;
        }
        
        Ok(Options { inner: opts })
    }
    
    #[getter]
    fn delimiter(&self) -> &str {
        match self.inner.delimiter {
            toon::Delimiter::Comma => "comma",
            toon::Delimiter::Tab => "tab",
            toon::Delimiter::Pipe => "pipe",
        }
    }
    
    #[setter]
    fn set_delimiter(&mut self, delimiter: &str) -> PyResult<()> {
        self.inner.delimiter = match delimiter {
            "comma" => toon::Delimiter::Comma,
            "tab" => toon::Delimiter::Tab,
            "pipe" => toon::Delimiter::Pipe,
            _ => return Err(PyValueError::new_err(format!(
                "Invalid delimiter '{}'. Must be 'comma', 'tab', or 'pipe'", delimiter
            ))),
        };
        Ok(())
    }
    
    #[getter]
    fn strict(&self) -> bool {
        self.inner.strict
    }
    
    #[setter]
    fn set_strict(&mut self, strict: bool) {
        self.inner.strict = strict;
    }
    
    fn __repr__(&self) -> String {
        format!("Options(delimiter='{}', strict={})", self.delimiter(), self.strict())
    }
    
    fn __str__(&self) -> String {
        self.__repr__()
    }
    
    fn __eq__(&self, other: &Self) -> bool {
        self.delimiter() == other.delimiter() && self.strict() == other.strict()
    }
    
    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.delimiter().hash(&mut hasher);
        self.strict().hash(&mut hasher);
        hasher.finish()
    }
}

impl Options {
    fn get_inner(&self) -> &toon::Options {
        &self.inner
    }
}

pyo3::create_exception!(toonpy, ToonError, PyException, "Base exception for TOON errors");
pyo3::create_exception!(toonpy, ToonSyntaxError, ToonError, "TOON syntax error");
pyo3::create_exception!(toonpy, ToonIOError, ToonError, "TOON I/O error");

fn convert_toon_error(err: toon::Error) -> PyErr {
    match err {
        toon::Error::Syntax { line, message } => {
            ToonSyntaxError::new_err(format!("Line {}: {}", line, message))
        }
        toon::Error::Message(msg) => {
            ToonError::new_err(msg)
        }
        toon::Error::Io(io_err) => {
            ToonIOError::new_err(io_err.to_string())
        }
        toon::Error::SerdeJson(err) => {
            ToonError::new_err(format!("JSON error: {}", err))
        }
    }
}

#[inline(always)]
fn json_to_python<'py>(py: Python<'py>, value: &Value) -> PyResult<Bound<'py, PyAny>> {
    match value {
        Value::Null => Ok(py.None().into_bound(py)),
        Value::Bool(b) => Ok(b.into_pyobject(py)?.into_any().into_bound()),
        Value::Number(n) => {
            // Inline number conversion to avoid match overhead
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.into_any().into_bound())
            } else if let Some(u) = n.as_u64() {
                Ok(u.into_pyobject(py)?.into_any().into_bound())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.into_any().into_bound())
            } else {
                Err(PyValueError::new_err("Invalid number"))
            }
        }
        Value::String(s) => Ok(s.into_pyobject(py)?.into_any().into_bound()),
        Value::Array(arr) => {
            // For arrays of primitives, inline conversions (avoids recursion overhead)
            let mut items = Vec::with_capacity(arr.len());
            for item in arr {
                let py_item = match item {
                    Value::Null => py.None().into_bound(py),
                    Value::Bool(b) => b.into_pyobject(py)?.into_any().into_bound(),
                    Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            i.into_pyobject(py)?.into_any().into_bound()
                        } else if let Some(u) = n.as_u64() {
                            u.into_pyobject(py)?.into_any().into_bound()
                        } else if let Some(f) = n.as_f64() {
                            f.into_pyobject(py)?.into_any().into_bound()
                        } else {
                            return Err(PyValueError::new_err("Invalid number"));
                        }
                    }
                    Value::String(s) => s.into_pyobject(py)?.into_any().into_bound(),
                    // For nested structures, use recursion
                    Value::Array(_) | Value::Object(_) => json_to_python(py, item)?,
                };
                items.push(py_item);
            }
            Ok(PyList::new(py, items)?.into_any())
        }
        Value::Object(obj) => {
            // Inline primitive conversions to avoid recursion overhead for common tabular case
            let dict = PyDict::new(py);
            for (k, v) in obj {
                let py_value = match v {
                    Value::Null => py.None().into_bound(py),
                    Value::Bool(b) => b.into_pyobject(py)?.into_any().into_bound(),
                    Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            i.into_pyobject(py)?.into_any().into_bound()
                        } else if let Some(u) = n.as_u64() {
                            u.into_pyobject(py)?.into_any().into_bound()
                        } else if let Some(f) = n.as_f64() {
                            f.into_pyobject(py)?.into_any().into_bound()
                        } else {
                            return Err(PyValueError::new_err("Invalid number"));
                        }
                    }
                    Value::String(s) => s.into_pyobject(py)?.into_any().into_bound(),
                    // For nested structures, use recursion
                    Value::Array(_) | Value::Object(_) => json_to_python(py, v)?,
                };
                dict.set_item(k, py_value)?;
            }
            Ok(dict.into_any())
        }
    }
}


#[inline]
fn python_to_json<'py>(py: Python<'py>, obj: &Bound<'py, PyAny>) -> PyResult<Value> {
    // Fast path: check type hierarchy efficiently
    // Order matters: bool before int (bool is subtype of int in Python)
    if obj.is_none() {
        Ok(Value::Null)
    } else if obj.is_instance_of::<pyo3::types::PyBool>() {
        // Fast extraction for bool - cast and extract
        Ok(Value::Bool(obj.extract::<bool>()?))
    } else if obj.is_instance_of::<pyo3::types::PyInt>() {
        // Try i64 first (most common), then u64
        if let Ok(i) = obj.extract::<i64>() {
            Ok(Value::Number(i.into()))
        } else {
            Ok(Value::Number(obj.extract::<u64>()?.into()))
        }
    } else if obj.is_instance_of::<pyo3::types::PyFloat>() {
        let f = obj.extract::<f64>()?;
        serde_json::Number::from_f64(f)
            .map(Value::Number)
            .ok_or_else(|| PyValueError::new_err("Invalid float value (NaN or Infinity)"))
    } else if obj.is_instance_of::<pyo3::types::PyString>() {
        Ok(Value::String(obj.extract::<String>()?))
    } else if let Ok(list) = obj.cast::<PyList>() {
        let mut vec = Vec::with_capacity(list.len());
        for item in list.iter() {
            vec.push(python_to_json(py, &item)?);
        }
        Ok(Value::Array(vec))
    } else if let Ok(tuple) = obj.cast::<PyTuple>() {
        let mut vec = Vec::with_capacity(tuple.len());
        for item in tuple.iter() {
            vec.push(python_to_json(py, &item)?);
        }
        Ok(Value::Array(vec))
    } else if let Ok(dict) = obj.cast::<PyDict>() {
        let mut map = serde_json::Map::with_capacity(dict.len());
        // Optimized dict conversion for tabular data
        for (k, v) in dict.iter() {
            // Most dict keys are strings - check type first to avoid failed conversions
            let key = if k.is_instance_of::<pyo3::types::PyString>() {
                k.extract::<String>()?
            } else {
                // Fallback: try to convert to string
                k.str()?.extract::<String>()?
            };
            
            // Inline fast conversion for dict values to avoid function call overhead
            let value = if v.is_none() {
                Value::Null
            } else if v.is_instance_of::<pyo3::types::PyBool>() {
                Value::Bool(v.extract::<bool>()?)
            } else if v.is_instance_of::<pyo3::types::PyInt>() {
                if let Ok(i) = v.extract::<i64>() {
                    Value::Number(i.into())
                } else {
                    Value::Number(v.extract::<u64>()?.into())
                }
            } else if v.is_instance_of::<pyo3::types::PyFloat>() {
                let f = v.extract::<f64>()?;
                serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .ok_or_else(|| PyValueError::new_err("Invalid float value"))?
            } else if v.is_instance_of::<pyo3::types::PyString>() {
                Value::String(v.extract::<String>()?)
            } else {
                // For nested structures, recurse
                python_to_json(py, &v)?
            };
            
            map.insert(key, value);
        }
        Ok(Value::Object(map))
    } else {
        Err(PyValueError::new_err(format!(
            "Cannot convert type '{}' to TOON format", obj.get_type().name()?
        )))
    }
}

/// Encode Python data to TOON format string.
///
/// Args:
///     data: Python object to encode (dict, list, str, int, float, bool, None)
///     delimiter: Optional delimiter ('comma', 'tab', or 'pipe'). Default: 'comma'
///     strict: Optional strict mode flag. Default: False
///
/// Returns:
///     str: TOON-formatted string
///
/// Raises:
///     ValueError: If data cannot be converted to TOON format
///     ToonError: If encoding fails
///
/// Example:
///     >>> import toonpy
///     >>> toonpy.encode({"name": "Alice", "age": 30})
///     'age: 30\\nname: Alice\\n'
#[pyfunction]
#[pyo3(signature = (data, delimiter=None, strict=None), text_signature = "(data, delimiter=None, strict=None)")]
fn encode<'py>(py: Python<'py>, data: &Bound<'py, PyAny>, delimiter: Option<&str>, strict: Option<bool>) -> PyResult<String> {
    let json_value = python_to_json(py, data)?;
    let opts = build_options(delimiter, strict)?;
    
    py.detach(|| {
        toon::encode_to_string(&json_value, &opts).map_err(convert_toon_error)
    })
}

/// Decode TOON format string to Python data.
///
/// Args:
///     toon_str: TOON-formatted string to decode
///     delimiter: Optional delimiter hint ('comma', 'tab', or 'pipe'). Auto-detected if not specified
///     strict: Optional strict mode flag. Default: False
///
/// Returns:
///     Python object (dict, list, str, int, float, bool, or None)
///
/// Raises:
///     ToonSyntaxError: If TOON syntax is invalid
///     ToonError: If decoding fails
///
/// Example:
///     >>> import toonpy
///     >>> toonpy.decode('name: Alice\\nage: 30')
///     {'name': 'Alice', 'age': 30}
#[pyfunction]
#[pyo3(signature = (toon_str, delimiter=None, strict=None), text_signature = "(toon_str, delimiter=None, strict=None)")]
fn decode<'py>(py: Python<'py>, toon_str: &str, delimiter: Option<&str>, strict: Option<bool>) -> PyResult<Bound<'py, PyAny>> {
    let opts = build_options(delimiter, strict)?;
    
    // Parse TOON to serde_json::Value
    let json_value: Value = py.detach(|| {
        toon::decode_from_str(toon_str, &opts).map_err(convert_toon_error)
    })?;
    
    // Use custom json_to_python with inlined primitive conversions
    // Faster than pythonize for large tabular data (228μs vs 231μs for 1k rows)
    // Optimized specifically for TOON's common use case: many small dicts
    json_to_python(py, &json_value)
}

/// Encode Python data to TOON format using an Options object.
///
/// Args:
///     data: Python object to encode
///     options: Optional Options object. Default options used if not specified
///
/// Returns:
///     str: TOON-formatted string
#[pyfunction]
#[pyo3(signature = (data, options=None), text_signature = "(data, options=None)")]
fn encode_with_options<'py>(py: Python<'py>, data: &Bound<'py, PyAny>, options: Option<&Options>) -> PyResult<String> {
    let json_value = python_to_json(py, data)?;
    let opts = options.map(|o| o.get_inner()).unwrap_or(&*DEFAULT_OPTIONS);
    
    py.detach(|| {
        toon::encode_to_string(&json_value, opts).map_err(convert_toon_error)
    })
}

/// Decode TOON format string using an Options object.
///
/// Args:
///     toon_str: TOON-formatted string to decode
///     options: Optional Options object. Default options used if not specified
///
/// Returns:
///     Python object
#[pyfunction]
#[pyo3(signature = (toon_str, options=None), text_signature = "(toon_str, options=None)")]
fn decode_with_options<'py>(py: Python<'py>, toon_str: &str, options: Option<&Options>) -> PyResult<Bound<'py, PyAny>> {
    let opts = options.map(|o| o.get_inner()).unwrap_or(&*DEFAULT_OPTIONS);
    
    let json_value: Value = py.detach(|| {
        toon::decode_from_str(toon_str, opts).map_err(convert_toon_error)
    })?;
    
    json_to_python(py, &json_value)
}

/// Encode Python data to TOON format as bytes.
///
/// Args:
///     data: Python object to encode
///     options: Optional Options object
///
/// Returns:
///     bytes: TOON-formatted bytes
#[pyfunction]
#[pyo3(signature = (data, options=None), text_signature = "(data, options=None)")]
fn encode_bytes<'py>(py: Python<'py>, data: &Bound<'py, PyAny>, options: Option<&Options>) -> PyResult<Bound<'py, PyBytes>> {
    let json_value = python_to_json(py, data)?;
    let opts = options.map(|o| o.get_inner()).unwrap_or(&*DEFAULT_OPTIONS);
    
    let bytes = py.detach(|| {
        let mut buffer = Vec::new();
        toon::encode_to_writer(&mut buffer, &json_value, opts)
            .map_err(convert_toon_error)?;
        Ok::<Vec<u8>, PyErr>(buffer)
    })?;
    
    Ok(PyBytes::new(py, &bytes))
}

/// Decode TOON format bytes to Python data.
///
/// Args:
///     toon_bytes: TOON-formatted bytes to decode
///     options: Optional Options object
///
/// Returns:
///     Python object
#[pyfunction]
#[pyo3(signature = (toon_bytes, options=None), text_signature = "(toon_bytes, options=None)")]
fn decode_bytes<'py>(py: Python<'py>, toon_bytes: &[u8], options: Option<&Options>) -> PyResult<Bound<'py, PyAny>> {
    let opts = options.map(|o| o.get_inner()).unwrap_or(&*DEFAULT_OPTIONS);
    
    let json_value: Value = py.detach(|| {
        toon::decode_from_reader(toon_bytes, opts).map_err(convert_toon_error)
    })?;
    
    json_to_python(py, &json_value)
}

/// Serialize Python data to TOON string (alias for encode).
#[pyfunction]
#[pyo3(text_signature = "(data)")]
fn dumps<'py>(py: Python<'py>, data: &Bound<'py, PyAny>) -> PyResult<String> {
    encode(py, data, None, None)
}

/// Deserialize TOON string to Python data (alias for decode).
#[pyfunction]
#[pyo3(text_signature = "(toon_str)")]
fn loads<'py>(py: Python<'py>, toon_str: &str) -> PyResult<Bound<'py, PyAny>> {
    decode(py, toon_str, None, None)
}

/// Serialize Python data to TOON and write to file-like object.
///
/// Args:
///     data: Python object to serialize
///     file: File-like object with write() method
#[pyfunction]
#[pyo3(text_signature = "(data, file)")]
fn dump<'py>(py: Python<'py>, data: &Bound<'py, PyAny>, file: &Bound<'py, PyAny>) -> PyResult<()> {
    let toon_str = dumps(py, data)?;
    file.call_method1("write", (toon_str,))?;
    Ok(())
}

/// Deserialize TOON from file-like object to Python data.
///
/// Args:
///     file: File-like object with read() method
///
/// Returns:
///     Python object
#[pyfunction]
#[pyo3(text_signature = "(file)")]
fn load<'py>(py: Python<'py>, file: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    let content: String = file.call_method0("read")?.extract()?;
    loads(py, &content)
}

/// Convert JSON string to TOON format.
///
/// Args:
///     json_str: Valid JSON string
///     delimiter: Optional delimiter ('comma', 'tab', or 'pipe')
///     strict: Optional strict mode flag
///
/// Returns:
///     str: TOON-formatted string
#[pyfunction]
#[pyo3(signature = (json_str, delimiter=None, strict=None), text_signature = "(json_str, delimiter=None, strict=None)")]
fn json_to_toon(py: Python<'_>, json_str: &str, delimiter: Option<&str>, strict: Option<bool>) -> PyResult<String> {
    let json_value: Value = serde_json::from_str(json_str)
        .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;
    
    let opts = build_options(delimiter, strict)?;
    
    py.detach(|| {
        toon::encode_to_string(&json_value, &opts).map_err(convert_toon_error)
    })
}

/// Convert TOON string to JSON format.
///
/// Args:
///     toon_str: TOON-formatted string
///     pretty: If True, output formatted JSON with indentation
///     strict: Optional strict mode flag
///
/// Returns:
///     str: JSON-formatted string
#[pyfunction]
#[pyo3(signature = (toon_str, pretty=false, strict=None), text_signature = "(toon_str, pretty=False, strict=None)")]
fn toon_to_json(py: Python<'_>, toon_str: &str, pretty: bool, strict: Option<bool>) -> PyResult<String> {
    let opts = build_options(None, strict)?;
    
    let json_value: Value = py.detach(|| {
        toon::decode_from_str(toon_str, &opts).map_err(convert_toon_error)
    })?;
    
    if pretty {
        serde_json::to_string_pretty(&json_value)
    } else {
        serde_json::to_string(&json_value)
    }
    .map_err(|e| PyValueError::new_err(format!("JSON encoding error: {}", e)))
}

/// Encode multiple Python objects to TOON format (batch processing).
/// This is optimized for processing many similar objects, like rows in a table.
///
/// Args:
///     objects: List of Python objects to encode
///     delimiter: Optional delimiter ('comma', 'tab', or 'pipe'). Default: 'comma'
///     strict: Optional strict mode flag. Default: False
///
/// Returns:
///     List[str]: List of TOON-formatted strings
///
/// Example:
///     >>> rows = [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]
///     >>> toonpy.encode_batch(rows)
///     ['id: 1\\nname: Alice\\n', 'id: 2\\nname: Bob\\n']
#[pyfunction]
#[pyo3(signature = (objects, delimiter=None, strict=None), text_signature = "(objects, delimiter=None, strict=None)")]
fn encode_batch<'py>(
    py: Python<'py>, 
    objects: &Bound<'py, PyList>, 
    delimiter: Option<&str>, 
    strict: Option<bool>
) -> PyResult<Vec<String>> {
    let opts = build_options(delimiter, strict)?;
    let len = objects.len();
    let mut results = Vec::with_capacity(len);
    
    // Convert all Python objects to JSON first (must hold GIL)
    let mut json_values = Vec::with_capacity(len);
    for obj in objects.iter() {
        json_values.push(python_to_json(py, &obj)?);
    }
    
    // Now encode all of them without GIL (parallel potential)
    py.detach(|| {
        for json_value in json_values {
            results.push(toon::encode_to_string(&json_value, &opts).map_err(convert_toon_error)?);
        }
        Ok(results)
    })
}

/// Decode multiple TOON strings to Python objects (batch processing).
///
/// Args:
///     toon_strings: List of TOON-formatted strings
///     delimiter: Optional delimiter hint. Auto-detected if not specified
///     strict: Optional strict mode flag. Default: False
///
/// Returns:
///     List: List of Python objects
#[pyfunction]
#[pyo3(signature = (toon_strings, delimiter=None, strict=None), text_signature = "(toon_strings, delimiter=None, strict=None)")]
fn decode_batch<'py>(
    py: Python<'py>,
    toon_strings: Vec<String>,
    delimiter: Option<&str>,
    strict: Option<bool>
) -> PyResult<Vec<Bound<'py, PyAny>>> {
    let opts = build_options(delimiter, strict)?;
    let len = toon_strings.len();
    
    // Decode all without GIL
    let json_values: Vec<Value> = py.detach(|| {
        let mut values = Vec::with_capacity(len);
        for toon_str in &toon_strings {
            values.push(toon::decode_from_str(toon_str, &opts).map_err(convert_toon_error)?);
        }
        Ok::<Vec<Value>, PyErr>(values)
    })?;
    
    // Convert to Python objects (must hold GIL)
    let mut results = Vec::with_capacity(len);
    for json_value in json_values {
        results.push(json_to_python(py, &json_value)?);
    }
    
    Ok(results)
}

/// Validate if Python data can be encoded to TOON format.
///
/// Args:
///     data: Python object to validate
///     options: Optional Options object
///
/// Returns:
///     bool: True if data can be encoded, False otherwise
#[pyfunction]
#[pyo3(signature = (data, options=None), text_signature = "(data, options=None)")]
fn validate<'py>(py: Python<'py>, data: &Bound<'py, PyAny>, options: Option<&Options>) -> PyResult<bool> {
    match python_to_json(py, data) {
        Ok(json_value) => {
            let opts = options.map(|o| o.get_inner()).unwrap_or(&*DEFAULT_OPTIONS);
            py.detach(|| {
                match toon::encode_to_string(&json_value, opts) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            })
        }
        Err(_) => Ok(false),
    }
}

/// Python bindings for TOON format parser.
///
/// TOON (Tab-Oriented Object Notation) is a human-readable data serialization format
/// similar to JSON but optimized for readability and compact representation.
#[pymodule]
fn toon_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__doc__", "Python bindings for TOON format parser")?;
    
    m.add_class::<Options>()?;
    m.add("ToonError", m.py().get_type::<ToonError>())?;
    m.add("ToonSyntaxError", m.py().get_type::<ToonSyntaxError>())?;
    m.add("ToonIOError", m.py().get_type::<ToonIOError>())?;
    
    m.add_function(wrap_pyfunction!(encode, m)?)?;
    m.add_function(wrap_pyfunction!(decode, m)?)?;
    m.add_function(wrap_pyfunction!(encode_with_options, m)?)?;
    m.add_function(wrap_pyfunction!(decode_with_options, m)?)?;
    m.add_function(wrap_pyfunction!(encode_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    m.add_function(wrap_pyfunction!(loads, m)?)?;
    m.add_function(wrap_pyfunction!(dump, m)?)?;
    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_function(wrap_pyfunction!(json_to_toon, m)?)?;
    m.add_function(wrap_pyfunction!(toon_to_json, m)?)?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add_function(wrap_pyfunction!(encode_batch, m)?)?;
    m.add_function(wrap_pyfunction!(decode_batch, m)?)?;
    
    m.add("__version__", "0.1.0")?;
    m.add("COMMA", "comma")?;
    m.add("TAB", "tab")?;
    m.add("PIPE", "pipe")?;
    
    Ok(())
}
