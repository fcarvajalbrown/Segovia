use std::collections::VecDeque;
use std::sync::mpsc::{sync_channel, Receiver};
use std::sync::Mutex;

use numpy::ndarray::{concatenate, s, Array2, ArrayView2, Axis};
use rayon::prelude::*;

use crate::dsp::cmr::common_median_reference;
use crate::dsp::filter::{self, Section};
use crate::dsp::whiten::Whitener;

pub struct ChainParams {
    pub sos: Vec<Section>,
    pub padlen: usize,
    pub margin: usize,
    pub batch: usize,
    pub eps: f64,
    pub apply_mean: bool,
    pub calib_samples: usize,
    pub whiten: bool,
}

type ChunkStream = Box<dyn Iterator<Item = Array2<i16>> + Send + Sync>;

struct Prefetch {
    rx: Mutex<Receiver<Array2<i16>>>,
}

impl Prefetch {
    fn new(mut inner: ChunkStream, depth: usize) -> Self {
        let (tx, rx) = sync_channel::<Array2<i16>>(depth.max(1));
        std::thread::spawn(move || {
            while let Some(chunk) = inner.next() {
                if tx.send(chunk).is_err() {
                    break;
                }
            }
        });
        Self { rx: Mutex::new(rx) }
    }
}

impl Iterator for Prefetch {
    type Item = Array2<i16>;

    fn next(&mut self) -> Option<Array2<i16>> {
        self.rx.get_mut().expect("prefetch mutex").recv().ok()
    }
}

fn filter_to_f32(
    slab: &Array2<f32>,
    valid_start: usize,
    valid_len: usize,
    sos: &[Section],
    padlen: usize,
) -> Array2<f32> {
    let n = slab.nrows();
    let c = slab.ncols();
    let pad = padlen.min(n.saturating_sub(1));

    let mut filtered = Array2::<f32>::zeros((valid_len, c));
    let mut col = vec![0.0f64; n];
    for ch in 0..c {
        for (i, slot) in col.iter_mut().enumerate() {
            *slot = slab[[i, ch]] as f64;
        }
        let y = filter::sosfiltfilt(sos, &col, pad);
        for i in 0..valid_len {
            filtered[[i, ch]] = y[valid_start + i] as f32;
        }
    }
    filtered
}

fn process_slab(
    slab: &Array2<f32>,
    valid_start: usize,
    valid_len: usize,
    sos: &[Section],
    padlen: usize,
    whitener: Option<&Whitener>,
) -> Array2<f32> {
    let c = slab.ncols();
    let mut filtered = filter_to_f32(slab, valid_start, valid_len, sos, padlen);

    let mut scratch = Vec::with_capacity(c);
    common_median_reference(&mut filtered, &mut scratch);

    match whitener {
        Some(w) => w.apply(&filtered),
        None => filtered,
    }
}

fn read_calibration(src: &mut ChunkStream, calib_samples: usize) -> Array2<f32> {
    let mut blocks: Vec<Array2<f32>> = Vec::new();
    let mut total = 0usize;
    while total < calib_samples {
        match src.next() {
            Some(b) => {
                total += b.nrows();
                blocks.push(b.mapv(|v| v as f32));
            }
            None => break,
        }
    }
    let views: Vec<ArrayView2<f32>> = blocks.iter().map(|b| b.view()).collect();
    let joined = concatenate(Axis(0), &views).expect("calibration blocks share width");
    let take = calib_samples.min(joined.nrows());
    joined.slice(s![..take, ..]).to_owned()
}

pub struct Pipeline {
    stream: ChunkStream,
    params: ChainParams,
    whitener: Option<Whitener>,
    prev_tail: Option<Array2<f32>>,
    cur: Option<Array2<f32>>,
    primed: bool,
    out_queue: VecDeque<Array2<f32>>,
    done: bool,
}

impl Pipeline {
    pub fn new(mut calib: ChunkStream, stream: ChunkStream, params: ChainParams) -> Self {
        let whitener = if params.whiten {
            let calib_block = read_calibration(&mut calib, params.calib_samples);
            let rows = calib_block.nrows();
            let c = calib_block.ncols();

            let mut filtered = filter_to_f32(&calib_block, 0, rows, &params.sos, params.padlen);
            let mut scratch = Vec::with_capacity(c);
            common_median_reference(&mut filtered, &mut scratch);

            Some(Whitener::estimate(&filtered, params.eps, params.apply_mean))
        } else {
            None
        };

        let depth = (params.batch * 2).max(2);
        let stream: ChunkStream = Box::new(Prefetch::new(stream, depth));

        Self {
            stream,
            params,
            whitener,
            prev_tail: None,
            cur: None,
            primed: false,
            out_queue: VecDeque::new(),
            done: false,
        }
    }

    fn read_f32(&mut self) -> Option<Array2<f32>> {
        self.stream.next().map(|b| b.mapv(|v| v as f32))
    }

    fn next_slab(&mut self) -> Option<(Array2<f32>, usize, usize)> {
        if !self.primed {
            self.cur = self.read_f32();
            self.primed = true;
        }
        let cur = self.cur.take()?;
        let next = self.read_f32();

        let m = self.params.margin;
        let c = cur.ncols();

        let left = self
            .prev_tail
            .take()
            .unwrap_or_else(|| Array2::<f32>::zeros((0, c)));
        let right = match &next {
            Some(n) => {
                let r = m.min(n.nrows());
                n.slice(s![..r, ..]).to_owned()
            }
            None => Array2::<f32>::zeros((0, c)),
        };

        let valid_start = left.nrows();
        let valid_len = cur.nrows();
        let slab = concatenate(Axis(0), &[left.view(), cur.view(), right.view()])
            .expect("slab parts share width");

        let t = m.min(cur.nrows());
        self.prev_tail = Some(cur.slice(s![cur.nrows() - t.., ..]).to_owned());
        self.cur = next;

        Some((slab, valid_start, valid_len))
    }

    fn fill_queue(&mut self) {
        let mut slabs = Vec::with_capacity(self.params.batch);
        for _ in 0..self.params.batch {
            match self.next_slab() {
                Some(s) => slabs.push(s),
                None => break,
            }
        }
        if slabs.is_empty() {
            self.done = true;
            return;
        }
        let sos = &self.params.sos;
        let padlen = self.params.padlen;
        let whitener = self.whitener.as_ref();
        let results: Vec<Array2<f32>> = slabs
            .par_iter()
            .map(|(slab, vs, vl)| process_slab(slab, *vs, *vl, sos, padlen, whitener))
            .collect();
        self.out_queue.extend(results);
    }

    pub fn next_chunk(&mut self) -> Option<Array2<f32>> {
        while self.out_queue.is_empty() && !self.done {
            self.fill_queue();
        }
        self.out_queue.pop_front()
    }
}
