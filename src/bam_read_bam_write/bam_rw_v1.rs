use crossbeam::channel::bounded;
use gskits::pbar::{DEFAULT_INTERVAL, get_spin_pb};
use rust_htslib::bam::{self, Read};
use std::{fs, path, thread, time::Instant};

pub fn dump_smc_input_bam(subreads_bam_path: &str, o_filepath: &str, bam_threads: Option<usize>) {
    println!("Start");

    let bam_threads = bam_threads.unwrap_or(40);
    if path::Path::new(o_filepath).exists() {
        fs::remove_file(o_filepath).expect(&format!("remove {o_filepath} error"));
    }

    let start_time = Instant::now();
    let (sender, receiver) = bounded::<bam::Record>(1024); // 有限缓冲队列

    // Spawn writer thread
    let o_filepath = o_filepath.to_string();
    let sbr_p = subreads_bam_path.to_string();
    let writer_handle = thread::spawn(move || {
        let pb = get_spin_pb(format!("writing"), DEFAULT_INTERVAL);
        let mut bam_writer = {
            let bam_reader = bam::Reader::from_path(&sbr_p).unwrap();
            let header = bam::Header::from_template(bam_reader.header());
            let writer = bam::Writer::from_path(&o_filepath, &header, bam::Format::Bam)
                .expect(&format!("open {o_filepath} error"));
            writer
        };

        bam_writer.set_threads(bam_threads).unwrap();

        for record in receiver {
            bam_writer.write(&record).unwrap();
            pb.inc(1);
        }
        pb.finish();
    });

    // Reader loop (main thread or you can spawn it as another thread)
    let mut bam_reader = bam::Reader::from_path(subreads_bam_path)
        .expect(&format!("read {subreads_bam_path} error"));
    bam_reader.set_threads(80).unwrap();

    for result in bam_reader.records() {
        let record = result.unwrap();
        // Clone record to send across threads
        sender.send(record).unwrap();
    }

    // Drop sender to signal end of stream
    drop(sender);
    writer_handle.join().unwrap();

    println!("End. ElapsedTime:{}", start_time.elapsed().as_secs());
}
