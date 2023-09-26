fn main() {
    use faiss::{Index, index_factory, MetricType};

    let mut index = index_factory(64, "Flat", MetricType::L2).unwrap();
    index.add(&[0f32;64]).unwrap();
    index.add(&[0f32;64]).unwrap();
    index.add(&[0f32;64]).unwrap();
    index.add(&[0f32;64]).unwrap();
    index.add(&[0f32;64]).unwrap();

    let result = index.search(&[0f32;64], 5).unwrap();
    for (i, (l, d)) in result.labels.iter()
        .zip(result.distances.iter())
        .enumerate()
    {
        println!("#{}: {} (D={})", i + 1, *l, *d);
    }
} 
