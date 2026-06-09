use numpy::ndarray::Array2;

pub trait ChunkSource {
    type Chunks: Iterator<Item = Array2<i16>>;

    fn n_channels(&self) -> usize;
    fn n_samples(&self) -> usize;
    fn sample_rate(&self) -> f64;
    fn chunks(&self, chunk_samples: usize) -> Self::Chunks;
}
