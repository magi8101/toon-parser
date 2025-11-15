use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

// Initialize Python interpreter once
fn with_python<F, R>(f: F) -> R
where
    F: FnOnce(Python) -> R,
{
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| f(py))
}

fn bench_type_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_checking");
    
    with_python(|py| {
        // Create test objects
        let int_obj = 42i64.into_pyobject(py).unwrap();
        let str_obj = "hello".into_pyobject(py).unwrap();
        let bool_obj = true.into_pyobject(py).unwrap();
        let list_obj = PyList::new(py, vec![1, 2, 3]).unwrap();
        let dict_obj = PyDict::new(py);
        dict_obj.set_item("key", "value").unwrap();
        
        group.bench_function("is_instance_of_int", |b| {
            b.iter(|| {
                black_box(int_obj.is_instance_of::<pyo3::types::PyInt>())
            })
        });
        
        group.bench_function("is_instance_of_str", |b| {
            b.iter(|| {
                black_box(str_obj.is_instance_of::<pyo3::types::PyString>())
            })
        });
        
        group.bench_function("is_instance_of_bool", |b| {
            b.iter(|| {
                black_box(bool_obj.is_instance_of::<pyo3::types::PyBool>())
            })
        });
        
        group.bench_function("is_instance_of_list", |b| {
            b.iter(|| {
                black_box(list_obj.is_instance_of::<pyo3::types::PyList>())
            })
        });
        
        group.bench_function("is_instance_of_dict", |b| {
            b.iter(|| {
                black_box(dict_obj.is_instance_of::<pyo3::types::PyDict>())
            })
        });
    });
    
    group.finish();
}

fn bench_dict_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("dict_iteration");
    
    with_python(|py| {
        // Create dicts of different sizes
        for size in [10, 100, 1000] {
            let dict = PyDict::new(py);
            for i in 0..size {
                dict.set_item(format!("key{}", i), i).unwrap();
            }
            
            group.bench_with_input(BenchmarkId::new("items_iter", size), &dict, |b, d| {
                b.iter(|| {
                    for (key, value) in d.iter() {
                        black_box((key, value));
                    }
                })
            });
        }
    });
    
    group.finish();
}

fn bench_list_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_creation");
    
    with_python(|py| {
        for size in [10, 100, 1000] {
            // Method 1: PyList::new with Vec
            group.bench_with_input(BenchmarkId::new("new_from_vec", size), &size, |b, &s| {
                b.iter(|| {
                    let vec: Vec<i64> = (0..s).collect();
                    black_box(PyList::new(py, vec).unwrap())
                })
            });
            
            // Method 2: PyList::empty + append
            group.bench_with_input(BenchmarkId::new("empty_then_append", size), &size, |b, &s| {
                b.iter(|| {
                    let list = PyList::empty(py);
                    for i in 0..s {
                        list.append(i).unwrap();
                    }
                    black_box(list)
                })
            });
        }
    });
    
    group.finish();
}

fn bench_dict_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("dict_creation");
    
    with_python(|py| {
        for size in [10, 100, 1000] {
            group.bench_with_input(BenchmarkId::new("new_then_set", size), &size, |b, &s| {
                b.iter(|| {
                    let dict = PyDict::new(py);
                    for i in 0..s {
                        dict.set_item(format!("key{}", i), i).unwrap();
                    }
                    black_box(dict)
                })
            });
        }
    });
    
    group.finish();
}

fn bench_string_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_extraction");
    
    with_python(|py| {
        let short_str = "hello".into_pyobject(py).unwrap();
        let medium_str = "hello world this is a medium string".into_pyobject(py).unwrap();
        let long_str = "hello world this is a much longer string with lots of text that goes on and on".into_pyobject(py).unwrap();
        
        group.bench_function("short_str", |b| {
            b.iter(|| {
                let s: String = short_str.extract().unwrap();
                black_box(s)
            })
        });
        
        group.bench_function("medium_str", |b| {
            b.iter(|| {
                let s: String = medium_str.extract().unwrap();
                black_box(s)
            })
        });
        
        group.bench_function("long_str", |b| {
            b.iter(|| {
                let s: String = long_str.extract().unwrap();
                black_box(s)
            })
        });
    });
    
    group.finish();
}

fn bench_number_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("number_extraction");
    
    with_python(|py| {
        let int_obj = 42i64.into_pyobject(py).unwrap();
        let float_obj = 3.14f64.into_pyobject(py).unwrap();
        
        group.bench_function("extract_int", |b| {
            b.iter(|| {
                let n: i64 = int_obj.extract().unwrap();
                black_box(n)
            })
        });
        
        group.bench_function("extract_float", |b| {
            b.iter(|| {
                let n: f64 = float_obj.extract().unwrap();
                black_box(n)
            })
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_type_checking,
    bench_dict_iteration,
    bench_list_creation,
    bench_dict_creation,
    bench_string_extraction,
    bench_number_extraction
);
criterion_main!(benches);
